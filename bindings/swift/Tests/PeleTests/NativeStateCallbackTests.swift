import Foundation
import XCTest
import Pele

private final class StateCallbackFailure: Error {}
@MainActor private enum StateCallbackDocument {
    case normal(Pele.MutableDocument), prepared(Pele.PreparedDocument)
    var revision: UInt64 { switch self { case .normal(let d): d.revision; case .prepared(let d): d.revision } }
    func snapshot() throws -> Pele.PublicSnapshot { switch self { case .normal(let d): try d.snapshot(); case .prepared(let d): try d.snapshot() } }
    func apply(_ graph: Pele.PublicGraph) throws {
        switch self { case .normal(let d): _ = try d.apply(graph, expectedRevision: d.revision); case .prepared(let d): _ = try d.apply(graph, expectedRevision: d.revision) }
    }
    func render(_ resources: Pele.HostResources, responsive: Bool = false) throws -> FullRenderedDiagram {
        var options = Pele.Engine.Options(); options.maxWidth = responsive ? 160 : nil
        switch self { case .normal(let d): return try d.render(options: options, resources: resources); case .prepared(let d): return try d.render(options: options, resources: resources) }
    }
    func invoke(_ method:UInt32,_ arguments:[Pele.PublicGraph.Atom])throws{let root=try snapshot().graph.root;switch self{case .normal(let d):let out=try d.invoke(method:method,receiver:root,arguments:arguments);guard case .returned = out.result else {throw PeleError(code:"test",message:"State helper failed")};case .prepared(let d):let out=try d.invoke(method:method,receiver:root,arguments:arguments);guard case .returned = out.result else {throw PeleError(code:"test",message:"State helper failed")};}}
    func close() throws { switch self { case .normal(let d): try d.close(); case .prepared(let d): try d.close() } }
}
@MainActor private final class StateCallbackState {
    var fired = false
    var trace: [[Any]] = []
    var nested: FullRenderedDiagram?
    var nestedRevision: UInt64?
    let failure = StateCallbackFailure()
}

