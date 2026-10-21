import Foundation
import XCTest
import Pele

private final class KanbanCallbackFailure: Error {}
@MainActor private enum KanbanCallbackDocument {
    case normal(Pele.MutableDocument), prepared(Pele.PreparedDocument)
    var revision: UInt64 { switch self { case .normal(let d): d.revision; case .prepared(let d): d.revision } }
    func snapshot() throws -> Pele.PublicSnapshot { switch self { case .normal(let d): try d.snapshot(); case .prepared(let d): try d.snapshot() } }
    func apply(_ graph: Pele.PublicGraph) throws {
        switch self { case .normal(let d): _ = try d.apply(graph, expectedRevision: d.revision); case .prepared(let d): _ = try d.apply(graph, expectedRevision: d.revision) }
    }
    func render(_ resources: Pele.HostResources, responsive: Bool = false) throws -> FullRenderedDiagram {
        var options = Pele.Engine.Options(); options.maxWidth = responsive ? 300 : nil
        switch self { case .normal(let d): return try d.render(options: options, resources: resources); case .prepared(let d): return try d.render(options: options, resources: resources) }
    }
    func close() throws { switch self { case .normal(let d): try d.close(); case .prepared(let d): try d.close() } }
}
@MainActor private final class KanbanCallbackState {
    var fired = false
    var iconCalls = 0
    var trace: [[Any]] = []
    var nested: FullRenderedDiagram?
    var nestedRevision: UInt64?
    let failure = KanbanCallbackFailure()
}

