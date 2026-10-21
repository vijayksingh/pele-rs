import Foundation
import XCTest
import Pele

private final class TreemapCallbackFailure: Error {}
@MainActor private enum TreemapCallbackDocument {
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
@MainActor private final class TreemapCallbackState {
    var fired = false
    var trace: [[Any]] = []
    var nested: FullRenderedDiagram?
    var nestedRevision: UInt64?
    let failure = TreemapCallbackFailure()
}

@MainActor final class NativeTreemapCallbackTests: XCTestCase {
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
    private static func sameRef(_ a:Pele.PublicGraph.Atom?,_ b:Pele.PublicGraph.Atom?)->Bool {
        if case (.some(.ref(let x)),.some(.ref(let y))) = (a,b) { return x == y }; return false
    }
    private static func ref(_ atom: Pele.PublicGraph.Atom?) throws -> UInt32 {
        guard case .ref(let id) = atom else { throw PeleError(code: "test", message: "Treemap reference missing") }; return id
    }
    private static func setText(_ graph: inout Pele.PublicGraph, _ key: String, _ value: String, on node: UInt32? = nil) throws {
        let id = try graph.intern(value); try graph.setField(key, value: .string(id), on: node)
    }
    private static func append(_ graph: inout Pele.PublicGraph, on node:UInt32, value:Pele.PublicGraph.Atom) throws {
        guard case .array(let length,_) = graph.nodes[Int(node)] else {throw PeleError(code:"test",message:"Treemap array missing")}
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
        } else { throw PeleError(code: "test", message: "Unexpected Treemap fixture input") }
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
        default: throw PeleError(code: "test", message: "Unexpected Treemap fixture atom")
        }
    }
    private static func state(_ graph: Pele.PublicGraph) throws -> [String: Any] {
        var result: [String: Any] = [:]
        for key in ["title", "accTitle", "accDescr", "roots"] { result[key] = try value(graph, graph.field(key) ?? .undefined) }
        return result
    }
    func testTreemapCapturedArraysLateFieldsNestedCallbacksAndRecoveryMatchAllOracleRows() throws {
        var path = URL(fileURLWithPath: #filePath); for _ in 0..<5 { path.deleteLastPathComponent() }
        let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-treemap-live-render.json"))) as? [String: Any])
        XCTAssertEqual(fixture["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        let source = try XCTUnwrap(fixture["source"] as? String), rows = try XCTUnwrap(fixture["rows"] as? [[String: Any]])
        XCTAssertEqual(rows.count, 56)
        for row in rows {
            let raw = try XCTUnwrap(row["raw"] as? Bool), responsive = try XCTUnwrap(row["responsive"] as? Bool), mode = try XCTUnwrap(row["mode"] as? String)
            let engine = Pele.Engine()
            let document: TreemapCallbackDocument = try raw ? .prepared(engine.parsePrepared("treemap", source: source)) : .normal(engine.parseMutable(source))
            defer { try? document.close() }
            var setup=try document.snapshot().graph
            let roots=try Self.ref(setup.field("roots")), group=try Self.ref(setup.field("0",on:roots)), children=try Self.ref(setup.field("children",on:group)), original=try Self.ref(setup.field("0",on:children)), second=try Self.ref(setup.field("1",on:children)), styles=try Self.input(&setup,["fill:red","color:blue"])
            try Self.setText(&setup,"title","Initial");try setup.setField("cssCompiledStyles",value:styles,on:original);try setup.setField("cssCompiledStyles",value:styles,on:second);try document.apply(setup)
            let observed=TreemapCallbackState()
            var resources = pure()
            resources.textWidth = { request in try MainActor.assumeIsolated {
                observed.trace.append([request.text, String(format: "%016llx", request.size.bitPattern), Int(request.style)])
                if !observed.fired && (!mode.hasPrefix("late-") || request.text == "Alpha") {
                    observed.fired = true
                    XCTAssertThrowsError(try document.close())
                    var graph = try document.snapshot().graph
                    let currentRoots=try Self.ref(graph.field("roots")), currentGroup=try Self.ref(graph.field("0",on:currentRoots)), currentChildren=try Self.ref(graph.field("children",on:currentGroup)), first=try Self.ref(graph.field("0",on:currentChildren)), second=try Self.ref(graph.field("1",on:currentChildren))
                    switch mode {
                    case "name": try Self.setText(&graph,"name","Changed group",on:currentGroup);try Self.setText(&graph,"name","Changed Alpha",on:first)
                    case "next-name":try Self.setText(&graph,"name","Changed Beta",on:second)
                    case "values":try graph.setField("value",value:.number(40),on:first);try graph.setField("value",value:.number(20),on:second)
                    case "children-replace":let v=try Self.input(&graph,[["name":"Replacement","value":9]]);try graph.setField("children",value:v,on:currentGroup)
                    case "roots-replace":let v=try Self.input(&graph,[["name":"Replacement root","value":9]]);try graph.setField("roots",value:v)
                    case "append":let v=try Self.input(&graph,["name":"Appended","value":8]);try Self.append(&graph,on:currentChildren,value:v)
                    case "styles":let v=try Self.input(&graph,["fill:green","color:white"]);try graph.setField("cssCompiledStyles",value:v,on:first);try Self.setText(&graph,"classSelector","changed",on:first)
                    case "late-class":try Self.setText(&graph,"classSelector","late",on:first);try Self.setText(&graph,"name","After captured name",on:first)
                    case "late-styles":let array=try Self.ref(graph.field("cssCompiledStyles",on:first));try Self.setText(&graph,"0","fill:green",on:array)
                    case "shape":let properties=graph.nodes[Int(currentGroup)].properties.filter{!(graph.strings[Int($0.key)].stringIfWellFormed == "children")};graph.nodes[Int(currentGroup)] = .object(properties:properties);try graph.setField("value",value:.number(8),on:currentGroup)
                    case "title-accessibility":try Self.setText(&graph,"title","Changed title");try Self.setText(&graph,"accTitle","Changed access");try Self.setText(&graph,"accDescr","Changed description")
                    case "throw":try Self.setText(&graph,"title","Before throw");try Self.setText(&graph,"name","Saved Alpha",on:first)
                    case "nested":try Self.setText(&graph,"title","Nested title")
                    case "baseline":break
                    default:throw PeleError(code:"test",message:"Unknown Treemap mode")
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
            let currentRoots=try Self.ref(after.graph.field("roots")), currentGroup=try Self.ref(after.graph.field("0",on:currentRoots)), currentChildren=try after.graph.field("children",on:currentGroup) ?? .undefined
            let currentFirst:Pele.PublicGraph.Atom
            if case .ref(let array)=currentChildren{currentFirst=try after.graph.field("0",on:array) ?? .undefined}else{currentFirst = .undefined}
            XCTAssertEqual(currentRoots==roots,row["rootsCurrent"] as? Bool);XCTAssertEqual(Self.sameRef(currentChildren,.ref(children)),row["childrenCurrent"] as? Bool);XCTAssertEqual(Self.sameRef(currentFirst,.ref(original)),row["firstCurrent"] as? Bool);XCTAssertEqual(try Self.sameRef(after.graph.field("cssCompiledStyles",on:original),styles),row["stylesCurrent"] as? Bool)
            for(name,value)in[("originalRoots",Pele.PublicGraph.Atom.ref(roots)),("originalChildren",.ref(children)),("originalStyles",styles)]{XCTAssertTrue(NSArray(array:try XCTUnwrap(Self.value(after.graph,value) as? [Any])).isEqual(to:try XCTUnwrap(row[name] as? [Any])),mode+name)}
            XCTAssertTrue(NSDictionary(dictionary:try XCTUnwrap(Self.value(after.graph,.ref(original)) as? [String:Any])).isEqual(to:try XCTUnwrap(row["originalFirst"] as? [String:Any])),mode+" original first")
            if mode == "nested" { try compare(XCTUnwrap(observed.nested), XCTUnwrap(row["nested"] as? [String: Any])); XCTAssertGreaterThan(document.revision, try XCTUnwrap(observed.nestedRevision)) }
            let recovery = TreemapCallbackState(); var plain = pure()
            plain.textWidth = { request in MainActor.assumeIsolated { recovery.trace.append([request.text, String(format: "%016llx", request.size.bitPattern), Int(request.style)]); return Double(request.text.utf16.count) * request.size * 0.6 } }
            try compare(document.render(plain, responsive: responsive), XCTUnwrap(row["recovery"] as? [String: Any]))
            XCTAssertTrue(NSArray(array: recovery.trace).isEqual(to: try XCTUnwrap(row["recoveryTrace"] as? [[Any]])), mode + " recovery trace")
        }
    }
}
