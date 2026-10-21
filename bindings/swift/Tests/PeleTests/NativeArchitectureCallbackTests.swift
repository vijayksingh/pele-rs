import Foundation
import XCTest
import Pele

private final class ArchitectureCallbackFailure: Error {}
@MainActor private enum ArchitectureCallbackDocument {
    case normal(Pele.MutableDocument), prepared(Pele.PreparedDocument)
    var revision: UInt64 { switch self { case .normal(let d): d.revision; case .prepared(let d): d.revision } }
    func snapshot() throws -> Pele.PublicSnapshot { switch self { case .normal(let d): try d.snapshot(); case .prepared(let d): try d.snapshot() } }
    func apply(_ graph: Pele.PublicGraph) throws { switch self { case .normal(let d): _ = try d.apply(graph, expectedRevision: d.revision); case .prepared(let d): _ = try d.apply(graph, expectedRevision: d.revision) } }
    func render(_ resources: Pele.HostResources, responsive: Bool) throws -> FullRenderedDiagram {
        var options = Pele.Engine.Options(); options.maxWidth = responsive ? 220 : nil
        switch self { case .normal(let d): return try d.render(options: options, resources: resources); case .prepared(let d): return try d.render(options: options, resources: resources) }
    }
    func close() throws { switch self { case .normal(let d): try d.close(); case .prepared(let d): try d.close() } }
}
@MainActor private final class ArchitectureCallbackState {
    var fired = false
    var trace: [[Any]] = []
    var nested: FullRenderedDiagram?
    var nestedRevision: UInt64?
    let failure = ArchitectureCallbackFailure()
}
@MainActor final class NativeArchitectureCallbackTests: XCTestCase {
    private static func bits(_ n: Double) -> String { String(format: "%016llx", n.bitPattern) }
    private static func ref(_ value: Pele.PublicGraph.Atom?) throws -> UInt32 {
        guard case .ref(let id) = value else { throw PeleError(code: "test", message: "Expected Architecture reference") }; return id
    }
    private static func mapValue(_ graph:Pele.PublicGraph,_ key:String,_ name:String)throws->UInt32{
        let map=try ref(graph.field(key));guard case .map(let entries,_)=graph.nodes[Int(map)] else{throw ArchitectureCallbackFailure()};for entry in entries{if case .string(let id)=entry.key,graph.strings[Int(id)].stringIfWellFormed==name{return try ref(entry.value)}};throw ArchitectureCallbackFailure()
    }
    private static func nodes(_ g:Pele.PublicGraph)throws->(nodes:UInt32,groups:UInt32,later:UInt32,edges:UInt32,first:UInt32,group:UInt32){(try ref(g.field("nodes")),try ref(g.field("groups")),try mapValue(g,"nodes","b"),try ref(g.field("edges")),try mapValue(g,"nodes","a"),try mapValue(g,"groups","g"))}
    private static func setText(_ graph: inout Pele.PublicGraph, _ key: String, _ value: String, on node: UInt32? = nil) throws {
        let id = try graph.intern(value); try graph.setField(key, value: .string(id), on: node)
    }
    private static func array(_ graph: inout Pele.PublicGraph, _ values: [Pele.PublicGraph.Atom]) throws -> UInt32 {
        let id = UInt32(graph.nodes.count); graph.nodes.append(.array(length: 0, properties: []))
        for (i, value) in values.enumerated() { try graph.setArrayElement(UInt32(i), value: value, on: id) }; return id
    }
    private static func encoded(_ graph: Pele.PublicGraph, _ atom: Pele.PublicGraph.Atom) throws -> Any {
        switch atom {
        case .undefined: return NSNull()
        case .null: return NSNull()
        case .bool(let b): return b
        case .number(let n): return n
        case .string(let id): return try XCTUnwrap(graph.strings[Int(id)].stringIfWellFormed)
        case .ref(let id):
            if case .array(let length, _) = graph.nodes[Int(id)] {
                return try (0..<length).map { try encoded(graph, graph.field(String($0), on: id) ?? .undefined) }
            }
            var object: [String: Any] = [:]
            for p in graph.nodes[Int(id)].properties { if case .undefined=p.value {continue}; object[try XCTUnwrap(graph.strings[Int(p.key)].stringIfWellFormed)] = try encoded(graph, p.value) }; return object
        default: throw ArchitectureCallbackFailure()
        }
    }
    private static func state(_ graph:Pele.PublicGraph) throws -> [String:Any] {
        var result:[String:Any]=[:]
        for key in ["title","accTitle","accDescr","nodes","groups","edges","layoutHints"] {
            let value=try graph.field(key) ?? .undefined
            if case .undefined=value {continue}
            if ["nodes","groups"].contains(key) {
                let id=try ref(value);guard case .map(let entries,_)=graph.nodes[Int(id)] else{throw ArchitectureCallbackFailure()}
                result[key]=try entries.map{[try encoded(graph,$0.key),try encoded(graph,$0.value)]}
            }else{result[key]=try encoded(graph,value)}
        };return result
    }
    private static func resources(_ state: ArchitectureCallbackState? = nil) -> Pele.HostResources {
        var host = Pele.HostResources()
        host.textWidth = { request in MainActor.assumeIsolated {
            state?.trace.append([request.text, bits(request.size), Int(request.style)])
            return Double(request.text.utf16.count) * request.size * 0.6
        }}
        host.icon = { name in MainActor.assumeIsolated {
            state?.trace.append(["icon", name]); return Pele.TrustedSVG(trustedMarkup: "<path d=\"M0,0H2\"/>")
        }}
        return host
    }
    private func compare(_ actual: FullRenderedDiagram, _ expected: [String: Any]) throws {
        XCTAssertEqual(actual.svg, try XCTUnwrap(expected["svg"] as? String)); XCTAssertEqual(Self.bits(actual.width), expected["width"] as? String); XCTAssertEqual(Self.bits(actual.height), expected["height"] as? String); XCTAssertTrue(actual.links.isEmpty)
    }
    private static func mutate(_ document: ArchitectureCallbackDocument, _ state: ArchitectureCallbackState, _ mode: String, _ responsive: Bool) throws {
        state.fired = true; XCTAssertThrowsError(try document.close())
        var graph = try document.snapshot().graph; let n = try nodes(graph)
        switch mode {
        case "baseline":break
        case "late-title":try setText(&graph,"title","Changed title")
        case "late-accessibility":try setText(&graph,"accTitle","Changed access");try setText(&graph,"accDescr","Changed description")
        case "later-label":try setText(&graph,"title","Changed second",on:n.later)
        case "replace-maps":
            let c=try mapValue(graph,"nodes","c"),bk=try graph.intern("b"),ck=try graph.intern("c"),id=UInt32(graph.nodes.count)
            graph.nodes.append(.map(entries:[.init(key:.string(bk),value:.ref(n.later)),.init(key:.string(ck),value:.ref(c))],properties:[]));try graph.setField("nodes",value:.ref(id))
            let g=UInt32(graph.nodes.count);graph.nodes.append(.map(entries:[],properties:[]));try graph.setField("groups",value:.ref(g))
        case "append-node":
            let id=UInt32(graph.nodes.count);graph.nodes.append(.object(properties:[]))
            for(k,v)in[("id","d"),("type","service"),("icon","server"),("title","Appended")]{try setText(&graph,k,v,on:id)}
            let key=try graph.intern("d");guard case .map(var entries,let properties)=graph.nodes[Int(n.nodes)] else{throw ArchitectureCallbackFailure()};entries.append(.init(key:.string(key),value:.ref(id)));graph.nodes[Int(n.nodes)] = .map(entries:entries,properties:properties)
        case "edge-title":let e=try ref(graph.field("0",on:n.edges));try setText(&graph,"title","Changed edge",on:e)
        case "replace-edges":
            let old=try ref(graph.field("1",on:n.edges)),id=UInt32(graph.nodes.count);graph.nodes.append(graph.nodes[Int(old)]);try setText(&graph,"title","Replacement edge",on:id);let a=try array(&graph,[.ref(id)]);try graph.setField("edges",value:.ref(a))
        case "group-title":try setText(&graph,"title","Changed cluster",on:n.group)
        case "group-icon":try setText(&graph,"icon","disk",on:n.group)
        case "node-icon":try setText(&graph,"icon","internet",on:n.later)
        case "icon-group-id":try setText(&graph,"id","changed-group",on:n.group);try setText(&graph,"title","After icon")
        case "icon-later-node":try setText(&graph,"title","Changed after icon",on:n.later);try setText(&graph,"icon","internet",on:n.later);try setText(&graph,"id","changed-node",on:n.later)
        case "throw-object","throw-undefined":try setText(&graph,"title","Saved");try setText(&graph,"title","Saved later",on:n.later)
        case "nested":try setText(&graph,"title","Nested")
        default:throw ArchitectureCallbackFailure()
        }
        if mode != "baseline" {try document.apply(graph)}
        // Swift has Error values; both source throw lanes retain that exact object.
        if mode.hasPrefix("throw-") {throw state.failure}
        if mode == "nested" {
            state.nested = try document.render(resources(),responsive:false);state.nestedRevision=document.revision
            var latest=try document.snapshot().graph;try setText(&latest,"title","After nested");try document.apply(latest)
        }
    }
    func testArchitectureCopiedGraphLateMetadataResponsiveRedrawMatches64OracleRows() throws {
        var path = URL(fileURLWithPath: #filePath); for _ in 0..<5 { path.deleteLastPathComponent() }
        let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-architecture-live-render.json"))) as? [String: Any])
        XCTAssertEqual(fixture["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        let rows = try XCTUnwrap(fixture["rows"] as? [[String: Any]]), source = try XCTUnwrap(fixture["source"] as? String); XCTAssertEqual(rows.count, 64)
        for row in rows {
            let raw = try XCTUnwrap(row["raw"] as? Bool), responsive = try XCTUnwrap(row["responsive"] as? Bool), mode = try XCTUnwrap(row["mode"] as? String), engine = Pele.Engine()
            let document: ArchitectureCallbackDocument = try raw ? .prepared(engine.parsePrepared("architecture", source: source)) : .normal(engine.parseMutable(source)); defer { try? document.close() }
            var initial = try document.snapshot().graph;try Self.setText(&initial,"title","Initial");try document.apply(initial)
            let original = try Self.nodes(document.snapshot().graph), observed = ArchitectureCallbackState(); var host = Self.resources()
            host.textWidth = { request in try MainActor.assumeIsolated {
                observed.trace.append([request.text, Self.bits(request.size), Int(request.style)])
                let hit = request.text == "First" && !mode.hasPrefix("icon-")
                if !observed.fired && hit { try Self.mutate(document, observed, mode, responsive) }
                return Double(request.text.utf16.count) * request.size * 0.6
            }}
            host.icon = { name in try MainActor.assumeIsolated {
                observed.trace.append(["icon", name]); if !observed.fired && mode.hasPrefix("icon-") && name == "cloud" { try Self.mutate(document, observed, mode, responsive) }
                return Pele.TrustedSVG(trustedMarkup: "<path d=\"M0,0H2\"/>")
            }}
            let first = try XCTUnwrap(row["first"] as? [String: Any])
            if mode.hasPrefix("throw-") { XCTAssertEqual(first["original"] as? Bool, true); XCTAssertThrowsError(try document.render(host, responsive: responsive)) { XCTAssertTrue(($0 as AnyObject) === observed.failure) } }
            else { try compare(document.render(host, responsive: responsive), first) }
            XCTAssertTrue(observed.fired); XCTAssertTrue(NSArray(array: observed.trace).isEqual(to: try XCTUnwrap(row["trace"] as? [[Any]])), "\(mode)/\(raw)/\(responsive) trace")
            let after = try document.snapshot(); XCTAssertEqual(after.revision, document.revision)
            XCTAssertTrue(NSDictionary(dictionary:try Self.state(after.graph)).isEqual(to:try XCTUnwrap(row["after"] as? [String:Any])),mode+" state")
            XCTAssertEqual(try Self.ref(after.graph.field("nodes"))==original.nodes,row["nodesCurrent"] as? Bool)
            XCTAssertEqual(try Self.ref(after.graph.field("groups"))==original.groups,row["groupsCurrent"] as? Bool)
            XCTAssertEqual(try Self.mapValue(after.graph,"nodes","b")==original.later,row["laterCurrent"] as? Bool)
            XCTAssertEqual(try Self.ref(after.graph.field("edges"))==original.edges,row["edgesCurrent"] as? Bool)
            for(key,id)in[("originalFirst",original.first),("originalLater",original.later),("originalGroup",original.group)] { XCTAssertTrue(NSDictionary(dictionary:try XCTUnwrap(Self.encoded(after.graph,.ref(id)) as? [String:Any])).isEqual(to:try XCTUnwrap(row[key] as? [String:Any])),mode+key) }
            if mode == "nested" { try compare(XCTUnwrap(observed.nested), XCTUnwrap(row["nested"] as? [String: Any])); XCTAssertGreaterThan(document.revision, try XCTUnwrap(observed.nestedRevision)) }
            let recovery = ArchitectureCallbackState(); try compare(document.render(Self.resources(recovery), responsive: responsive), XCTUnwrap(row["recovery"] as? [String: Any]))
            XCTAssertTrue(NSArray(array: recovery.trace).isEqual(to: try XCTUnwrap(row["recoveryTrace"] as? [[Any]])), mode + " recovery trace")
        }
    }
}
