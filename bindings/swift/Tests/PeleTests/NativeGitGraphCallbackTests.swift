import Foundation
import XCTest
import Pele

private final class GitGraphCallbackFailure: Error {}
@MainActor private enum GitGraphCallbackDocument {
    case normal(Pele.MutableDocument), prepared(Pele.PreparedDocument)
    var revision: UInt64 { switch self { case .normal(let d): d.revision; case .prepared(let d): d.revision } }
    func snapshot() throws -> Pele.PublicSnapshot { switch self { case .normal(let d): try d.snapshot(); case .prepared(let d): try d.snapshot() } }
    func apply(_ graph: Pele.PublicGraph) throws {
        switch self { case .normal(let d): _ = try d.apply(graph, expectedRevision: d.revision); case .prepared(let d): _ = try d.apply(graph, expectedRevision: d.revision) }
    }
    func invoke(method: UInt32, receiver: Pele.PublicGraph.Atom, arguments: [Pele.PublicGraph.Atom]) throws {
        switch self { case .normal(let d): _ = try d.invoke(method: method, receiver: receiver, arguments: arguments); case .prepared(let d): _ = try d.invoke(method: method, receiver: receiver, arguments: arguments) }
    }
    func render(_ resources: Pele.HostResources, responsive: Bool = false) throws -> FullRenderedDiagram {
        var options = Pele.Engine.Options(); options.maxWidth = responsive ? 220 : nil
        switch self { case .normal(let d): return try d.render(options: options, resources: resources); case .prepared(let d): return try d.render(options: options, resources: resources) }
    }
    func close() throws { switch self { case .normal(let d): try d.close(); case .prepared(let d): try d.close() } }
}
@MainActor private final class GitGraphCallbackState {
    var fired = false
    var trace: [[Any]] = []
    var nested: FullRenderedDiagram?
    var nestedRevision: UInt64?
    let failure = GitGraphCallbackFailure()
}

