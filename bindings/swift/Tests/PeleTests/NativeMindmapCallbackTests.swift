import Foundation
import XCTest
import Pele

private final class MindmapCallbackFailure: Error {}
@MainActor private enum MindmapCallbackDocument {
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
@MainActor private final class MindmapCallbackState {
    var fired = false
    var trace: [[Any]] = []
    var nested: FullRenderedDiagram?
    var nestedRevision: UInt64?
    let failure = MindmapCallbackFailure()
}

@MainActor final class NativeMindmapCallbackTests: XCTestCase {
    private static func bits(_ n: Double) -> String { String(format: "%016llx", n.bitPattern) }
    private static func ref(_ value: Pele.PublicGraph.Atom?) throws -> UInt32 {
        guard case .ref(let id) = value else { throw PeleError(code: "test", message: "Expected Mindmap reference") }; return id
    }
    private static func first(_ graph: Pele.PublicGraph) throws -> (nodes: UInt32, root: UInt32) {
        let nodes = try ref(graph.field("nodes")); return (nodes, try ref(graph.field("0", on: nodes)))
    }
    private static func setText(_ graph: inout Pele.PublicGraph, _ key: String, _ value: String, on node: UInt32? = nil) throws {
        let id = try graph.intern(value); try graph.setField(key, value: .string(id), on: node)
    }
    private static func array(_ graph: inout Pele.PublicGraph, _ values: [Pele.PublicGraph.Atom]) throws -> UInt32 {
        let id = UInt32(graph.nodes.count); graph.nodes.append(.array(length: 0, properties: []))
        for (i, value) in values.enumerated() { try graph.setArrayElement(UInt32(i), value: value, on: id) }; return id
    }
    private static func state(_ graph: Pele.PublicGraph) throws -> [String: Any] {
        func scalar(_ atom: Pele.PublicGraph.Atom?) throws -> Any {
            switch atom {
            case .string(let id): return try XCTUnwrap(graph.strings[Int(id)].stringIfWellFormed)
            case .number(let n): return bits(n)
            default: return NSNull()
            }
        }
        let first = try first(graph)
        guard case .array(let length, _) = graph.nodes[Int(first.nodes)] else { throw MindmapCallbackFailure() }
        var nodes: [[String: Any]] = []
        for i in 0..<length {
            let node = try ref(graph.field(String(i), on: first.nodes)), children = try ref(graph.field("children", on: node))
            guard case .array(let length, _) = graph.nodes[Int(children)] else { throw MindmapCallbackFailure() }
            var child: [Any] = []
            for j in 0..<length { let c = try ref(graph.field(String(j), on: children)); child.append(try scalar(graph.field("id", on: c))) }
            var row: [String: Any] = ["children": child]
            for field in ["id", "nodeId", "descr", "type", "class", "icon", "section"] { row[field] = try scalar(graph.field(field, on: node)) }
            nodes.append(row)
        }
        return ["title": try scalar(graph.field("title")), "nodes": nodes]
    }
    private static func resources(_ state: MindmapCallbackState? = nil) -> Pele.HostResources {
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
    private static func mutate(_ document: MindmapCallbackDocument, _ state: MindmapCallbackState, _ originalMode: String) throws {
        state.fired = true; XCTAssertThrowsError(try document.close())
        let mode = originalMode.hasPrefix("icon-") ? String(originalMode.dropFirst(5)) : originalMode
        var graph = try document.snapshot().graph; let n = try first(graph)
        switch mode {
        case "baseline", "decorate": break
        case "title": try setText(&graph, "title", "Changed title")
        case "current-descr": try setText(&graph, "descr", "Changed Root", on: n.root)
        case "later-descr": let later = try ref(graph.field("1", on: n.nodes)); try setText(&graph, "descr", "Changed Alpha", on: later)
        case "type": let later = try ref(graph.field("1", on: n.nodes)); try graph.setField("type", value: .number(2), on: n.root); try graph.setField("type", value: .number(3), on: later)
        case "replace-nodes":
            let replacement = UInt32(graph.nodes.count); graph.nodes.append(graph.nodes[Int(n.root)])
            let empty = try array(&graph, []); try graph.setField("children", value: .ref(empty), on: replacement); try setText(&graph, "descr", "Replacement root", on: replacement)
            let nodes = try array(&graph, [.ref(replacement)]); try graph.setField("nodes", value: .ref(nodes))
        case "throw-object", "throw-undefined": try setText(&graph, "title", "Before failure"); try setText(&graph, "descr", "Persisted root", on: n.root)
        case "nested": try setText(&graph, "title", "Nested title")
        default: throw MindmapCallbackFailure()
        }
        if mode != "baseline" && mode != "decorate" { try document.apply(graph) }
        if mode == "decorate" {
            let argument = UInt32(graph.nodes.count); graph.nodes.append(.object(properties: []))
            try setText(&graph, "class", "decorated", on: argument); try setText(&graph, "icon", "decorated-icon", on: argument)
            try document.apply(graph); try document.invoke(method: 2, receiver: graph.root, arguments: [.ref(argument)])
        }
        // Swift has Error values, not JS undefined throws. Both fixture lanes
        // retain the actual original Swift error through C15 cleanup.
        if mode.hasPrefix("throw-") { throw state.failure }
        if mode == "nested" {
            state.nested = try document.render(resources()); state.nestedRevision = document.revision
            var latest = try document.snapshot().graph; try setText(&latest, "title", "After nested"); try document.apply(latest)
        }
    }
    func testMindmapCapturedNodesLateShapeIconsHelpersAndErrorsMatch48OracleRows() throws {
        var path = URL(fileURLWithPath: #filePath); for _ in 0..<5 { path.deleteLastPathComponent() }
        let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-mindmap-live-render.json"))) as? [String: Any])
        XCTAssertEqual(fixture["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        let source = try XCTUnwrap(fixture["source"] as? String), rows = try XCTUnwrap(fixture["rows"] as? [[String: Any]]); XCTAssertEqual(rows.count, 48)
        for row in rows {
            let raw = try XCTUnwrap(row["raw"] as? Bool), responsive = try XCTUnwrap(row["responsive"] as? Bool), mode = try XCTUnwrap(row["mode"] as? String), engine = Pele.Engine()
            let document: MindmapCallbackDocument = try raw ? .prepared(engine.parsePrepared("mindmap", source: source)) : .normal(engine.parseMutable(source)); defer { try? document.close() }
            var initial = try document.snapshot().graph; let n = try Self.first(initial), later = try Self.ref(initial.field("1", on: n.nodes))
            try Self.setText(&initial, "title", "Initial title"); try Self.setText(&initial, "icon", "root-icon", on: n.root); try Self.setText(&initial, "icon", "alpha-icon", on: later); try document.apply(initial)
            let original = try Self.first(document.snapshot().graph), observed = MindmapCallbackState(); var host = Self.resources()
            host.textWidth = { request in try MainActor.assumeIsolated {
                observed.trace.append(["width", request.text, Self.bits(request.size), Int(request.style)])
                if !observed.fired && !mode.hasPrefix("icon-") { try Self.mutate(document, observed, mode) }
                return Double(request.text.utf16.count) * request.size * 0.6
            }}
            host.icon = { name in try MainActor.assumeIsolated {
                observed.trace.append(["icon", name]); if !observed.fired && mode.hasPrefix("icon-") { try Self.mutate(document, observed, mode) }
                return Pele.TrustedSVG(trustedMarkup: "<path d=\"M0 0h1\"/>")
            }}
            let first = try XCTUnwrap(row["first"] as? [String: Any])
            if mode.contains("throw-") { XCTAssertEqual(first["original"] as? Bool, true); XCTAssertThrowsError(try document.render(host, responsive: responsive)) { XCTAssertTrue(($0 as AnyObject) === observed.failure) } }
            else { try compare(document.render(host, responsive: responsive), first) }
            XCTAssertTrue(observed.fired); XCTAssertTrue(NSArray(array: observed.trace).isEqual(to: try XCTUnwrap(row["trace"] as? [[Any]])), "\(mode)/\(raw)/\(responsive) trace")
            let after = try document.snapshot(); XCTAssertEqual(after.revision, document.revision)
            XCTAssertTrue(NSDictionary(dictionary: try Self.state(after.graph)).isEqual(to: try XCTUnwrap(row["after"] as? [String: Any])), mode + " state")
            let current = try Self.first(after.graph), identity = try XCTUnwrap(row["identity"] as? [String: Bool]); XCTAssertEqual(current.nodes == original.nodes, identity["nodes"]); XCTAssertEqual(current.root == original.root, identity["root"])
            if mode == "nested" || mode == "icon-nested" { try compare(XCTUnwrap(observed.nested), XCTUnwrap(row["nested"] as? [String: Any])); XCTAssertGreaterThan(document.revision, try XCTUnwrap(observed.nestedRevision)) }
            let recovery = MindmapCallbackState(); try compare(document.render(Self.resources(recovery), responsive: responsive), XCTUnwrap(row["recovery"] as? [String: Any]))
            XCTAssertTrue(NSArray(array: recovery.trace).isEqual(to: try XCTUnwrap(row["recoveryTrace"] as? [[Any]])), mode + " recovery trace")
        }
    }
}
