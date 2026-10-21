import Foundation
import XCTest
import Pele

private final class GitGraphIterableFailure: Error {}
@MainActor private enum GitGraphIterableDocument {
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
@MainActor private final class GitGraphIterableState {
    var fired = false
    var trace: [[Any]] = []
    var nested: FullRenderedDiagram?
    var nestedRevision: UInt64?
    let failure = GitGraphIterableFailure()
}

@MainActor final class NativeGitGraphIterableTests: XCTestCase {
    private static func bits(_ n: Double) -> String { String(format: "%016llx", n.bitPattern) }
    private static func ref(_ value: Pele.PublicGraph.Atom?) throws -> UInt32 {
        guard case .ref(let id) = value else { throw PeleError(code: "test", message: "Expected GitGraph reference") }; return id
    }
    private static func lookup(_ graph:Pele.PublicGraph,_ map:UInt32,_ key:String)throws->Pele.PublicGraph.Atom?{guard case .map(let entries,_)=graph.nodes[Int(map)]else{throw GitGraphIterableFailure()};return entries.first{entry in if case .string(let i)=entry.key{return graph.strings[Int(i)].stringIfWellFormed==key};return false}?.value}
    private static func first(_ graph:Pele.PublicGraph)throws->(nodes:UInt32,root:UInt32){let m=try ref(graph.field("commits"));return(m,try ref(lookup(graph,m,"a")))}
    private static func setText(_ graph: inout Pele.PublicGraph, _ key: String, _ value: String, on node: UInt32? = nil) throws {
        let id = try graph.intern(value); try graph.setField(key, value: .string(id), on: node)
    }
    private static func array(_ graph: inout Pele.PublicGraph, _ values: [Pele.PublicGraph.Atom]) throws -> UInt32 {
        let id = UInt32(graph.nodes.count); graph.nodes.append(.array(length: 0, properties: []))
        for (i, value) in values.enumerated() { try graph.setArrayElement(UInt32(i), value: value, on: id) }; return id
    }
    private static func setNode(_ graph:inout Pele.PublicGraph,_ strings:[String])throws->Pele.PublicGraph.Atom{
        let values=try strings.map{Pele.PublicGraph.Atom.string(try graph.intern($0))}
        let id=UInt32(graph.nodes.count);graph.nodes.append(.set(values:values,properties:[]));return .ref(id)
    }
    private static func appendSet(_ graph:inout Pele.PublicGraph,_ atom:Pele.PublicGraph.Atom,_ text:String)throws{
        let id=try ref(atom),value=try graph.intern(text)
        guard case .set(var values,let props)=graph.nodes[Int(id)]else{throw GitGraphIterableFailure()}
        values.append(.string(value));graph.nodes[Int(id)] = .set(values:values,properties:props)
    }
    private static func same(_ graph:Pele.PublicGraph,_ lhs:Pele.PublicGraph.Atom,_ rhs:Pele.PublicGraph.Atom)->Bool{
        switch(lhs,rhs){case(.ref(let a),.ref(let b)):return a==b;case(.string(let a),.string(let b)):return graph.strings[Int(a)]==graph.strings[Int(b)];default:return false}
    }
    private static func iteratedStrings(_ graph:Pele.PublicGraph,_ atom:Pele.PublicGraph.Atom)throws->[String]{
        if case .string(let i)=atom{return try XCTUnwrap(graph.strings[Int(i)].stringIfWellFormed).unicodeScalars.map{String($0)}}
        let id=try ref(atom),values:[Pele.PublicGraph.Atom]
        switch graph.nodes[Int(id)]{
        case .set(let items,_):values=items
        case .array(let n,_):values=try (0..<n).map{try XCTUnwrap(graph.field(String($0),on:id))}
        default:throw GitGraphIterableFailure()
        }
        return try values.map{guard case .string(let i)=$0 else{throw GitGraphIterableFailure()};return try XCTUnwrap(graph.strings[Int(i)].stringIfWellFormed)}
    }
    private static func state(_ graph:Pele.PublicGraph)throws->[String:Any]{
      func scalar(_ atom:Pele.PublicGraph.Atom?)throws->Any{switch atom{case .string(let i):return try XCTUnwrap(graph.strings[Int(i)].stringIfWellFormed);case .number(let n):return bits(n);case .bool(let b):return b;default:return NSNull()}}
      func strings(_ atom:Pele.PublicGraph.Atom?)throws->[Any]{try iteratedStrings(graph,try XCTUnwrap(atom))}
      let map=try ref(graph.field("commits"));guard case .map(let entries,_)=graph.nodes[Int(map)]else{throw GitGraphIterableFailure()}
      let commits=try entries.map{entry in let c=try ref(entry.value);var row:[String:Any]=["key":try scalar(entry.key)];for k in ["id","message","seq","type","customType","customId","branch"]{row[k]=try scalar(graph.field(k,on:c))};for k in ["tags","parents"]{row[k]=try strings(graph.field(k,on:c))};return row}
      return ["title":try scalar(graph.field("title")),"direction":try scalar(graph.field("direction")),"lanes":try strings(graph.field("lanes")),"commits":commits]
    }
    private static func resources(_ state: GitGraphIterableState? = nil) -> Pele.HostResources {
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
    private static func mutate(_ document: GitGraphIterableDocument, _ state: GitGraphIterableState, _ originalMode: String) throws {
        state.fired = true; XCTAssertThrowsError(try document.close())
        let mode=originalMode
        var graph=try document.snapshot().graph;let n=try first(graph),later=try ref(lookup(graph,n.nodes,"b")),tags=try XCTUnwrap(graph.field("tags",on:n.root))
        switch mode{
        case "initial-set","initial-string","initial-astral","initial-empty":break
        case "later-set":let value=try setNode(&graph,["Later","Tail"]);try graph.setField("tags",value:value,on:later)
        case "later-string":try setText(&graph,"tags","😀Z",on:later)
        case "set-append":try appendSet(&graph,tags,"late-tag")
        case "set-delete-future":
            let node=try ref(tags);guard case .set(let values,let props)=graph.nodes[Int(node)]else{throw GitGraphIterableFailure()}
            let filtered=values.filter{if case .string(let id)=$0{return graph.strings[Int(id)].stringIfWellFormed != "v2"};return true}
            graph.nodes[Int(node)] = .set(values:filtered,properties:props)
        case "set-replace":let value=try setNode(&graph,["Replacement"]);try graph.setField("tags",value:value,on:n.root)
        case "string-replace":try setText(&graph,"tags","Replacement",on:n.root)
        case "throw-object","throw-undefined":try setText(&graph,"title","Before failure");try appendSet(&graph,tags,"Persisted")
        case "nested":try appendSet(&graph,tags,"Nested tag");try setText(&graph,"title","Nested title")
        default:throw GitGraphIterableFailure()
        }
        if !mode.hasPrefix("initial-") {try document.apply(graph)}
        // Swift has Error values, not JS undefined throws. Both fixture lanes
        // retain the actual original Swift error through C15 cleanup.
        if mode.hasPrefix("throw-") { throw state.failure }
        if mode == "nested" {
            state.nested = try document.render(resources()); state.nestedRevision = document.revision
            var latest = try document.snapshot().graph; try setText(&latest, "title", "After nested"); try document.apply(latest)
        }
    }
    func testGitGraphTagIteratorsMatch52OracleRows() throws {
        var path = URL(fileURLWithPath: #filePath); for _ in 0..<5 { path.deleteLastPathComponent() }
        let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-gitgraph-tag-iterables.json"))) as? [String: Any])
        XCTAssertEqual(fixture["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        let source = try XCTUnwrap(fixture["source"] as? String), rows = try XCTUnwrap(fixture["rows"] as? [[String: Any]]); XCTAssertEqual(rows.count, 52)
        for row in rows {
            let raw = try XCTUnwrap(row["raw"] as? Bool), responsive = try XCTUnwrap(row["responsive"] as? Bool), mode = try XCTUnwrap(row["mode"] as? String), engine = Pele.Engine()
            let document: GitGraphIterableDocument = try raw ? .prepared(engine.parsePrepared("gitGraph", source: source)) : .normal(engine.parseMutable(source)); defer { try? document.close() }
            var initial=try document.snapshot().graph;try Self.setText(&initial,"title","Initial title")
            let initialFirst=try Self.first(initial).root
            let tags:Pele.PublicGraph.Atom
            switch mode{
            case "initial-string","string-replace":tags = .string(try initial.intern("XY"))
            case "initial-astral":tags = .string(try initial.intern("😀Z"))
            case "initial-empty":tags = try Self.setNode(&initial,[])
            case let value where value.hasPrefix("later-"):tags = try XCTUnwrap(initial.field("tags",on:initialFirst))
            default:tags = try Self.setNode(&initial,["v1","v2","v3"])
            }
            try initial.setField("tags",value:tags,on:initialFirst);try document.apply(initial)
            let oldGraph=try document.snapshot().graph,original=try Self.first(oldGraph),oldLanes=try Self.ref(oldGraph.field("lanes")),oldTags=try XCTUnwrap(oldGraph.field("tags",on:original.root)),observed=GitGraphIterableState();var host=Self.resources()
            host.textWidth = { request in try MainActor.assumeIsolated {
                observed.trace.append(["width", request.text, Self.bits(request.size), Int(request.style)])
                let selected:Bool
                switch mode { case "initial-empty":selected=true;case "initial-string","string-replace":selected=request.text=="X";case "initial-astral":selected=request.text=="😀";default:selected=request.text=="v1" }
                if !observed.fired && selected {try Self.mutate(document,observed,mode)}
                return Double(request.text.utf16.count) * request.size * 0.6
            }}
            let first = try XCTUnwrap(row["first"] as? [String: Any])
            if mode.contains("throw-") { XCTAssertEqual(first["original"] as? Bool, true); XCTAssertThrowsError(try document.render(host, responsive: responsive)) { XCTAssertTrue(($0 as AnyObject) === observed.failure) } }
            else { try compare(document.render(host, responsive: responsive), first) }
            XCTAssertTrue(observed.fired); XCTAssertTrue(NSArray(array: observed.trace).isEqual(to: try XCTUnwrap(row["trace"] as? [[Any]])), "\(mode)/\(raw)/\(responsive) trace")
            let after = try document.snapshot(); XCTAssertEqual(after.revision, document.revision)
            XCTAssertTrue(NSDictionary(dictionary: try Self.state(after.graph)).isEqual(to: try XCTUnwrap(row["after"] as? [String: Any])), mode + " state")
            let map=try Self.ref(after.graph.field("commits")),identity=try XCTUnwrap(row["identity"] as? [String:Bool]);XCTAssertEqual(map==original.nodes,identity["map"]);let found=try Self.lookup(after.graph,map,"a");if case .ref(let i)=found{XCTAssertEqual(i==original.root,identity["first"])}else{XCTAssertEqual(identity["first"],false)};XCTAssertEqual(Self.same(after.graph,try XCTUnwrap(after.graph.field("tags",on:original.root)),oldTags),identity["tags"]);XCTAssertEqual(try Self.ref(after.graph.field("lanes"))==oldLanes,identity["lanes"])
            XCTAssertEqual(try Self.iteratedStrings(after.graph,oldTags),try XCTUnwrap(row["originalTags"] as? [String]),mode + " retained tags")
            if mode == "nested" || mode == "icon-nested" { try compare(XCTUnwrap(observed.nested), XCTUnwrap(row["nested"] as? [String: Any])); XCTAssertGreaterThan(document.revision, try XCTUnwrap(observed.nestedRevision)) }
            let recovery = GitGraphIterableState(); try compare(document.render(Self.resources(recovery), responsive: responsive), XCTUnwrap(row["recovery"] as? [String: Any]))
            XCTAssertTrue(NSArray(array: recovery.trace).isEqual(to: try XCTUnwrap(row["recoveryTrace"] as? [[Any]])), mode + " recovery trace")
        }
    }
}
