import Foundation
import XCTest
import Pele

private final class C4CallbackFailure: Error {}
@MainActor private enum C4CallbackDocument {
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
        var options = Pele.Engine.Options(); options.maxWidth = responsive ? 300 : nil
        switch self { case .normal(let d): return try d.render(options: options, resources: resources); case .prepared(let d): return try d.render(options: options, resources: resources) }
    }
    func close() throws { switch self { case .normal(let d): try d.close(); case .prepared(let d): try d.close() } }
}
@MainActor private final class C4CallbackState {
    var fired = false
    var trace: [[Any]] = []
    var nested: FullRenderedDiagram?
    var nestedRevision: UInt64?
    let failure = C4CallbackFailure()
}

@MainActor final class NativeC4CallbackTests: XCTestCase {
    private static func bits(_ n: Double) -> String { String(format: "%016llx", n.bitPattern) }
    private static func ref(_ value: Pele.PublicGraph.Atom?) throws -> UInt32 {
        guard case .ref(let id) = value else { throw PeleError(code: "test", message: "Expected C4 reference") }; return id
    }
    private static func first(_ graph:Pele.PublicGraph)throws->(nodes:UInt32,root:UInt32){let n=try ref(graph.field("shapes"));return(n,try ref(graph.field("0",on:n)))}
    private static func child(_ graph:Pele.PublicGraph,_ key:String,_ index:String)throws->UInt32{try ref(graph.field(index,on:ref(graph.field(key))))}
    private static func setText(_ graph: inout Pele.PublicGraph, _ key: String, _ value: String, on node: UInt32? = nil) throws {
        let id = try graph.intern(value); try graph.setField(key, value: .string(id), on: node)
    }
    private static func array(_ graph: inout Pele.PublicGraph, _ values: [Pele.PublicGraph.Atom]) throws -> UInt32 {
        let id = UInt32(graph.nodes.count); graph.nodes.append(.array(length: 0, properties: []))
        for (i, value) in values.enumerated() { try graph.setArrayElement(UInt32(i), value: value, on: id) }; return id
    }
    private static func state(_ graph:Pele.PublicGraph)throws->[String:Any]{
      func scalar(_ atom:Pele.PublicGraph.Atom?)throws->Any{switch atom{case .string(let i):return try XCTUnwrap(graph.strings[Int(i)].stringIfWellFormed);case .number(let n):return ["bits":bits(n)];default:return NSNull()}}
      func number(_ key:String)throws->String{guard case .number(let n)=try graph.field(key) else{throw C4CallbackFailure()};return bits(n)}
      func records(_ key:String,_ fields:[String],parent:Bool)throws->[[String:Any]]{let a=try ref(graph.field(key));guard case .array(let n,_)=graph.nodes[Int(a)] else{throw C4CallbackFailure()};return try (0..<n).map{i in let v=try ref(graph.field(String(i),on:a));var row:[String:Any]=[:];for key in fields{row[key]=try scalar(graph.field(key,on:v))};if parent{if case .ref(let p)=try graph.field("parent",on:v){row["parent"]=try scalar(graph.field("alias",on:p))}else{row["parent"]=NSNull()}};return row}}
      return ["title":try scalar(graph.field("title")),"shapeInRow":try number("shapeInRow"),"boundaryInRow":try number("boundaryInRow"),"shapes":try records("shapes",["alias","label","descr","techn","kind","shape","bgColor","fontColor","borderColor"],parent:true),"boundaries":try records("boundaries",["alias","label","type","descr","bgColor","fontColor","borderColor"],parent:true),"rels":try records("rels",["from","to","label","techn","descr","type","lineColor","textColor","offsetX","offsetY"],parent:false)]
    }
    private static func resources(_ state: C4CallbackState? = nil) -> Pele.HostResources {
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
    private static func mutate(_ document: C4CallbackDocument, _ state: C4CallbackState, _ originalMode: String) throws {
        state.fired = true; XCTAssertThrowsError(try document.close())
        let mode=originalMode
        var graph=try document.snapshot().graph;let n=try first(graph),later=try child(graph,"shapes","1"),boundary=try child(graph,"boundaries","1"),rel=try child(graph,"rels","0")
        switch mode{
        case "baseline","style":break
        case "title":try setText(&graph,"title","Changed title")
        case "boundary-label":try setText(&graph,"label","Changed Bank",on:boundary);try setText(&graph,"descr","Late boundary description",on:boundary)
        case "current-shape":try setText(&graph,"label","Changed customer",on:n.root);try setText(&graph,"descr","Late current description",on:n.root);try setText(&graph,"kind","System",on:n.root)
        case "later-shape":try setText(&graph,"label","Changed Banking",on:later);try setText(&graph,"techn","Late technology",on:later)
        case "relation-label":try setText(&graph,"label","Changed relation",on:rel);try setText(&graph,"techn","Late protocol",on:rel);try setText(&graph,"descr","Late relation description",on:rel)
        case "relation-offset":try graph.setField("offsetX",value:.number(13),on:rel);try graph.setField("offsetY",value:.number(17),on:rel);try setText(&graph,"textColor","#234567",on:rel)
        case "replace-shapes":let values=try (0..<3).reversed().map{try XCTUnwrap(graph.field(String($0),on:n.nodes))};let a=try array(&graph,values);try graph.setField("shapes",value:.ref(a))
        case "throw-object","throw-undefined":try setText(&graph,"title","Before failure");try setText(&graph,"label","Persisted customer",on:n.root)
        case "nested":try setText(&graph,"title","Nested title")
        default:throw C4CallbackFailure()
        }
        if mode != "baseline" && mode != "style" {try document.apply(graph)}
        if mode == "style" {let values=try ["user","#123456","#654321","#abcdef"].map{Pele.PublicGraph.Atom.string(try graph.intern($0))};let a=try array(&graph,values),kind=try graph.intern("UpdateElementStyle");try document.apply(graph);try document.invoke(method:16,receiver:graph.root,arguments:[.string(kind),.ref(a)])}
        // Swift has Error values, not JS undefined throws. Both fixture lanes
        // retain the actual original Swift error through C15 cleanup.
        if mode.hasPrefix("throw-") { throw state.failure }
        if mode == "nested" {
            state.nested = try document.render(resources()); state.nestedRevision = document.revision
            var latest = try document.snapshot().graph; try setText(&latest, "title", "After nested"); try document.apply(latest)
        }
    }
    func testC4BoundaryShapeRelationPhasesHelpersAndErrorsMatch48OracleRows() throws {
        var path = URL(fileURLWithPath: #filePath); for _ in 0..<5 { path.deleteLastPathComponent() }
        let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-c4-live-render.json"))) as? [String: Any])
        XCTAssertEqual(fixture["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        let source = try XCTUnwrap(fixture["source"] as? String), rows = try XCTUnwrap(fixture["rows"] as? [[String: Any]]); XCTAssertEqual(rows.count, 48)
        for row in rows {
            let raw = try XCTUnwrap(row["raw"] as? Bool), responsive = try XCTUnwrap(row["responsive"] as? Bool), mode = try XCTUnwrap(row["mode"] as? String), engine = Pele.Engine()
            let document: C4CallbackDocument = try raw ? .prepared(engine.parsePrepared("c4", source: source)) : .normal(engine.parseMutable(source)); defer { try? document.close() }
            var initial=try document.snapshot().graph;try Self.setText(&initial,"title","Initial title");try document.apply(initial)
            let oldGraph=try document.snapshot().graph,original=try Self.first(oldGraph),oldBoundary=try Self.child(oldGraph,"boundaries","1"),oldRel=try Self.child(oldGraph,"rels","0"),observed=C4CallbackState();var host=Self.resources()
            host.textWidth = { request in try MainActor.assumeIsolated {
                observed.trace.append(["width", request.text, Self.bits(request.size), Int(request.style)])
                let selected=mode.hasPrefix("relation-") ? request.text == "Uses" : (["current-shape","later-shape"].contains(mode) ? request.text == "Customer" : true);if !observed.fired && selected { try Self.mutate(document, observed, mode) }
                return Double(request.text.utf16.count) * request.size * 0.6
            }}
            let first = try XCTUnwrap(row["first"] as? [String: Any])
            if mode.contains("throw-") { XCTAssertEqual(first["original"] as? Bool, true); XCTAssertThrowsError(try document.render(host, responsive: responsive)) { XCTAssertTrue(($0 as AnyObject) === observed.failure) } }
            else { try compare(document.render(host, responsive: responsive), first) }
            XCTAssertTrue(observed.fired); XCTAssertTrue(NSArray(array: observed.trace).isEqual(to: try XCTUnwrap(row["trace"] as? [[Any]])), "\(mode)/\(raw)/\(responsive) trace")
            let after = try document.snapshot(); XCTAssertEqual(after.revision, document.revision)
            XCTAssertTrue(NSDictionary(dictionary: try Self.state(after.graph)).isEqual(to: try XCTUnwrap(row["after"] as? [String: Any])), mode + " state")
            let current=try Self.first(after.graph),identity=try XCTUnwrap(row["identity"] as? [String:Bool]);XCTAssertEqual(current.nodes==original.nodes,identity["shapes"]);XCTAssertEqual(current.root==original.root,identity["first"]);XCTAssertEqual(try Self.child(after.graph,"boundaries","1")==oldBoundary,identity["boundary"]);XCTAssertEqual(try Self.child(after.graph,"rels","0")==oldRel,identity["relation"])
            let map=try Self.ref(after.graph.field("shapeIndex"));guard case .map(let entries,_)=after.graph.nodes[Int(map)] else{throw C4CallbackFailure()};let found=entries.first{entry in if case .string(let i)=entry.key{return after.graph.strings[Int(i)].stringIfWellFormed=="user"};return false};XCTAssertEqual(try Self.ref(found?.value)==original.root,identity["lookup"])
            if mode == "nested" || mode == "icon-nested" { try compare(XCTUnwrap(observed.nested), XCTUnwrap(row["nested"] as? [String: Any])); XCTAssertGreaterThan(document.revision, try XCTUnwrap(observed.nestedRevision)) }
            let recovery = C4CallbackState(); try compare(document.render(Self.resources(recovery), responsive: responsive), XCTUnwrap(row["recovery"] as? [String: Any]))
            XCTAssertTrue(NSArray(array: recovery.trace).isEqual(to: try XCTUnwrap(row["recoveryTrace"] as? [[Any]])), mode + " recovery trace")
        }
    }
}