@MainActor final class NativeGitGraphCallbackTests: XCTestCase {
    private static func bits(_ n: Double) -> String { String(format: "%016llx", n.bitPattern) }
    private static func ref(_ value: Pele.PublicGraph.Atom?) throws -> UInt32 {
        guard case .ref(let id) = value else { throw PeleError(code: "test", message: "Expected GitGraph reference") }; return id
    }
    private static func lookup(_ graph:Pele.PublicGraph,_ map:UInt32,_ key:String)throws->Pele.PublicGraph.Atom?{guard case .map(let entries,_)=graph.nodes[Int(map)]else{throw GitGraphCallbackFailure()};return entries.first{entry in if case .string(let i)=entry.key{return graph.strings[Int(i)].stringIfWellFormed==key};return false}?.value}
    private static func first(_ graph:Pele.PublicGraph)throws->(nodes:UInt32,root:UInt32){let m=try ref(graph.field("commits"));return(m,try ref(lookup(graph,m,"a")))}
    private static func setText(_ graph: inout Pele.PublicGraph, _ key: String, _ value: String, on node: UInt32? = nil) throws {
        let id = try graph.intern(value); try graph.setField(key, value: .string(id), on: node)
    }
    private static func array(_ graph: inout Pele.PublicGraph, _ values: [Pele.PublicGraph.Atom]) throws -> UInt32 {
        let id = UInt32(graph.nodes.count); graph.nodes.append(.array(length: 0, properties: []))
        for (i, value) in values.enumerated() { try graph.setArrayElement(UInt32(i), value: value, on: id) }; return id
    }
    private static func state(_ graph:Pele.PublicGraph)throws->[String:Any]{
      func scalar(_ atom:Pele.PublicGraph.Atom?)throws->Any{switch atom{case .string(let i):return try XCTUnwrap(graph.strings[Int(i)].stringIfWellFormed);case .number(let n):return bits(n);case .bool(let b):return b;default:return NSNull()}}
      func strings(_ atom:Pele.PublicGraph.Atom?)throws->[Any]{let a=try ref(atom);guard case .array(let n,_)=graph.nodes[Int(a)]else{throw GitGraphCallbackFailure()};return try (0..<n).map{try scalar(graph.field(String($0),on:a))}}
      let map=try ref(graph.field("commits"));guard case .map(let entries,_)=graph.nodes[Int(map)]else{throw GitGraphCallbackFailure()}
      let commits=try entries.map{entry in let c=try ref(entry.value);var row:[String:Any]=["key":try scalar(entry.key)];for k in ["id","message","seq","type","customType","customId","branch"]{row[k]=try scalar(graph.field(k,on:c))};for k in ["tags","parents"]{row[k]=try strings(graph.field(k,on:c))};return row}
      return ["title":try scalar(graph.field("title")),"direction":try scalar(graph.field("direction")),"lanes":try strings(graph.field("lanes")),"commits":commits]
    }
    private static func resources(_ state: GitGraphCallbackState? = nil) -> Pele.HostResources {
        var host = Pele.HostResources()
        host.textWidth = { request in MainActor.assumeIsolated {
            state?.trace.append(["width", request.text, bits(request.size), Int(request.style)])
            return Double(request.text.utf16.count) * request.size * 0.6
        }}
        host.icon = { name in MainActor.assumeIsolated {
            state?.trace.append(["icon", name]); return Pele.TrustedSVG(trustedMarkup: "<path d=\"M0 0h1\"/>")
        }}
        return host
    }
    private func compare(_ actual: FullRenderedDiagram, _ expected: [String: Any]) throws {
        XCTAssertEqual(actual.svg, try XCTUnwrap(expected["svg"] as? String)); XCTAssertEqual(Self.bits(actual.width), expected["width"] as? String); XCTAssertEqual(Self.bits(actual.height), expected["height"] as? String); XCTAssertTrue(actual.links.isEmpty)
    }
    private static func mutate(_ document: GitGraphCallbackDocument, _ state: GitGraphCallbackState, _ originalMode: String) throws {
        state.fired = true; XCTAssertThrowsError(try document.close())
        let mode=originalMode
        var graph=try document.snapshot().graph;let n=try first(graph),later=try ref(lookup(graph,n.nodes,"b")),tags=try ref(graph.field("tags",on:n.root))
        switch mode{
        case "baseline":break
        case "title":try setText(&graph,"title","Changed title")
        case "lanes":let lanes=try ref(graph.field("lanes"));let values=try (0..<2).reversed().map{try XCTUnwrap(graph.field(String($0),on:lanes))};let a=try array(&graph,values);try graph.setField("lanes",value:.ref(a))
        case "current-tag":let a=try graph.intern("Changed tag"),b=try graph.intern("late-tag");try graph.setArrayElement(0,value:.string(a),on:tags);try graph.setArrayElement(1,value:.string(b),on:tags)
        case "current-id":try setText(&graph,"id","renamed-a",on:n.root)
        case "later-id":try setText(&graph,"id","renamed-b",on:later)
        case "parents":let a=try array(&graph,[]);try graph.setField("parents",value:.ref(a),on:later)
        case "replace-map":let a=UInt32(graph.nodes.count);graph.nodes.append(.map(entries:[],properties:[]));try graph.setField("commits",value:.ref(a))
        case "type":try graph.setField("type",value:.number(2),on:n.root);try graph.setField("customType",value:.number(1),on:later)
        case "throw-object","throw-undefined":try setText(&graph,"title","Before failure");try setText(&graph,"message","Persisted message",on:n.root)
        case "nested":try setText(&graph,"title","Nested title")
        default:throw GitGraphCallbackFailure()
        }
        if mode != "baseline" {try document.apply(graph)}
        // Swift has Error values, not JS undefined throws. Both fixture lanes
        // retain the actual original Swift error through C15 cleanup.
        if mode.hasPrefix("throw-") { throw state.failure }
        if mode == "nested" {
            state.nested = try document.render(resources()); state.nestedRevision = document.revision
            var latest = try document.snapshot().graph; try setText(&latest, "title", "After nested"); try document.apply(latest)
        }
    }
    func testGitGraphCapturedCommitTagsLateIdentityErrorsMatch48OracleRows() throws {
        var path = URL(fileURLWithPath: #filePath); for _ in 0..<5 { path.deleteLastPathComponent() }
        let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-gitgraph-live-render.json"))) as? [String: Any])
        XCTAssertEqual(fixture["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        let source = try XCTUnwrap(fixture["source"] as? String), rows = try XCTUnwrap(fixture["rows"] as? [[String: Any]]); XCTAssertEqual(rows.count, 48)
        for row in rows {
            let raw = try XCTUnwrap(row["raw"] as? Bool), responsive = try XCTUnwrap(row["responsive"] as? Bool), mode = try XCTUnwrap(row["mode"] as? String), engine = Pele.Engine()
            let document: GitGraphCallbackDocument = try raw ? .prepared(engine.parsePrepared("gitGraph", source: source)) : .normal(engine.parseMutable(source)); defer { try? document.close() }
            var initial=try document.snapshot().graph;try Self.setText(&initial,"title","Initial title");try document.apply(initial)
            let oldGraph=try document.snapshot().graph,original=try Self.first(oldGraph),oldLanes=try Self.ref(oldGraph.field("lanes")),oldTags=try Self.ref(oldGraph.field("tags",on:original.root)),observed=GitGraphCallbackState();var host=Self.resources()
            host.textWidth = { request in try MainActor.assumeIsolated {
                observed.trace.append(["width", request.text, Self.bits(request.size), Int(request.style)])
                let selected=["current-tag","replace-map","type"].contains(mode) ? request.text == "v1" : (["current-id","later-id","parents"].contains(mode) ? request.text == "a" : true);if !observed.fired && selected { try Self.mutate(document, observed, mode) }
                return Double(request.text.utf16.count) * request.size * 0.6
            }}
            let first = try XCTUnwrap(row["first"] as? [String: Any])
            if mode.contains("throw-") { XCTAssertEqual(first["original"] as? Bool, true); XCTAssertThrowsError(try document.render(host, responsive: responsive)) { XCTAssertTrue(($0 as AnyObject) === observed.failure) } }
            else { try compare(document.render(host, responsive: responsive), first) }
            XCTAssertTrue(observed.fired); XCTAssertTrue(NSArray(array: observed.trace).isEqual(to: try XCTUnwrap(row["trace"] as? [[Any]])), "\(mode)/\(raw)/\(responsive) trace")
            let after = try document.snapshot(); XCTAssertEqual(after.revision, document.revision)
            XCTAssertTrue(NSDictionary(dictionary: try Self.state(after.graph)).isEqual(to: try XCTUnwrap(row["after"] as? [String: Any])), mode + " state")
            let map=try Self.ref(after.graph.field("commits")),identity=try XCTUnwrap(row["identity"] as? [String:Bool]);XCTAssertEqual(map==original.nodes,identity["map"]);let found=try Self.lookup(after.graph,map,"a");if case .ref(let i)=found{XCTAssertEqual(i==original.root,identity["first"])}else{XCTAssertEqual(identity["first"],false)};XCTAssertEqual(try Self.ref(after.graph.field("tags",on:original.root))==oldTags,identity["tags"]);XCTAssertEqual(try Self.ref(after.graph.field("lanes"))==oldLanes,identity["lanes"])
            if mode == "nested" || mode == "icon-nested" { try compare(XCTUnwrap(observed.nested), XCTUnwrap(row["nested"] as? [String: Any])); XCTAssertGreaterThan(document.revision, try XCTUnwrap(observed.nestedRevision)) }
            let recovery = GitGraphCallbackState(); try compare(document.render(Self.resources(recovery), responsive: responsive), XCTUnwrap(row["recovery"] as? [String: Any]))
            XCTAssertTrue(NSArray(array: recovery.trace).isEqual(to: try XCTUnwrap(row["recoveryTrace"] as? [[Any]])), mode + " recovery trace")
        }
    }
}
