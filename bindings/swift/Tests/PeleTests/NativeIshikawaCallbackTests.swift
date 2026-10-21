import Foundation
import XCTest
import Pele

private final class IshikawaCallbackFailure: Error {}
@MainActor private enum IshikawaCallbackDocument {
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
@MainActor private final class IshikawaCallbackState {
    var fired = false
    var trace: [[Any]] = []
    var nested: FullRenderedDiagram?
    var nestedRevision: UInt64?
    let failure = IshikawaCallbackFailure()
}

@MainActor final class NativeIshikawaCallbackTests: XCTestCase {
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
    private static func ref(_ value: Pele.PublicGraph.Atom?) throws -> UInt32 {
        guard case .ref(let id) = value else { throw PeleError(code: "test", message: "Expected Ishikawa reference") }; return id
    }
    private static func first(_ graph: Pele.PublicGraph) throws -> (root: UInt32, children: UInt32, category: UInt32) {
        let root = try ref(graph.field("root")), children = try ref(graph.field("children", on: root)), category = try ref(graph.field("0", on: children)); return (root, children, category)
    }
    private static func setText(_ graph: inout Pele.PublicGraph, _ key: String, _ text: String, on object: UInt32? = nil) throws {
        let id = try graph.intern(text); try graph.setField(key, value: .string(id), on: object)
    }
    private static func node(_ graph: inout Pele.PublicGraph, _ text: String, children: [Pele.PublicGraph.Atom] = []) throws -> UInt32 {
        let id = UInt32(graph.nodes.count); graph.nodes.append(.object(properties: []))
        try setText(&graph, "text", text, on: id)
        let child = try array(&graph, children); try graph.setField("children", value: .ref(child), on: id)
        return id
    }
    private static func array(_ graph: inout Pele.PublicGraph, _ values: [Pele.PublicGraph.Atom]) throws -> UInt32 {
        let id = UInt32(graph.nodes.count); graph.nodes.append(.array(length: 0, properties: []))
        for (i, value) in values.enumerated() { try graph.setArrayElement(UInt32(i), value: value, on: id) }; return id
    }
    private func state(_ graph: Pele.PublicGraph) throws -> [String: Any] {
        func text(_ atom: Pele.PublicGraph.Atom?) throws -> Any {
            guard case .string(let i) = atom else { return NSNull() }
            return try XCTUnwrap(graph.strings[Int(i)].stringIfWellFormed)
        }
        func tree(_ id: UInt32) throws -> [String: Any] {
            let children = try Self.ref(graph.field("children", on: id))
            guard case .array(let length, _) = graph.nodes[Int(children)] else { throw IshikawaCallbackFailure() }
            var output: [[String: Any]] = []
            for i in 0..<length { output.append(try tree(Self.ref(graph.field(String(i), on: children)))) }
            return ["text": try text(graph.field("text", on: id)), "children": output]
        }
        return ["title": try text(graph.field("title")), "root": try tree(Self.ref(graph.field("root")))]
    }
    func testIshikawaRetainedTreePhasesHelpersOriginalErrorsAndNestedRendersMatch48OracleRows() throws {
        var path = URL(fileURLWithPath: #filePath); for _ in 0..<5 { path.deleteLastPathComponent() }
        let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-ishikawa-live-render.json"))) as? [String: Any])
        XCTAssertEqual(fixture["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        let source = try XCTUnwrap(fixture["source"] as? String), rows = try XCTUnwrap(fixture["rows"] as? [[String: Any]])
        XCTAssertEqual(rows.count, 48)
        for row in rows {
            let raw = try XCTUnwrap(row["raw"] as? Bool), responsive = try XCTUnwrap(row["responsive"] as? Bool), mode = try XCTUnwrap(row["mode"] as? String)
            let engine = Pele.Engine()
            let document: IshikawaCallbackDocument = try raw ? .prepared(engine.parsePrepared("ishikawa", source: source)) : .normal(engine.parseMutable(source))
            defer { try? document.close() }
            var initialGraph = try document.snapshot().graph
            try Self.setText(&initialGraph, "title", "Initial title"); try document.apply(initialGraph)
            let original = try Self.first(document.snapshot().graph), observed = IshikawaCallbackState()
            var resources = pure()
            resources.textWidth = { request in try MainActor.assumeIsolated {
                observed.trace.append([request.text, String(format: "%016llx", request.size.bitPattern), Int(request.style)])
                if !observed.fired && (!["current-text", "later-text", "append-child", "replace-child", "replace-children"].contains(mode) || request.text == "Category A") {
                    observed.fired = true
                    XCTAssertThrowsError(try document.close())
                    var graph = try document.snapshot().graph
                    let initial = try Self.first(graph)
                    switch mode {
                    case "title": try Self.setText(&graph, "title", "Changed title")
                    case "current-text": try Self.setText(&graph, "text", "Changed A", on: original.category)
                    case "later-text":
                        let second = try Self.ref(graph.field("1", on: initial.children)); try Self.setText(&graph, "text", "Changed B", on: second)
                    case "append-child":
                        let child = try Self.node(&graph, "Added cause"), children = try Self.ref(graph.field("children", on: original.category))
                        guard case .array(let length, _) = graph.nodes[Int(children)] else { throw observed.failure }
                        try graph.setArrayElement(length, value: .ref(child), on: children)
                    case "replace-child":
                        let child = try Self.node(&graph, "Replacement category"); try graph.setArrayElement(1, value: .ref(child), on: initial.children)
                    case "replace-children":
                        let child = try Self.node(&graph, "Replacement cause"), children = try Self.array(&graph, [.ref(child)])
                        try graph.setField("children", value: .ref(children), on: original.category)
                    case "replace-root":
                        let child = try Self.node(&graph, "Fresh category"), root = try Self.node(&graph, "Replacement root", children: [.ref(child)])
                        try graph.setField("root", value: .ref(root))
                    case "helper":
                        let label = try graph.intern("Helper cause"); try document.apply(graph)
                        try document.invoke(method: 0, receiver: graph.root, arguments: [.number(4), .string(label)])
                    case "throw-object", "throw-undefined":
                        try Self.setText(&graph, "title", "Before failure"); try Self.setText(&graph, "text", "Persisted category", on: original.category)
                    case "nested": try Self.setText(&graph, "title", "Nested title")
                    case "baseline": break
                    default: throw PeleError(code: "test", message: "Unknown Ishikawa fixture mode")
                    }
                    if mode != "baseline" && mode != "helper" { try document.apply(graph) }
                    // Swift has Error values, not JS undefined throws. Both oracle
                    // throw rows verify Swift's exact original Error identity.
                    if mode.hasPrefix("throw-") { throw observed.failure }
                    if mode == "nested" {
                        var inner = Pele.HostResources(); inner.textWidth = { Double($0.text.utf16.count) * $0.size * 0.6 }
                        observed.nested = try document.render(inner)
                        observed.nestedRevision = document.revision
                        var latest = try document.snapshot().graph
                        try Self.setText(&latest, "title", "After nested"); try document.apply(latest)
                    }
                }
                return Double(request.text.utf16.count) * request.size * 0.6
            }}
            let first = try XCTUnwrap(row["first"] as? [String: Any])
            if mode.hasPrefix("throw-") {
                XCTAssertEqual(first["original"] as? Bool, true)
                XCTAssertThrowsError(try document.render(resources, responsive: responsive)) { XCTAssertTrue(($0 as AnyObject) === observed.failure) }
            } else { try compare(document.render(resources, responsive: responsive), first) }
            XCTAssertTrue(observed.fired)
            XCTAssertTrue(NSArray(array: observed.trace).isEqual(to: try XCTUnwrap(row["trace"] as? [[Any]])), "\(mode)/\(raw)/\(responsive) trace")
            let after = try document.snapshot()
            XCTAssertEqual(after.revision, document.revision)
            XCTAssertTrue(NSDictionary(dictionary: try state(after.graph)).isEqual(to: try XCTUnwrap(row["after"] as? [String: Any])), mode + " state")
            let latest = try Self.first(after.graph), identity = try XCTUnwrap(row["identity"] as? [String: Bool])
            XCTAssertEqual(latest.root == original.root, identity["root"])
            XCTAssertEqual(latest.category == original.category, identity["category"])
            if mode == "nested" {
                try compare(XCTUnwrap(observed.nested), XCTUnwrap(row["nested"] as? [String: Any]))
                XCTAssertGreaterThan(document.revision, try XCTUnwrap(observed.nestedRevision))
            }
            let recovery = IshikawaCallbackState(); var plain = pure()
            plain.textWidth = { request in MainActor.assumeIsolated {
                recovery.trace.append([request.text, String(format: "%016llx", request.size.bitPattern), Int(request.style)])
                return Double(request.text.utf16.count) * request.size * 0.6
            }}
            try compare(document.render(plain, responsive: responsive), XCTUnwrap(row["recovery"] as? [String: Any]))
            XCTAssertTrue(NSArray(array: recovery.trace).isEqual(to: try XCTUnwrap(row["recoveryTrace"] as? [[Any]])), mode + " recovery trace")
        }
    }
}
