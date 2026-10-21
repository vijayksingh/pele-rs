import Foundation
import XCTest
import Pele

private final class ErCallbackFailure: Error {}
@MainActor private enum ErCallbackDocument {
    case normal(Pele.MutableDocument), prepared(Pele.PreparedDocument)
    var revision: UInt64 { switch self { case .normal(let d): d.revision; case .prepared(let d): d.revision } }
    func snapshot() throws -> Pele.PublicSnapshot { switch self { case .normal(let d): try d.snapshot(); case .prepared(let d): try d.snapshot() } }
    func apply(_ graph: Pele.PublicGraph) throws { switch self { case .normal(let d): _ = try d.apply(graph, expectedRevision: d.revision); case .prepared(let d): _ = try d.apply(graph, expectedRevision: d.revision) } }
    func render(_ resources: Pele.HostResources, responsive: Bool) throws -> FullRenderedDiagram {
        var options = Pele.Engine.Options(); options.maxWidth = responsive ? 180 : nil
        switch self { case .normal(let d): return try d.render(options: options, resources: resources); case .prepared(let d): return try d.render(options: options, resources: resources) }
    }
    func invoke(_ root: Pele.PublicGraph.Atom, _ arg: Pele.PublicGraph.Atom) throws -> Pele.PublicInvocationOutcome { switch self { case .normal(let d): try d.invoke(method: 1, receiver: root, arguments: [arg]); case .prepared(let d): try d.invoke(method: 1, receiver: root, arguments: [arg]) } }
    func close() throws { switch self { case .normal(let d): try d.close(); case .prepared(let d): try d.close() } }
}
@MainActor private final class ErCallbackState {
    var fired = false
    var trace: [[Any]] = []
    var nested: FullRenderedDiagram?
    var nestedRevision: UInt64?
    var helperIdentity: Bool?
    let failure = ErCallbackFailure()
}
@MainActor final class NativeErCallbackTests: XCTestCase {
    private static func bits(_ n: Double) -> String { String(format: "%016llx", n.bitPattern) }
    private static func ref(_ value: Pele.PublicGraph.Atom?) throws -> UInt32 {
        guard case .ref(let id) = value else { throw PeleError(code: "test", message: "Expected Er reference") }; return id
    }
    private static func entity(_ graph: Pele.PublicGraph, _ name: String) throws -> UInt32 {
        let map = try ref(graph.field("entities"))
        guard case .map(let entries, _) = graph.nodes[Int(map)] else { throw ErCallbackFailure() }
        for entry in entries { if case .string(let id) = entry.key, graph.strings[Int(id)].stringIfWellFormed == name { return try ref(entry.value) } }
        throw ErCallbackFailure()
    }
    private static func nodes(_ graph: Pele.PublicGraph) throws -> (saved: UInt32, attributes: UInt32, styles: UInt32, first: UInt32) {
        let saved = try entity(graph,"C"), attributes = try ref(graph.field("attributes",on:saved)), styles = try ref(graph.field("cssStyles",on:saved)), a = try entity(graph,"A"), rows = try ref(graph.field("attributes",on:a))
        return (saved,attributes,styles,try ref(graph.field("0",on:rows)))
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
        case .undefined: return ["undefined": true]
        case .null: return NSNull()
        case .bool(let b): return b
        case .number(let n): return ["number": bits(n)]
        case .string(let id): return try XCTUnwrap(graph.strings[Int(id)].stringIfWellFormed)
        case .ref(let id):
            if case .array(let length, _) = graph.nodes[Int(id)] {
                return try (0..<length).map { try encoded(graph, graph.field(String($0), on: id) ?? .undefined) }
            }
            if case .map(let entries, _) = graph.nodes[Int(id)] { return ["map": try entries.map { entry in [try encoded(graph,entry.key),try encoded(graph,entry.value)] }] }
            if case .set(let entries, _) = graph.nodes[Int(id)] { return ["set": try entries.map { try encoded(graph,$0) }] }
            var object: [String: Any] = [:]
            for p in graph.nodes[Int(id)].properties { object[try XCTUnwrap(graph.strings[Int(p.key)].stringIfWellFormed)] = try encoded(graph, p.value) }; return object
        default: throw ErCallbackFailure()
        }
    }
    private static func resources(_ state: ErCallbackState? = nil) -> Pele.HostResources {
        var host = Pele.HostResources()
        host.textWidth = { request in MainActor.assumeIsolated {
            state?.trace.append(["width", request.text, bits(request.size), Int(request.style)])
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
    private static func mutate(_ document: ErCallbackDocument, _ state: ErCallbackState, _ mode: String, _ responsive: Bool) throws {
        state.fired = true; XCTAssertThrowsError(try document.close())
        var graph = try document.snapshot().graph; let n = try nodes(graph)
        let later = try ref(graph.field("1",on:n.attributes))
        switch mode {
        case "baseline", "helper": break
        case "current": try setText(&graph,"name","Changed current",on:n.first)
        case "later": try setText(&graph,"name","Changed later",on:later); try setText(&graph,"alias","Changed alias",on:n.saved)
        case "replace-array":
            let id = UInt32(graph.nodes.count); graph.nodes.append(.object(properties:[])); try setText(&graph,"type","string",on:id); try setText(&graph,"name","Replacement",on:id); let keys = try array(&graph,[]); try graph.setField("keys",value:.ref(keys),on:id); try setText(&graph,"comment","",on:id); let list = try array(&graph,[.ref(id)]); try graph.setField("attributes",value:.ref(list),on:n.saved)
        case "detached-array": let list = try array(&graph,[]); try graph.setField("attributes",value:.ref(list),on:n.saved); try setText(&graph,"name","Changed detached",on:later)
        case "styles": let style = try graph.intern("fill:red"); try graph.setArrayElement(0,value:.string(style),on:n.styles)
        case "direction": try setText(&graph,"direction","TB")
        case "metadata": try setText(&graph,"title","Changed title"); try setText(&graph,"accTitle","Changed access"); try setText(&graph,"accDescr","Changed description")
        case "throw-object", "throw-undefined": try setText(&graph,"title","Persisted title"); try setText(&graph,"name","Persisted later",on:later)
        case "nested-render": try setText(&graph,"name","Nested later",on:later)
        default: throw ErCallbackFailure()
        }
        if mode != "baseline" && mode != "helper" { try document.apply(graph) }
        if mode == "helper" {
            let id = try XCTUnwrap(graph.strings.firstIndex(where: { $0.stringIfWellFormed == "C" })), outcome = try document.invoke(graph.root,.string(UInt32(id)))
            if case .returned(.ref(let result)) = outcome.result { state.helperIdentity = result == n.saved } else { throw ErCallbackFailure() }
        }
        // Swift has Error values, not JS undefined throws; both rows retain
        // the actual original Swift Error identity through C15 cleanup.
        if mode.hasPrefix("throw-") || mode == "icon-throw" { throw state.failure }
        if mode == "nested-render" {
            state.nested = try document.render(resources(), responsive: responsive); state.nestedRevision = document.revision
            var latest = try document.snapshot().graph; try setText(&latest, "title", "After nested"); try document.apply(latest)
        }
    }
    func testErCapturedNodeIdentityLiveChildrenIconsAndErrorsMatch48OracleRows() throws {
        var path = URL(fileURLWithPath: #filePath); for _ in 0..<5 { path.deleteLastPathComponent() }
        let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-er-owned-live-render.json"))) as? [String: Any])
        XCTAssertEqual(fixture["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        let rows = try XCTUnwrap(fixture["rows"] as? [[String: Any]]), source = try XCTUnwrap(fixture["source"] as? String); XCTAssertEqual(rows.count, 48)
        for row in rows {
            let raw = try XCTUnwrap(row["raw"] as? Bool), responsive = try XCTUnwrap(row["responsive"] as? Bool), mode = try XCTUnwrap(row["mode"] as? String), engine = Pele.Engine()
            let document: ErCallbackDocument = try raw ? .prepared(engine.parsePrepared("er", source: source)) : .normal(engine.parseMutable(source)); defer { try? document.close() }
            let original = try Self.nodes(document.snapshot().graph), observed = ErCallbackState(); var host = Self.resources()
            host.textWidth = { request in try MainActor.assumeIsolated {
                observed.trace.append(["width", request.text, Self.bits(request.size), Int(request.style)])
                let hit = mode != "current" || request.text == "int"
                if !observed.fired && hit { try Self.mutate(document, observed, mode, responsive) }
                return Double(request.text.utf16.count) * request.size * 0.6
            }}
            host.icon = { name in try MainActor.assumeIsolated {
                observed.trace.append(["icon", name]); if !observed.fired && mode.hasPrefix("icon-") { try Self.mutate(document, observed, mode, responsive) }
                return Pele.TrustedSVG(trustedMarkup: "<path d=\"M0,0H2\"/>")
            }}
            let first = try XCTUnwrap(row["first"] as? [String: Any])
            if mode.hasPrefix("throw-") || mode == "icon-throw" { XCTAssertEqual(first["original"] as? Bool, true); XCTAssertThrowsError(try document.render(host, responsive: responsive)) { XCTAssertTrue(($0 as AnyObject) === observed.failure) } }
            else { try compare(document.render(host, responsive: responsive), first) }
            XCTAssertTrue(observed.fired); XCTAssertTrue(NSArray(array: observed.trace).isEqual(to: try XCTUnwrap(row["trace"] as? [[Any]])), "\(mode)/\(raw)/\(responsive) trace")
            let after = try document.snapshot(); XCTAssertEqual(after.revision, document.revision)
            XCTAssertTrue(NSDictionary(dictionary: try XCTUnwrap(Self.encoded(after.graph, after.graph.root) as? [String: Any])).isEqual(to: try XCTUnwrap(row["after"] as? [String: Any])), mode + " state")
            let captured: [String:Any] = ["saved":try Self.encoded(after.graph,.ref(original.saved)),"attributes":try Self.encoded(after.graph,.ref(original.attributes)),"styles":try Self.encoded(after.graph,.ref(original.styles)),"first":try Self.encoded(after.graph,.ref(original.first))]
            XCTAssertTrue(NSDictionary(dictionary:captured).isEqual(to:try XCTUnwrap(row["captured"] as? [String:Any])),mode+" captured")
            if mode == "helper" { XCTAssertEqual(observed.helperIdentity,row["helperIdentity"] as? Bool) }
            if mode == "nested-render" { try compare(XCTUnwrap(observed.nested), XCTUnwrap(row["nested"] as? [String: Any])); XCTAssertGreaterThan(document.revision, try XCTUnwrap(observed.nestedRevision)) }
            let recovery = ErCallbackState(); try compare(document.render(Self.resources(recovery), responsive: responsive), XCTUnwrap(row["recovery"] as? [String: Any]))
            XCTAssertTrue(NSArray(array: recovery.trace).isEqual(to: try XCTUnwrap(row["recoveryTrace"] as? [[Any]])), mode + " recovery trace")
        }
    }
    func testErLiveJoinedAttributeTextBudgetPreservesWritesAndRecovery() throws {
        let engine = Pele.Engine(), document = try engine.parseMutable("erDiagram\nA { int id PK }")
        defer { try? document.close() }
        let original = try document.render(resources: Self.resources()), state = ErCallbackState()
        var host = Self.resources()
        host.textWidth = { request in try MainActor.assumeIsolated {
            if !state.fired {
                state.fired = true
                var graph = try document.snapshot().graph
                let entity = try Self.entity(graph,"A"), rows = try Self.ref(graph.field("attributes",on:entity)), row = try Self.ref(graph.field("0",on:rows)), shared = try graph.intern(String(repeating:"x",count:8192)), keys = try Self.array(&graph,Array(repeating:.string(shared),count:2049))
                try graph.setField("keys",value:.ref(keys),on:row); _ = try document.apply(graph,expectedRevision:document.revision)
            }
            return Double(request.text.utf16.count)*request.size*0.6
        }}
        XCTAssertThrowsError(try document.render(resources:host)) { XCTAssertEqual(($0 as? PeleError)?.code,"limit") }
        XCTAssertTrue(state.fired)
        var graph = try document.snapshot().graph
        let entity = try Self.entity(graph,"A"), rows = try Self.ref(graph.field("attributes",on:entity)), row = try Self.ref(graph.field("0",on:rows)), oldKeys = try Self.ref(graph.field("keys",on:row))
        guard case .array(let count,_) = graph.nodes[Int(oldKeys)] else { throw ErCallbackFailure() }; XCTAssertEqual(count,2049)
        let key = try graph.intern("PK"), keys = try Self.array(&graph,[.string(key)]); try graph.setField("keys",value:.ref(keys),on:row); _ = try document.apply(graph,expectedRevision:document.revision)
        let recovered = try document.render(resources:Self.resources()); XCTAssertEqual(recovered.svg,original.svg); XCTAssertEqual(recovered.width.bitPattern,original.width.bitPattern); XCTAssertEqual(recovered.height.bitPattern,original.height.bitPattern)
    }

}
