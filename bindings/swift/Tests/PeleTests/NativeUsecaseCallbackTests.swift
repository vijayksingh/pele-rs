import Foundation
import XCTest
import Pele

private final class UsecaseCallbackFailure: Error {}
@MainActor private enum UsecaseCallbackDocument {
    case normal(Pele.MutableDocument), prepared(Pele.PreparedDocument)
    var revision: UInt64 { switch self { case .normal(let d): d.revision; case .prepared(let d): d.revision } }
    func snapshot() throws -> Pele.PublicSnapshot { switch self { case .normal(let d): try d.snapshot(); case .prepared(let d): try d.snapshot() } }
    func apply(_ graph: Pele.PublicGraph) throws { switch self { case .normal(let d): _ = try d.apply(graph, expectedRevision: d.revision); case .prepared(let d): _ = try d.apply(graph, expectedRevision: d.revision) } }
    func render(_ resources: Pele.HostResources, responsive: Bool) throws -> FullRenderedDiagram {
        var options = Pele.Engine.Options(); options.maxWidth = responsive ? 200 : nil
        switch self { case .normal(let d): return try d.render(options: options, resources: resources); case .prepared(let d): return try d.render(options: options, resources: resources) }
    }
    func close() throws { switch self { case .normal(let d): try d.close(); case .prepared(let d): try d.close() } }
}
@MainActor private final class UsecaseCallbackState {
    var fired = false
    var trace: [[Any]] = []
    var nested: FullRenderedDiagram?
    var nestedRevision: UInt64?
    let failure = UsecaseCallbackFailure()
}
@MainActor final class NativeUsecaseCallbackTests: XCTestCase {
    private static func bits(_ n: Double) -> String { String(format: "%016llx", n.bitPattern) }
    private static func ref(_ value: Pele.PublicGraph.Atom?) throws -> UInt32 {
        guard case .ref(let id) = value else { throw PeleError(code: "test", message: "Expected Usecase reference") }; return id
    }
    private static func mapValue(_ graph:Pele.PublicGraph,_ key:String,_ name:String)throws->UInt32{
        let map=try ref(graph.field(key));guard case .map(let entries,_)=graph.nodes[Int(map)] else{throw UsecaseCallbackFailure()};for entry in entries{if case .string(let id)=entry.key,graph.strings[Int(id)].stringIfWellFormed==name{return try ref(entry.value)}};throw UsecaseCallbackFailure()
    }
    private static func nodes(_ graph:Pele.PublicGraph)throws->(actors:UInt32,cases:UInt32,later:UInt32,relations:UInt32){
        (try ref(graph.field("actors")),try ref(graph.field("useCases")),try mapValue(graph,"useCases","V"),try ref(graph.field("relationships")))
    }
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
        default: throw UsecaseCallbackFailure()
        }
    }
    private static func state(_ graph:Pele.PublicGraph) throws -> [String:Any] {
        var result:[String:Any]=[:]
        for key in ["title","accTitle","accDescr","direction","actors","useCases","relationships","jsonNodes","classDefs"] {
            let value=graph.field(key) ?? .undefined
            if ["actors","useCases","jsonNodes","classDefs"].contains(key) {
                let id=try ref(value);guard case .map(let entries,_)=graph.nodes[Int(id)] else{throw UsecaseCallbackFailure()}
                result[key]=try entries.map{[try encoded(graph,$0.key),try encoded(graph,$0.value)]}
            }else{result[key]=try encoded(graph,value)}
        };return result
    }
    private static func resources(_ state: UsecaseCallbackState? = nil) -> Pele.HostResources {
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
    private static func mutate(_ document: UsecaseCallbackDocument, _ state: UsecaseCallbackState, _ mode: String, _ responsive: Bool) throws {
        state.fired = true; XCTAssertThrowsError(try document.close())
        var graph = try document.snapshot().graph; let n = try nodes(graph)
        switch mode {
        case "baseline":break
        case "late-title":try setText(&graph,"title","Changed title")
        case "late-accessibility":try setText(&graph,"accTitle","Changed access");try setText(&graph,"accDescr","Changed description")
        case "later-label":try setText(&graph,"label","Changed later",on:n.later)
        case "replace-maps":
            let a=UInt32(graph.nodes.count);graph.nodes.append(.map(entries:[],properties:[]));try graph.setField("actors",value:.ref(a));let key=try graph.intern("V"),b=UInt32(graph.nodes.count);graph.nodes.append(.map(entries:[.init(key:.string(key),value:.ref(n.later))],properties:[]));try graph.setField("useCases",value:.ref(b))
        case "relationship":let first=try ref(graph.field("0",on:n.relations));try setText(&graph,"label","Changed relation",on:first);try setText(&graph,"target","V",on:first)
        case "class-style":
            let d=try mapValue(graph,"classDefs","blue"),a=try ref(graph.field("styles",on:d));guard case .array(let length,_)=graph.nodes[Int(a)]else{throw UsecaseCallbackFailure()};let v=try graph.intern("fill:#ff0000");try graph.setArrayElement(length,value:.string(v),on:a)
        case "json-value":let j=try mapValue(graph,"jsonNodes","J"),value=try ref(graph.field("value",on:j));try setText(&graph,"first","Changed JSON",on:value)
        case "direction":try setText(&graph,"direction","TB")
        case "throw-object","throw-undefined":try setText(&graph,"title","Saved");try setText(&graph,"label","Saved later",on:n.later)
        case "nested":try setText(&graph,"title","Nested")
        default:throw UsecaseCallbackFailure()
        }
        if mode != "baseline" {try document.apply(graph)}
        // Swift has Error values; both source throw lanes retain that exact object.
        if mode.hasPrefix("throw-") {throw state.failure}
        if mode == "nested" {
            state.nested = try document.render(resources(),responsive:false);state.nestedRevision=document.revision
            var latest=try document.snapshot().graph;try setText(&latest,"title","After nested");try document.apply(latest)
        }
    }
    func testUsecaseCopiedGraphLateMetadataResponsiveRedrawMatches48OracleRows() throws {
        var path = URL(fileURLWithPath: #filePath); for _ in 0..<5 { path.deleteLastPathComponent() }
        let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-usecase-live-render.json"))) as? [String: Any])
        XCTAssertEqual(fixture["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        let rows = try XCTUnwrap(fixture["rows"] as? [[String: Any]]), source = try XCTUnwrap(fixture["source"] as? String); XCTAssertEqual(rows.count, 48)
        for row in rows {
            let raw = try XCTUnwrap(row["raw"] as? Bool), responsive = try XCTUnwrap(row["responsive"] as? Bool), mode = try XCTUnwrap(row["mode"] as? String), engine = Pele.Engine()
            let document: UsecaseCallbackDocument = try raw ? .prepared(engine.parsePrepared("usecase", source: source)) : .normal(engine.parseMutable(source)); defer { try? document.close() }
            var initial = try document.snapshot().graph;try Self.setText(&initial,"title","Initial");try document.apply(initial)
            let original = try Self.nodes(document.snapshot().graph), observed = UsecaseCallbackState(); var host = Self.resources()
            host.textWidth = { request in try MainActor.assumeIsolated {
                observed.trace.append([request.text, Self.bits(request.size), Int(request.style)])
                let hit = request.text == "Person"
                if !observed.fired && hit { try Self.mutate(document, observed, mode, responsive) }
                return Double(request.text.utf16.count) * request.size * 0.6
            }}
            host.icon = { name in try MainActor.assumeIsolated {
                observed.trace.append(["icon", name]); if !observed.fired && mode.hasPrefix("icon-") { try Self.mutate(document, observed, mode, responsive) }
                return Pele.TrustedSVG(trustedMarkup: "<path d=\"M0,0H2\"/>")
            }}
            let first = try XCTUnwrap(row["first"] as? [String: Any])
            if mode.hasPrefix("throw-") { XCTAssertEqual(first["original"] as? Bool, true); XCTAssertThrowsError(try document.render(host, responsive: responsive)) { XCTAssertTrue(($0 as AnyObject) === observed.failure) } }
            else { try compare(document.render(host, responsive: responsive), first) }
            XCTAssertTrue(observed.fired); XCTAssertTrue(NSArray(array: observed.trace).isEqual(to: try XCTUnwrap(row["trace"] as? [[Any]])), "\(mode)/\(raw)/\(responsive) trace")
            let after = try document.snapshot(); XCTAssertEqual(after.revision, document.revision)
            XCTAssertTrue(NSDictionary(dictionary:try Self.state(after.graph)).isEqual(to:try XCTUnwrap(row["after"] as? [String:Any])),mode+" state")
            XCTAssertEqual(try Self.ref(after.graph.field("actors"))==original.actors,row["actorsCurrent"] as? Bool)
            XCTAssertEqual(try Self.ref(after.graph.field("useCases"))==original.cases,row["casesCurrent"] as? Bool)
            XCTAssertEqual(try Self.mapValue(after.graph,"useCases","V")==original.later,row["laterCurrent"] as? Bool)
            XCTAssertEqual(try Self.ref(after.graph.field("relationships"))==original.relations,row["relationsCurrent"] as? Bool)
            if mode == "nested" { try compare(XCTUnwrap(observed.nested), XCTUnwrap(row["nested"] as? [String: Any])); XCTAssertGreaterThan(document.revision, try XCTUnwrap(observed.nestedRevision)) }
            let recovery = UsecaseCallbackState(); try compare(document.render(Self.resources(recovery), responsive: responsive), XCTUnwrap(row["recovery"] as? [String: Any]))
            XCTAssertTrue(NSArray(array: recovery.trace).isEqual(to: try XCTUnwrap(row["recoveryTrace"] as? [[Any]])), mode + " recovery trace")
        }
    }
}
