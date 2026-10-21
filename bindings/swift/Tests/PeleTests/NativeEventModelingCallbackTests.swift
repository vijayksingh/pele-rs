import Foundation
import XCTest
import Pele

private final class EventModelingCallbackFailure: Error {}
@MainActor private enum EventModelingCallbackDocument {
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
@MainActor private final class EventModelingCallbackState {
    var fired = false
    var trace: [[Any]] = []
    var nested: FullRenderedDiagram?
    var nestedRevision: UInt64?
    let failure = EventModelingCallbackFailure()
}

@MainActor final class NativeEventModelingCallbackTests: XCTestCase {
    private func bits(_ n: Double) -> String { String(format: "%016llx", n.bitPattern) }
    private func compare(_ actual: FullRenderedDiagram, _ expected: [String: Any]) throws {
        XCTAssertEqual(actual.svg, try XCTUnwrap(expected["svg"] as? String))
        XCTAssertEqual(bits(actual.width), try XCTUnwrap(expected["width"] as? String))
        XCTAssertEqual(bits(actual.height), try XCTUnwrap(expected["height"] as? String))
        XCTAssertTrue(actual.links.isEmpty)
    }
    private func pure() -> Pele.HostResources {
        var host = Pele.HostResources(); host.textWidth = { Double($0.text.utf16.count) * $0.size * 0.6 }; return host
    }
    private static func ref(_ atom: Pele.PublicGraph.Atom?) throws -> UInt32 {
        guard case .ref(let id) = atom else { throw PeleError(code: "test", message: "EventModeling reference missing") }; return id
    }
    private static func setText(_ graph: inout Pele.PublicGraph, _ key: String, _ value: String, on node: UInt32? = nil) throws {
        let id = try graph.intern(value); try graph.setField(key, value: .string(id), on: node)
    }
    private static func append(_ graph: inout Pele.PublicGraph, on node:UInt32, value:Pele.PublicGraph.Atom) throws {
        guard case .array(let length,_) = graph.nodes[Int(node)] else {throw PeleError(code:"test",message:"EventModeling array missing")}
        try graph.setField(String(length),value:value,on:node)
        graph.nodes[Int(node)] = .array(length:length+1,properties:graph.nodes[Int(node)].properties)
    }
    private static func array(_ graph:inout Pele.PublicGraph,_ values:[Pele.PublicGraph.Atom])throws->Pele.PublicGraph.Atom {
        var properties:[Pele.PublicGraph.Property]=[]
        for (i,value) in values.enumerated(){properties.append(.init(key:try graph.intern(String(i)),value:value))}
        let id=UInt32(graph.nodes.count);graph.nodes.append(.array(length:UInt32(values.count),properties:properties));return .ref(id)
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
        } else { throw PeleError(code: "test", message: "Unexpected EventModeling fixture input") }
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
            if case .map(let entries,_) = node { return ["$map":try entries.map { [try value(graph,$0.key),try value(graph,$0.value)] }] }
            var object: [String: Any] = [:]
            for property in node.properties { if case .undefined = property.value { continue }; object[try XCTUnwrap(graph.strings[Int(property.key)].stringIfWellFormed)] = try value(graph, property.value) }
            return object
        default: throw PeleError(code: "test", message: "Unexpected EventModeling fixture atom")
        }
    }
    private static func state(_ graph:Pele.PublicGraph)throws->[String:Any]{try XCTUnwrap(value(graph,graph.root) as? [String:Any])}
    func testEventModelingCapturedArraysLateFieldsNestedCallbacksAndRecoveryMatchAllOracleRows() throws {
        var path = URL(fileURLWithPath: #filePath); for _ in 0..<5 { path.deleteLastPathComponent() }
        let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-eventmodeling-live-render.json"))) as? [String: Any])
        XCTAssertEqual(fixture["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        let source = try XCTUnwrap(fixture["source"] as? String), rows = try XCTUnwrap(fixture["rows"] as? [[String: Any]])
        XCTAssertEqual(rows.count, 56)
        for row in rows {
            let raw = try XCTUnwrap(row["raw"] as? Bool), responsive = try XCTUnwrap(row["responsive"] as? Bool), mode = try XCTUnwrap(row["mode"] as? String)
            let engine = Pele.Engine()
            let document: EventModelingCallbackDocument = try raw ? .prepared(engine.parsePrepared("eventmodeling", source: source)) : .normal(engine.parseMutable(source))
            defer { try? document.close() }
            var setup=try document.snapshot().graph
            try Self.setText(&setup,"title","Initial")
            let lanes=try Self.ref(setup.field("lanes")),lane=try Self.ref(setup.field("0",on:lanes)),boxes=try Self.ref(setup.field("boxes")),firstBox=try Self.ref(setup.field("0",on:boxes)),second=try Self.ref(setup.field("1",on:boxes)),third=try Self.ref(setup.field("2",on:boxes)),relations=try Self.ref(setup.field("relations")),relation=try Self.ref(setup.field("0",on:relations)),notes=try Self.ref(setup.field("notes")),note=try Self.ref(setup.field("0",on:notes)),specs=try Self.ref(setup.field("specifications")),spec=try Self.ref(setup.field("0",on:specs))
            try document.apply(setup)
            let observed=EventModelingCallbackState()
            var resources = pure()
            resources.textWidth = { request in try MainActor.assumeIsolated {
                observed.trace.append([request.text, String(format: "%016llx", request.size.bitPattern), Int(request.style)])
                let trigger=["box-data":"AddItem","box-late":"Initial","notes":"Note · 02 AddItem","specifications":"Given / When / Then · 02 AddItem"][mode]
                if !observed.fired && (trigger == nil || request.text == trigger) {
                    observed.fired = true
                    XCTAssertThrowsError(try document.close())
                    var graph = try document.snapshot().graph
                    switch mode {
                    case "lane-label":try Self.setText(&graph,"label","Changed lane",on:lane)
                    case "lanes-replace":let secondLane=try XCTUnwrap(graph.field("1",on:lanes)),thirdLane=try XCTUnwrap(graph.field("2",on:lanes));let v=try Self.array(&graph,[secondLane,.ref(lane),thirdLane]);try graph.setField("lanes",value:v)
                    case "boxes-replace":let v=try Self.array(&graph,[.ref(second),.ref(third)]);try graph.setField("boxes",value:v)
                    case "box-name":try Self.setText(&graph,"name","Changed first",on:firstBox);try Self.setText(&graph,"name","Changed second",on:second)
                    case "box-data":try Self.setText(&graph,"data","Changed data",on:second);try Self.setText(&graph,"name","After captured name",on:second)
                    case "box-late":try Self.setText(&graph,"kind","processor",on:firstBox);try graph.setField("reset",value:.bool(true),on:firstBox);try Self.setText(&graph,"id","Changed id",on:firstBox);try Self.setText(&graph,"label","Late lane",on:lane)
                    case "box-append":let id=UInt32(graph.nodes.count);graph.nodes.append(graph.nodes[Int(third)]);try Self.setText(&graph,"id","04",on:id);try Self.setText(&graph,"name","Appended",on:id);try Self.append(&graph,on:boxes,value:.ref(id))
                    case "relations":try graph.setField("target",value:.ref(third),on:relation);let id=UInt32(graph.nodes.count);graph.nodes.append(.object(properties:[]));try graph.setField("source",value:.ref(firstBox),on:id);try graph.setField("target",value:.ref(third),on:id);let v=try Self.array(&graph,[.ref(id)]);try graph.setField("relations",value:v)
                    case "notes":try Self.setText(&graph,"text","After captured note",on:note);let v=try Self.input(&graph,["frame":"01","text":"Appended note"]);try Self.append(&graph,on:notes,value:v)
                    case "specifications":let given=try Self.ref(graph.field("given",on:spec)),v=try Self.input(&graph,"Added");try Self.append(&graph,on:given,value:v);let replacement=try Self.input(&graph,["Changed"]);try graph.setField("then",value:replacement,on:spec)
                    case "title-accessibility":try Self.setText(&graph,"title","Changed title");try Self.setText(&graph,"accTitle","Changed access");try Self.setText(&graph,"accDescr","Changed description")
                    case "throw":try Self.setText(&graph,"title","Before throw");try Self.setText(&graph,"name","Saved first",on:firstBox)
                    case "nested":try Self.setText(&graph,"title","Nested title")
                    case "baseline":break
                    default:throw PeleError(code:"test",message:"Unknown EventModeling mode")
                    }
                    if mode != "baseline" { try document.apply(graph) }
                    if mode == "throw" { throw observed.failure }
                    if mode == "nested" {
                        var inner = Pele.HostResources(); inner.textWidth = { Double($0.text.utf16.count) * $0.size * 0.6 }
                        observed.nested = try document.render(inner); observed.nestedRevision = document.revision
                        var latest = try document.snapshot().graph; try Self.setText(&latest, "title", "After nested"); try document.apply(latest)
                    }
                }
                return Double(request.text.utf16.count) * request.size * 0.6
            }}
            let first = try XCTUnwrap(row["first"] as? [String: Any])
            if mode == "throw" { XCTAssertEqual(first["original"] as? Bool, true); XCTAssertThrowsError(try document.render(resources, responsive: responsive)) { XCTAssertTrue(($0 as AnyObject) === observed.failure) } }
            else { try compare(document.render(resources, responsive: responsive), first) }
            XCTAssertTrue(observed.fired)
            XCTAssertTrue(NSArray(array: observed.trace).isEqual(to: try XCTUnwrap(row["trace"] as? [[Any]])), "\(mode)/\(raw)/\(responsive) trace")
            let after = try document.snapshot(); XCTAssertEqual(after.revision, document.revision)
            XCTAssertTrue(NSDictionary(dictionary: try Self.state(after.graph)).isEqual(to: try XCTUnwrap(row["after"] as? [String: Any])), mode + " state")
            XCTAssertEqual(try Self.ref(after.graph.field("lanes"))==lanes,row["lanesCurrent"] as? Bool)
            XCTAssertEqual(try Self.ref(after.graph.field("boxes"))==boxes,row["boxesCurrent"] as? Bool)
            XCTAssertEqual(try Self.ref(after.graph.field("relations"))==relations,row["relationsCurrent"] as? Bool)
            for(name,id)in[("originalLane",lane),("originalFirst",firstBox),("originalSecond",second),("originalRelation",relation),("originalNote",note),("originalSpecification",spec)]{XCTAssertTrue(NSDictionary(dictionary:try XCTUnwrap(Self.value(after.graph,.ref(id)) as? [String:Any])).isEqual(to:try XCTUnwrap(row[name] as? [String:Any])),mode+name)}
            for(name,id)in[("originalLanes",lanes),("originalBoxes",boxes),("originalRelations",relations),("originalNotes",notes)]{XCTAssertTrue(NSArray(array:try XCTUnwrap(Self.value(after.graph,.ref(id)) as? [Any])).isEqual(to:try XCTUnwrap(row[name] as? [Any])),mode+name)}
            if mode == "nested" { try compare(XCTUnwrap(observed.nested), XCTUnwrap(row["nested"] as? [String: Any])); XCTAssertGreaterThan(document.revision, try XCTUnwrap(observed.nestedRevision)) }
            let recovery = EventModelingCallbackState(); var plain = pure()
            plain.textWidth = { request in MainActor.assumeIsolated { recovery.trace.append([request.text, String(format: "%016llx", request.size.bitPattern), Int(request.style)]); return Double(request.text.utf16.count) * request.size * 0.6 } }
            try compare(document.render(plain, responsive: responsive), XCTUnwrap(row["recovery"] as? [String: Any]))
            XCTAssertTrue(NSArray(array: recovery.trace).isEqual(to: try XCTUnwrap(row["recoveryTrace"] as? [[Any]])), mode + " recovery trace")
        }
    }
}
