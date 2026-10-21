import Foundation
import XCTest
import Pele

private final class WardleyCallbackFailure: Error {}
@MainActor private enum WardleyCallbackDocument {
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
@MainActor private final class WardleyCallbackState {
    var fired = false
    var trace: [[Any]] = []
    var nested: FullRenderedDiagram?
    var nestedRevision: UInt64?
    let failure = WardleyCallbackFailure()
}

@MainActor final class NativeWardleyCallbackTests: XCTestCase {
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
        guard case .ref(let id) = atom else { throw PeleError(code: "test", message: "Wardley reference missing") }; return id
    }
    private static func setText(_ graph: inout Pele.PublicGraph, _ key: String, _ value: String, on node: UInt32? = nil) throws {
        let id = try graph.intern(value); try graph.setField(key, value: .string(id), on: node)
    }
    private static func append(_ graph: inout Pele.PublicGraph, on node:UInt32, value:Pele.PublicGraph.Atom) throws {
        guard case .array(let length,_) = graph.nodes[Int(node)] else {throw PeleError(code:"test",message:"Wardley array missing")}
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
        } else { throw PeleError(code: "test", message: "Unexpected Wardley fixture input") }
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
            for property in node.properties { if case .undefined = property.value { continue }; object[try XCTUnwrap(graph.strings[Int(property.key)].stringIfWellFormed)] = try value(graph, property.value) }
            return object
        default: throw PeleError(code: "test", message: "Unexpected Wardley fixture atom")
        }
    }
    private static func state(_ graph:Pele.PublicGraph)throws->[String:Any]{try XCTUnwrap(value(graph,graph.root) as? [String:Any])}
    func testWardleyCapturedArraysLateFieldsNestedCallbacksAndRecoveryMatchAllOracleRows() throws {
        var path = URL(fileURLWithPath: #filePath); for _ in 0..<5 { path.deleteLastPathComponent() }
        let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-wardley-live-render.json"))) as? [String: Any])
        XCTAssertEqual(fixture["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        let source = try XCTUnwrap(fixture["source"] as? String), rows = try XCTUnwrap(fixture["rows"] as? [[String: Any]])
        XCTAssertEqual(rows.count, 60)
        for row in rows {
            let raw = try XCTUnwrap(row["raw"] as? Bool), responsive = try XCTUnwrap(row["responsive"] as? Bool), mode = try XCTUnwrap(row["mode"] as? String)
            let engine = Pele.Engine()
            let document: WardleyCallbackDocument = try raw ? .prepared(engine.parsePrepared("wardley", source: source)) : .normal(engine.parseMutable(source))
            defer { try? document.close() }
            var setup=try document.snapshot().graph
            let nodes=try Self.ref(setup.field("nodes")), original=try Self.ref(setup.field("0",on:nodes)), links=try Self.ref(setup.field("links")), link=try Self.ref(setup.field("0",on:links)), annotations=try Self.ref(setup.field("annotations")), annotation=try Self.ref(setup.field("0",on:annotations)), notes=try Self.ref(setup.field("notes")), note=try Self.ref(setup.field("0",on:notes)), axes=try Self.ref(setup.field("axes"))
            let stages=try Self.input(&setup,["Genesis","Custom","Product","Commodity"]);try setup.setField("stages",value:stages,on:axes);try Self.setText(&setup,"label","Edge caption",on:link);try document.apply(setup)
            let observed=WardleyCallbackState()
            var resources = pure()
            resources.textWidth = { request in try MainActor.assumeIsolated {
                observed.trace.append([request.text, String(format: "%016llx", request.size.bitPattern), Int(request.style)])
                let trigger=["node-late":"User","link-late":"Edge caption","annotation-late":"1","note-late":"Build carefully"][mode]
                if !observed.fired && (trigger == nil || request.text == trigger) {
                    observed.fired = true
                    XCTAssertThrowsError(try document.close())
                    var graph = try document.snapshot().graph
                    switch mode {
                    case "stage-edit":let array=try Self.ref(stages);try Self.setText(&graph,"1","Changed stage",on:array);let v=try Self.input(&graph,"Extra");try Self.append(&graph,on:array,value:v)
                    case "stages-replace":let v=try Self.input(&graph,["Replacement"]);try graph.setField("stages",value:v,on:axes)
                    case "axis-edit":try Self.setText(&graph,"xLabel","Changed evolution",on:axes);try Self.setText(&graph,"yLabel","Changed visibility",on:axes);let v=try Self.input(&graph,["width":700,"height":400]);try graph.setField("size",value:v)
                    case "nodes-replace":let v=try Self.input(&graph,[["id":"Replacement","label":"Replacement node","x":50,"y":50]]);try graph.setField("nodes",value:v)
                    case "node-late":try Self.setText(&graph,"id","ChangedUser",on:original);try Self.setText(&graph,"label","After captured label",on:original);try graph.setField("x",value:.number(80),on:original);try graph.setField("labelOffsetX",value:.number(20),on:original);try graph.setField("inPipeline",value:.bool(true),on:original)
                    case "link-late":try Self.setText(&graph,"label","After captured edge",on:link);try graph.setField("dashed",value:.bool(true),on:link);try Self.setText(&graph,"source","ChangedSource",on:link)
                    case "annotation-late":try graph.setField("number",value:.number(3),on:annotation);try Self.setText(&graph,"text","Changed annotation",on:annotation);let v=try Self.input(&graph,[["x":20,"y":30]]);try graph.setField("coordinates",value:v,on:annotation)
                    case "note-late":try Self.setText(&graph,"text","After captured note",on:note);try graph.setField("x",value:.number(15),on:note);try graph.setField("y",value:.number(20),on:note)
                    case "forces":let array=try Self.ref(graph.field("accelerators")), item=try Self.ref(graph.field("0",on:array));try Self.setText(&graph,"name","Changed accelerator",on:item);let v=try Self.input(&graph,[["name":"Replacement force","x":10,"y":10]]);try graph.setField("deaccelerators",value:v)
                    case "title-accessibility":try Self.setText(&graph,"title","Changed title");try Self.setText(&graph,"accTitle","Changed access");try Self.setText(&graph,"accDescr","Changed description")
                    case "throw":try Self.setText(&graph,"title","Before throw");try Self.setText(&graph,"label","Saved user",on:original)
                    case "nested":try Self.setText(&graph,"title","Nested title")
                    case "pipeline-edit":let pipelines=try Self.ref(graph.field("pipelines")), pipeline=try Self.ref(graph.field("0",on:pipelines)), ids=try Self.ref(graph.field("componentIds",on:pipeline)), a=try XCTUnwrap(graph.field("0",on:ids)), b=try XCTUnwrap(graph.field("1",on:ids));try graph.setField("0",value:b,on:ids);try graph.setField("1",value:a,on:ids);let item=try Self.ref(graph.field("3",on:nodes));try graph.setField("x",value:.number(5),on:item)
                    case "append":let v=try Self.input(&graph,["id":"Extra","label":"Appended node","x":60,"y":20]);try Self.append(&graph,on:nodes,value:v);let n=try Self.input(&graph,["text":"Appended note","x":20,"y":80]);try Self.append(&graph,on:notes,value:n)
                    case "baseline":break
                    default:throw PeleError(code:"test",message:"Unknown Wardley mode")
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
            let currentNodes=try Self.ref(after.graph.field("nodes")),currentFirst=try after.graph.field("0",on:currentNodes) ?? .undefined
            XCTAssertEqual(currentNodes==nodes,row["nodesCurrent"] as? Bool);var firstCurrent = false; if case .ref(let id) = currentFirst { firstCurrent = id == original }; XCTAssertEqual(firstCurrent,row["firstCurrent"] as? Bool); XCTAssertEqual(try Self.ref(after.graph.field("stages",on:axes)) == Self.ref(stages),row["stagesCurrent"] as? Bool)
            for(name,value)in[("originalNodes",Pele.PublicGraph.Atom.ref(nodes)),("originalLinks",.ref(links)),("originalAnnotations",.ref(annotations)),("originalNotes",.ref(notes)),("originalStages",stages)]{XCTAssertTrue(NSArray(array:try XCTUnwrap(Self.value(after.graph,value) as? [Any])).isEqual(to:try XCTUnwrap(row[name] as? [Any])),mode+name)}
            for(name,value)in[("originalFirst",Pele.PublicGraph.Atom.ref(original)),("originalLink",.ref(link)),("originalAnnotation",.ref(annotation)),("originalNote",.ref(note))]{XCTAssertTrue(NSDictionary(dictionary:try XCTUnwrap(Self.value(after.graph,value) as? [String:Any])).isEqual(to:try XCTUnwrap(row[name] as? [String:Any])),mode+name)}
            if mode == "nested" { try compare(XCTUnwrap(observed.nested), XCTUnwrap(row["nested"] as? [String: Any])); XCTAssertGreaterThan(document.revision, try XCTUnwrap(observed.nestedRevision)) }
            let recovery = WardleyCallbackState(); var plain = pure()
            plain.textWidth = { request in MainActor.assumeIsolated { recovery.trace.append([request.text, String(format: "%016llx", request.size.bitPattern), Int(request.style)]); return Double(request.text.utf16.count) * request.size * 0.6 } }
            try compare(document.render(plain, responsive: responsive), XCTUnwrap(row["recovery"] as? [String: Any]))
            XCTAssertTrue(NSArray(array: recovery.trace).isEqual(to: try XCTUnwrap(row["recoveryTrace"] as? [[Any]])), mode + " recovery trace")
        }
    }
}