@MainActor final class NativeStateCallbackTests: XCTestCase {
    private func bits(_ n: Double) -> String { String(format: "%016llx", n.bitPattern) }
    private func compare(_ actual: FullRenderedDiagram, _ expected: [String: Any]) throws {
        XCTAssertEqual(actual.svg, try XCTUnwrap(expected["svg"] as? String))
        XCTAssertEqual(bits(actual.width), try XCTUnwrap(expected["width"] as? String))
        XCTAssertEqual(bits(actual.height), try XCTUnwrap(expected["height"] as? String))
        let links=try XCTUnwrap(JSONSerialization.jsonObject(with:JSONEncoder().encode(actual.links)) as? [Any]);XCTAssertTrue(NSArray(array:links).isEqual(to:try XCTUnwrap(expected["links"] as? [Any])))
    }
    private func pure() -> Pele.HostResources {
        var host = Pele.HostResources(); host.textWidth = { Double($0.text.utf16.count) * $0.size * 0.6 }; return host
    }
    private static func ref(_ atom: Pele.PublicGraph.Atom?) throws -> UInt32 {
        guard case .ref(let id) = atom else { throw PeleError(code: "test", message: "State reference missing") }; return id
    }
    private static func setText(_ graph: inout Pele.PublicGraph, _ key: String, _ value: String, on node: UInt32? = nil) throws {
        let id = try graph.intern(value); try graph.setField(key, value: .string(id), on: node)
    }
    private static func append(_ graph: inout Pele.PublicGraph, on node:UInt32, value:Pele.PublicGraph.Atom) throws {
        guard case .array(let length,_) = graph.nodes[Int(node)] else {throw PeleError(code:"test",message:"State array missing")}
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
        } else { throw PeleError(code: "test", message: "Unexpected State fixture input") }
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
        default: throw PeleError(code: "test", message: "Unexpected State fixture atom")
        }
    }
    private static func state(_ graph:Pele.PublicGraph)throws->[String:Any]{try XCTUnwrap(value(graph,graph.root) as? [String:Any])}
    func testStateCapturedArraysLateFieldsNestedCallbacksAndRecoveryMatchAllOracleRows() throws {
        var path = URL(fileURLWithPath: #filePath); for _ in 0..<5 { path.deleteLastPathComponent() }
        let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-state-live-render.json"))) as? [String: Any])
        XCTAssertEqual(fixture["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        let source = try XCTUnwrap(fixture["source"] as? String), rows = try XCTUnwrap(fixture["rows"] as? [[String: Any]])
        XCTAssertEqual(rows.count, 64)
        for row in rows {
            let raw = try XCTUnwrap(row["raw"] as? Bool), responsive = try XCTUnwrap(row["responsive"] as? Bool), mode = try XCTUnwrap(row["mode"] as? String)
            let engine = Pele.Engine()
            let document: StateCallbackDocument = try raw ? .prepared(engine.parsePrepared("state", source: source)) : .normal(engine.parseMutable(source))
            defer { try? document.close() }
            var setup=try document.snapshot().graph
            try Self.setText(&setup,"title","Initial")
            let originalGraph=try Self.ref(setup.field("graph")),nodes=try Self.ref(setup.field("nodes",on:originalGraph)),group=try Self.ref(setup.field("0",on:nodes)),alpha=try Self.ref(setup.field("1",on:nodes)),beta=try Self.ref(setup.field("2",on:nodes)),edges=try Self.ref(setup.field("edges",on:originalGraph)),edge=try Self.ref(setup.field("0",on:edges)),description=try Self.ref(setup.field("description",on:alpha)),styles=try Self.ref(setup.field("cssStyles",on:beta))
            try Self.setText(&setup,"dir","LR",on:group);let red=try Self.input(&setup,"fill:red");try Self.append(&setup,on:styles,value:red);try document.apply(setup)
            let observed=StateCallbackState()
            var resources = pure()
            resources.textWidth = { request in try MainActor.assumeIsolated {
                observed.trace.append([request.text, String(format: "%016llx", request.size.bitPattern), Int(request.style)])
                let trigger=["description-replace":"Alpha","description-append":"Detail one","late-fields":"transition","edge-late":"transition"][mode]
                if !observed.fired && (trigger == nil || request.text == trigger) {
                    observed.fired = true
                    XCTAssertThrowsError(try document.close())
                    var graph = try document.snapshot().graph
                    var calls:[(UInt32,[Pele.PublicGraph.Atom])]=[]
                    switch mode {
                    case "node-label":try Self.setText(&graph,"label","Changed Alpha",on:alpha);try Self.setText(&graph,"label","Changed Beta",on:beta)
                    case "description-replace":let v=try Self.input(&graph,["Replacement one","Replacement two"]);try graph.setField("description",value:v,on:alpha)
                    case "description-append":let v=try Self.input(&graph,"Appended detail");try Self.append(&graph,on:description,value:v);let replacement=try Self.input(&graph,["Replaced after capture"]);try graph.setField("description",value:replacement,on:alpha)
                    case "styles-replace":let v=try Self.input(&graph,["fill:green"]);try graph.setField("cssStyles",value:v,on:beta)
                    case "styles-mutate":try Self.setText(&graph,"0","fill:green",on:styles)
                    case "late-fields":try Self.setText(&graph,"cssClasses","late",on:beta);try Self.setText(&graph,"id","Changed B",on:beta)
                    case "graph-replace":let v=try Self.input(&graph,["nodes":[Any](),"edges":[Any]()]);try graph.setField("graph",value:v)
                    case "edges-replace":let v=try Self.input(&graph,[["id":"new-edge","start":"B","end":"A","label":"Replacement edge","classes":"transition"]]);try graph.setField("edges",value:v,on:originalGraph)
                    case "edge-late":try Self.setText(&graph,"id","Changed edge",on:edge);try Self.setText(&graph,"pattern","dashed",on:edge);try Self.setText(&graph,"label","After captured label",on:edge);let v=try Self.input(&graph,["id":"new-edge","start":"B","end":"A","label":"Appended edge","classes":"transition"]);try Self.append(&graph,on:edges,value:v)
                    case "links":let map=try Self.ref(graph.field("links")),key=try graph.intern("A"),value=try Self.input(&graph,["url":"https://example.com/alpha","tooltip":"Alpha link"]);guard case .map(var entries,let props)=graph.nodes[Int(map)]else{throw PeleError(code:"test",message:"Map expected")};entries.append(.init(key:.string(key),value:value));graph.nodes[Int(map)] = .map(entries:entries,properties:props)
                    case "direction":try Self.setText(&graph,"dir","TB",on:group);let v=try Self.input(&graph,"TB");calls.append((14,[v]))
                    case "helper":let name=try Self.input(&graph,"Added"),detail=try Self.input(&graph,"Saved helper");calls.append((5,[name]));calls.append((9,[name,detail]))
                    case "title-accessibility":try Self.setText(&graph,"title","Changed title");try Self.setText(&graph,"accTitle","Changed access");try Self.setText(&graph,"accDescr","Changed description")
                    case "throw":try Self.setText(&graph,"title","Before throw");try Self.setText(&graph,"label","Saved Alpha",on:alpha)
                    case "nested":try Self.setText(&graph,"title","Nested title")
                    case "baseline":break
                    default:throw PeleError(code:"test",message:"Unknown State mode")
                    }
                    if mode != "baseline" { try document.apply(graph);for(method,args)in calls{try document.invoke(method,args)} }
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
            XCTAssertEqual(try Self.ref(after.graph.field("graph"))==originalGraph,row["graphCurrent"] as? Bool)
            XCTAssertEqual(try Self.ref(after.graph.field("edges",on:originalGraph))==edges,row["edgesCurrent"] as? Bool)
            XCTAssertEqual(try Self.ref(after.graph.field("description",on:alpha))==description,row["descriptionCurrent"] as? Bool)
            XCTAssertEqual(try Self.ref(after.graph.field("cssStyles",on:beta))==styles,row["stylesCurrent"] as? Bool)
            for(name,id)in[("originalGraph",originalGraph),("originalAlpha",alpha),("originalBeta",beta),("originalEdge",edge)]{XCTAssertTrue(NSDictionary(dictionary:try XCTUnwrap(Self.value(after.graph,.ref(id)) as? [String:Any])).isEqual(to:try XCTUnwrap(row[name] as? [String:Any])),mode+name)}
            for(name,id)in[("originalNodes",nodes),("originalEdges",edges),("originalDescription",description),("originalStyles",styles)]{XCTAssertTrue(NSArray(array:try XCTUnwrap(Self.value(after.graph,.ref(id)) as? [Any])).isEqual(to:try XCTUnwrap(row[name] as? [Any])),mode+name)}
            if mode == "nested" { try compare(XCTUnwrap(observed.nested), XCTUnwrap(row["nested"] as? [String: Any])); XCTAssertGreaterThan(document.revision, try XCTUnwrap(observed.nestedRevision)) }
            let recovery = StateCallbackState(); var plain = pure()
            plain.textWidth = { request in MainActor.assumeIsolated { recovery.trace.append([request.text, String(format: "%016llx", request.size.bitPattern), Int(request.style)]); return Double(request.text.utf16.count) * request.size * 0.6 } }
            try compare(document.render(plain, responsive: responsive), XCTUnwrap(row["recovery"] as? [String: Any]))
            XCTAssertTrue(NSArray(array: recovery.trace).isEqual(to: try XCTUnwrap(row["recoveryTrace"] as? [[Any]])), mode + " recovery trace")
        }
    }
}