@MainActor final class NativeKanbanCallbackTests: XCTestCase {
    private func bits(_ n: Double) -> String { String(format: "%016llx", n.bitPattern) }
    private func compare(_ actual: FullRenderedDiagram, _ expected: [String: Any]) throws {
        XCTAssertEqual(actual.svg, try XCTUnwrap(expected["svg"] as? String))
        XCTAssertEqual(bits(actual.width), try XCTUnwrap(expected["width"] as? String))
        XCTAssertEqual(bits(actual.height), try XCTUnwrap(expected["height"] as? String))
        XCTAssertTrue(actual.links.isEmpty)
    }
    private static func pure() -> Pele.HostResources {
        var host = Pele.HostResources(); host.textWidth = { Double($0.text.utf16.count) * $0.size * 0.6 }; host.icon = { _ in .init(trustedMarkup: "<path d=\"M0 0h4v4z\"/>") }; return host
    }
    private static func sameRef(_ a:Pele.PublicGraph.Atom?,_ b:Pele.PublicGraph.Atom?)->Bool {
        if case (.some(.ref(let x)),.some(.ref(let y))) = (a,b) { return x == y }; return false
    }
    private static func ref(_ atom: Pele.PublicGraph.Atom?) throws -> UInt32 {
        guard case .ref(let id) = atom else { throw PeleError(code: "test", message: "Kanban reference missing") }; return id
    }
    private static func setText(_ graph: inout Pele.PublicGraph, _ key: String, _ value: String, on node: UInt32? = nil) throws {
        let id = try graph.intern(value); try graph.setField(key, value: .string(id), on: node)
    }
    private static func append(_ graph: inout Pele.PublicGraph, on node:UInt32, value:Pele.PublicGraph.Atom) throws {
        guard case .array(let length,_) = graph.nodes[Int(node)] else {throw PeleError(code:"test",message:"Kanban array missing")}
        try graph.setField(String(length),value:value,on:node)
        graph.nodes[Int(node)] = .array(length:length+1,properties:graph.nodes[Int(node)].properties)
    }
    private static func input(_ graph: inout Pele.PublicGraph, _ value: Any) throws -> Pele.PublicGraph.Atom {
        if let string = value as? String { return .string(try graph.intern(string)) }
        if let number = value as? NSNumber { return .number(number.doubleValue) }
        let id = UInt32(graph.nodes.count); graph.nodes.append(.object(properties: []))
        var properties: [Pele.PublicGraph.Property] = []
        if let array = value as? [Any] {
            for (i, value) in array.enumerated() { let key = try graph.intern(String(i)), atom = try input(&graph, value); properties.append(.init(key: key, value: atom)) }
            graph.nodes[Int(id)] = .array(length: UInt32(array.count), properties: properties)
        } else if let object = value as? [String: Any] {
            for (name, value) in object.sorted(by: { $0.key < $1.key }) { let key = try graph.intern(name), atom = try input(&graph, value); properties.append(.init(key: key, value: atom)) }
            graph.nodes[Int(id)] = .object(properties: properties)
        } else { throw PeleError(code: "test", message: "Unexpected Kanban fixture input") }
        return .ref(id)
    }
    private static func value(_ graph: Pele.PublicGraph, _ atom: Pele.PublicGraph.Atom) throws -> Any {
        switch atom {
        case .undefined, .null: return NSNull()
        case .bool(let b): return b
        case .number(let n): return n
        case .string(let id): return try XCTUnwrap(graph.strings[Int(id)].stringIfWellFormed)
        case .ref(let id):
            let node = graph.nodes[Int(id)]
            if case .array(let length, _) = node { return try (0..<length).map { try value(graph, graph.field(String($0), on: id) ?? .undefined) } }
            var object: [String: Any] = [:]
            for property in node.properties { object[try XCTUnwrap(graph.strings[Int(property.key)].stringIfWellFormed)] = try value(graph, property.value) }
            return object
        default: throw PeleError(code: "test", message: "Unexpected Kanban fixture atom")
        }
    }
    private static func state(_ graph: Pele.PublicGraph) throws -> [String: Any] {
        var result: [String: Any] = [:]
        for key in ["title", "sections"] { result[key] = try value(graph, graph.field(key) ?? .undefined) }
        return result
    }
    private static func mutate(_ document: KanbanCallbackDocument, _ mode: String, _ observed: KanbanCallbackState) throws {
        observed.fired = true
        XCTAssertThrowsError(try document.close())
        var graph = try document.snapshot().graph
        let sections = try ref(graph.field("sections")), section = try ref(graph.field("0", on: sections)), items = try ref(graph.field("items", on: section)), first = try ref(graph.field("0", on: items))
        switch mode {
        case "header-edit": try setText(&graph,"label","Changed header",on:section); try setText(&graph,"id","changed-header",on:section)
        case "item-edit": let second=try ref(graph.field("1",on:items)); try setText(&graph,"label","Changed first",on:first);try setText(&graph,"label","Changed second",on:second)
        case "item-replace": let v=try input(&graph,["id":"replacement","level":1,"label":"Replacement"]);try graph.setField("1",value:v,on:items)
        case "items-replace": let v=try input(&graph,[["id":"replacement","level":1,"label":"Replacement"]]);try graph.setField("items",value:v,on:section)
        case "sections-replace": let v=try input(&graph,[["id":"new-root","level":0,"label":"New root","items":[["id":"new","level":1,"label":"New"]]]]);try graph.setField("sections",value:v)
        case "item-append": let v=try input(&graph,["id":"append","level":1,"label":"Appended"]);try append(&graph,on:items,value:v)
        case "section-append": let v=try input(&graph,["id":"append-section","level":0,"label":"Appended section","items":[["id":"append","level":1,"label":"Appended"]]]);try append(&graph,on:sections,value:v)
        case "section-replace": let v=try input(&graph,["id":"replacement-section","level":0,"label":"Replacement section","items":[["id":"replacement","level":1,"label":"Replacement"]]]);try graph.setField("1",value:v,on:sections)
        case "late-fields": for(k,v)in[("id","late-id"),("priority","high"),("cssClasses","late-card"),("ticket","Changed ticket"),("assigned","Grace")]{try setText(&graph,k,v,on:first)};try setText(&graph,"cssClasses","late-column",on:section);try setText(&graph,"title","Late title")
        case "throw": try setText(&graph,"title","Before throw");try setText(&graph,"label","Saved label",on:first)
        case "nested": try setText(&graph,"title","Nested title")
        case "icon-label": try setText(&graph,"label","After icon",on:first);try setText(&graph,"ticket","After icon ticket",on:first)
        case "icon-paint": try setText(&graph,"id","paint-id",on:first);try setText(&graph,"cssClasses","paint-card",on:first);try setText(&graph,"ticket","Paint ticket",on:first);let v=try input(&graph,[Any]());try graph.setField("items",value:v,on:section)
        case "icon-throw": try setText(&graph,"label","Saved icon label",on:first)
        case "baseline": break
        default: throw PeleError(code:"test",message:"Unknown Kanban mode")
        }
        if mode != "baseline" {try document.apply(graph)}
        if mode == "throw" || mode == "icon-throw" {throw observed.failure}
        if mode == "nested" {
            observed.nested = try document.render(pure());observed.nestedRevision=document.revision
            var latest=try document.snapshot().graph;try setText(&latest,"title","After nested");try document.apply(latest)
        }
    }
    func testKanbanCapturedArraysLateFieldsNestedCallbacksAndRecoveryMatchAllOracleRows() throws {
        var path = URL(fileURLWithPath: #filePath); for _ in 0..<5 { path.deleteLastPathComponent() }
        let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-kanban-live-render.json"))) as? [String: Any])
        XCTAssertEqual(fixture["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        let source = try XCTUnwrap(fixture["source"] as? String), rows = try XCTUnwrap(fixture["rows"] as? [[String: Any]])
        XCTAssertEqual(rows.count, 54)
        for row in rows {
            let raw = try XCTUnwrap(row["raw"] as? Bool), responsive = try XCTUnwrap(row["responsive"] as? Bool), mode = try XCTUnwrap(row["mode"] as? String)
            let engine = Pele.Engine()
            let document: KanbanCallbackDocument = try raw ? .prepared(engine.parsePrepared("kanban", source: source)) : .normal(engine.parseMutable(source))
            defer { try? document.close() }
            var setup=try document.snapshot().graph
            let sections=try Self.ref(setup.field("sections")), section=try Self.ref(setup.field("0",on:sections)), items=try Self.ref(setup.field("items",on:section)), original=try Self.ref(setup.field("0",on:items))
            try Self.setText(&setup,"title","Initial");try Self.setText(&setup,"icon","header",on:section)
            for(k,v)in[("icon","item"),("ticket","T1"),("assigned","Ada")]{try Self.setText(&setup,k,v,on:original)}
            try document.apply(setup)
            let observed=KanbanCallbackState()
            var resources=Self.pure()
            resources.textWidth={request in try MainActor.assumeIsolated{
                observed.trace.append(["width",request.text,String(format:"%016llx",request.size.bitPattern),Int(request.style)])
                let trigger=mode == "item-append" ? request.text == "First task" : mode == "late-fields" ? request.text == "Initial" : true
                if !observed.fired && !mode.hasPrefix("icon-") && trigger {try Self.mutate(document,mode,observed)}
                return Double(request.text.utf16.count)*request.size*0.6
            }}
            resources.icon={name in try MainActor.assumeIsolated{
                observed.trace.append(["icon",name]);observed.iconCalls += 1
                if !observed.fired && mode.hasPrefix("icon-") && ((mode == "icon-paint" && observed.iconCalls == 3) || (mode != "icon-paint" && name == "item")){try Self.mutate(document,mode,observed)}
                return .init(trustedMarkup:"<path d=\"M0 0h4v4z\"/>")
            }}
            let first = try XCTUnwrap(row["first"] as? [String: Any])
            if mode == "throw" || mode == "icon-throw" { XCTAssertEqual(first["original"] as? Bool, true); XCTAssertThrowsError(try document.render(resources, responsive: responsive)) { XCTAssertTrue(($0 as AnyObject) === observed.failure) } }
            else { try compare(document.render(resources, responsive: responsive), first) }
            XCTAssertTrue(observed.fired)
            XCTAssertTrue(NSArray(array: observed.trace).isEqual(to: try XCTUnwrap(row["trace"] as? [[Any]])), "\(mode)/\(raw)/\(responsive) trace")
            let after = try document.snapshot(); XCTAssertEqual(after.revision, document.revision)
            XCTAssertTrue(NSDictionary(dictionary: try Self.state(after.graph)).isEqual(to: try XCTUnwrap(row["after"] as? [String: Any])), mode + " state")
            let currentSections=try Self.ref(after.graph.field("sections")), currentSection=try Self.ref(after.graph.field("0",on:currentSections)), currentItems=try Self.ref(after.graph.field("items",on:currentSection))
            XCTAssertEqual(currentSections==sections,row["sectionsCurrent"] as? Bool);XCTAssertEqual(currentItems==items,row["itemsCurrent"] as? Bool)
            XCTAssertEqual(try Self.sameRef(after.graph.field("0",on:currentItems),.ref(original)),row["itemCurrent"] as? Bool)
            for(name,node)in[("originalSections",sections),("originalItems",items)]{XCTAssertTrue(NSArray(array:try XCTUnwrap(Self.value(after.graph,.ref(node)) as? [Any])).isEqual(to:try XCTUnwrap(row[name] as? [Any])),mode+name)}
            XCTAssertTrue(NSDictionary(dictionary:try XCTUnwrap(Self.value(after.graph,.ref(original)) as? [String:Any])).isEqual(to:try XCTUnwrap(row["originalItem"] as? [String:Any])),mode+" original item")
            if mode == "nested" { try compare(XCTUnwrap(observed.nested), XCTUnwrap(row["nested"] as? [String: Any])); XCTAssertGreaterThan(document.revision, try XCTUnwrap(observed.nestedRevision)) }
            let recovery = KanbanCallbackState(); var plain = Self.pure()
            plain.textWidth = { request in MainActor.assumeIsolated { recovery.trace.append(["width", request.text, String(format: "%016llx", request.size.bitPattern), Int(request.style)]); return Double(request.text.utf16.count) * request.size * 0.6 } }
            plain.icon={name in MainActor.assumeIsolated{recovery.trace.append(["icon",name]);return .init(trustedMarkup:"<path d=\"M0 0h4v4z\"/>")}}
            try compare(document.render(plain, responsive: responsive), XCTUnwrap(row["recovery"] as? [String: Any]))
            XCTAssertTrue(NSArray(array: recovery.trace).isEqual(to: try XCTUnwrap(row["recoveryTrace"] as? [[Any]])), mode + " recovery trace")
        }
    }
}
