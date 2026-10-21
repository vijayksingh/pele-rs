import Foundation
import XCTest
import Pele

private final class TreeViewCallbackFailure: Error {}
@MainActor private enum TreeViewCallbackDocument {
    case normal(Pele.MutableDocument), prepared(Pele.PreparedDocument)
    var revision: UInt64 { switch self { case .normal(let d): d.revision; case .prepared(let d): d.revision } }
    func snapshot() throws -> Pele.PublicSnapshot { switch self { case .normal(let d): try d.snapshot(); case .prepared(let d): try d.snapshot() } }
    func apply(_ graph: Pele.PublicGraph) throws { switch self { case .normal(let d): _ = try d.apply(graph, expectedRevision: d.revision); case .prepared(let d): _ = try d.apply(graph, expectedRevision: d.revision) } }
    func render(_ resources: Pele.HostResources, responsive: Bool) throws -> FullRenderedDiagram {
        var options = Pele.Engine.Options(); options.maxWidth = responsive ? 180 : nil
        switch self { case .normal(let d): return try d.render(options: options, resources: resources); case .prepared(let d): return try d.render(options: options, resources: resources) }
    }
    func close() throws { switch self { case .normal(let d): try d.close(); case .prepared(let d): try d.close() } }
}
@MainActor private final class TreeViewCallbackState {
    var fired = false
    var trace: [[Any]] = []
    var nested: FullRenderedDiagram?
    var nestedRevision: UInt64?
    let failure = TreeViewCallbackFailure()
}
@MainActor final class NativeTreeViewCallbackTests: XCTestCase {
    private static func bits(_ n: Double) -> String { String(format: "%016llx", n.bitPattern) }
    private static func ref(_ value: Pele.PublicGraph.Atom?) throws -> UInt32 {
        guard case .ref(let id) = value else { throw PeleError(code: "test", message: "Expected TreeView reference") }; return id
    }
    private static func nodes(_ graph: Pele.PublicGraph) throws -> (root: UInt32, folder: UInt32, alpha: UInt32, beta: UInt32) {
        let root = try ref(graph.field("root")), children = try ref(graph.field("children", on: root)), folder = try ref(graph.field("0", on: children)), list = try ref(graph.field("children", on: folder))
        return (root, folder, try ref(graph.field("0", on: list)), try ref(graph.field("1", on: list)))
    }
    private static func setText(_ graph: inout Pele.PublicGraph, _ key: String, _ value: String, on node: UInt32? = nil) throws {
        let id = try graph.intern(value); try graph.setField(key, value: .string(id), on: node)
    }
    private static func array(_ graph: inout Pele.PublicGraph, _ values: [Pele.PublicGraph.Atom]) throws -> UInt32 {
        let id = UInt32(graph.nodes.count); graph.nodes.append(.array(length: 0, properties: []))
        for (i, value) in values.enumerated() { try graph.setArrayElement(UInt32(i), value: value, on: id) }; return id
    }
    private static func fresh(_ graph: inout Pele.PublicGraph, _ name: String) throws -> UInt32 {
        let id = UInt32(graph.nodes.count); graph.nodes.append(.object(properties: []))
        try graph.setField("id", value: .number(9), on: id); try graph.setField("level", value: .number(4), on: id)
        try setText(&graph, "name", name, on: id); try setText(&graph, "nodeType", "file", on: id)
        let children = try array(&graph, []); try graph.setField("children", value: .ref(children), on: id); return id
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
            var object: [String: Any] = [:]
            for p in graph.nodes[Int(id)].properties { object[try XCTUnwrap(graph.strings[Int(p.key)].stringIfWellFormed)] = try encoded(graph, p.value) }; return object
        default: throw TreeViewCallbackFailure()
        }
    }
    private static func resources(_ state: TreeViewCallbackState? = nil) -> Pele.HostResources {
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
    private static func mutate(_ document: TreeViewCallbackDocument, _ state: TreeViewCallbackState, _ mode: String, _ responsive: Bool) throws {
        state.fired = true; XCTAssertThrowsError(try document.close())
        var graph = try document.snapshot().graph; let n = try nodes(graph)
        switch mode {
        case "baseline": break
        case "title": try setText(&graph, "title", "Changed title")
        case "current": try setText(&graph, "name", "Changed current", on: n.alpha); try setText(&graph, "nodeType", "directory", on: n.alpha); try setText(&graph, "cssClass", "highlight red", on: n.alpha)
        case "later": try setText(&graph, "name", "Changed later", on: n.beta); try setText(&graph, "icon", "none", on: n.beta)
        case "ids": let id = try XCTUnwrap(graph.field("id", on: n.beta)); try graph.setField("id", value: id, on: n.alpha); try graph.setField("id", value: id, on: n.folder)
        case "children-append": let children = try ref(graph.field("children", on: n.folder)), child = try fresh(&graph, "Appended"); try graph.setArrayElement(2, value: .ref(child), on: children)
        case "children-replace": let child = try fresh(&graph, "Replacement"), children = try array(&graph, [.ref(child)]); try graph.setField("children", value: .ref(children), on: n.folder)
        case "root-replace":
            let child = try fresh(&graph, "New root child"), children = try array(&graph, [.ref(child)]), newRoot = UInt32(graph.nodes.count)
            graph.nodes.append(graph.nodes[Int(n.root)]); try graph.setField("children", value: .ref(children), on: newRoot); try graph.setField("root", value: .ref(newRoot))
        case "description": try setText(&graph, "description", "Changed description", on: n.alpha); try setText(&graph, "description", "Later description", on: n.beta)
        case "icon-current": try setText(&graph, "name", "Icon current", on: n.folder); try setText(&graph, "nodeType", "file", on: n.folder); try setText(&graph, "cssClass", "highlight blue", on: n.folder); try setText(&graph, "description", "Icon description", on: n.folder)
        case "icon-later": try setText(&graph, "name", "Icon later", on: n.beta); try setText(&graph, "nodeType", "directory", on: n.beta); try setText(&graph, "description", "Icon later description", on: n.beta)
        case "accessibility": try setText(&graph, "accTitle", "Changed access"); try setText(&graph, "accDescr", "Changed description")
        case "throw-object", "throw-undefined", "icon-throw": try setText(&graph, "title", "Persisted title"); try setText(&graph, "name", "Persisted later", on: n.beta)
        case "nested-render": try setText(&graph, "title", "Nested title")
        default: throw TreeViewCallbackFailure()
        }
        if mode != "baseline" { try document.apply(graph) }
        // Swift has Error values, not JS undefined throws; all three rows retain
        // the actual original Swift Error identity through C15 cleanup.
        if mode.hasPrefix("throw-") || mode == "icon-throw" { throw state.failure }
        if mode == "nested-render" {
            state.nested = try document.render(resources(), responsive: responsive); state.nestedRevision = document.revision
            var latest = try document.snapshot().graph; try setText(&latest, "title", "After nested"); try document.apply(latest)
        }
    }
    func testTreeViewCapturedNodeIdentityLiveChildrenIconsAndErrorsMatch64OracleRows() throws {
        var path = URL(fileURLWithPath: #filePath); for _ in 0..<5 { path.deleteLastPathComponent() }
        let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-treeview-live-render.json"))) as? [String: Any])
        XCTAssertEqual(fixture["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        let rows = try XCTUnwrap(fixture["rows"] as? [[String: Any]]), source = try XCTUnwrap(fixture["source"] as? String); XCTAssertEqual(rows.count, 64)
        for row in rows {
            let raw = try XCTUnwrap(row["raw"] as? Bool), responsive = try XCTUnwrap(row["responsive"] as? Bool), mode = try XCTUnwrap(row["mode"] as? String), engine = Pele.Engine()
            let document: TreeViewCallbackDocument = try raw ? .prepared(engine.parsePrepared("treeView", source: source)) : .normal(engine.parseMutable(source)); defer { try? document.close() }
            let original = try Self.nodes(document.snapshot().graph), observed = TreeViewCallbackState(); var host = Self.resources()
            host.textWidth = { request in try MainActor.assumeIsolated {
                observed.trace.append(["width", request.text, Self.bits(request.size), Int(request.style)])
                let target = ["children-append", "children-replace"].contains(mode) ? "folder" : ["current", "later", "ids", "root-replace", "description"].contains(mode) ? "Alpha.txt" : "Initial title"
                if !observed.fired && !mode.hasPrefix("icon-") && request.text == target { try Self.mutate(document, observed, mode, responsive) }
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
            XCTAssertTrue(NSDictionary(dictionary: try XCTUnwrap(Self.encoded(after.graph, .ref(original.root)) as? [String: Any])).isEqual(to: try XCTUnwrap(row["capturedRoot"] as? [String: Any])), mode + " captured")
            XCTAssertEqual(try Self.ref(after.graph.field("root")) == original.root, row["rootStillCurrent"] as? Bool)
            if mode == "nested-render" { try compare(XCTUnwrap(observed.nested), XCTUnwrap(row["nested"] as? [String: Any])); XCTAssertGreaterThan(document.revision, try XCTUnwrap(observed.nestedRevision)) }
            let recovery = TreeViewCallbackState(); try compare(document.render(Self.resources(recovery), responsive: responsive), XCTUnwrap(row["recovery"] as? [String: Any]))
            XCTAssertTrue(NSArray(array: recovery.trace).isEqual(to: try XCTUnwrap(row["recoveryTrace"] as? [[Any]])), mode + " recovery trace")
        }
    }
}
