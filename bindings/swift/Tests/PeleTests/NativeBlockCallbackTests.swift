import Foundation
import XCTest
import Pele

private final class BlockCallbackFailure: Error {}
@MainActor private enum BlockCallbackDocument {
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
@MainActor private final class BlockCallbackState {
    var fired = false
    var trace: [[Any]] = []
    var nested: FullRenderedDiagram?
    var nestedRevision: UInt64?
    let failure = BlockCallbackFailure()
}

@MainActor final class NativeBlockCallbackTests: XCTestCase {
    private static func bits(_ n: Double) -> String { String(format: "%016llx", n.bitPattern) }
    private static func ref(_ a: Pele.PublicGraph.Atom?) throws -> UInt32 {
        guard case .ref(let id) = a else { throw BlockCallbackFailure() }; return id
    }
    private struct Handles {
        let root: UInt32, rootChildren: UInt32, group: UInt32, children: UInt32, a: UInt32, b: UInt32, arrow: UInt32, edges: UInt32, edge: UInt32
    }
    private static func named(_ g: Pele.PublicGraph, _ array: UInt32, _ name: String) throws -> UInt32 {
        guard case .array(let n, _) = g.nodes[Int(array)] else { throw BlockCallbackFailure() }
        for i in 0..<n { let node = try ref(g.field(String(i), on: array)); if case .string(let key) = try g.field("id", on: node), g.strings[Int(key)].stringIfWellFormed == name { return node } }
        throw BlockCallbackFailure()
    }
    private static func handles(_ g: Pele.PublicGraph) throws -> Handles {
        let root = try ref(g.field("root")), rootChildren = try ref(g.field("children", on: root)), group = try named(g, rootChildren, "g"), children = try ref(g.field("children", on: group))
        let a = try ref(g.field("0", on: children)), b = try ref(g.field("1", on: children)), arrow = try named(g, rootChildren, "c"), edges = try ref(g.field("edges")), edge = try ref(g.field("0", on: edges))
        return Handles(root: root, rootChildren: rootChildren, group: group, children: children, a: a, b: b, arrow: arrow, edges: edges, edge: edge)
    }
    private static func lookup(_ g: Pele.PublicGraph, _ map: UInt32, _ key: String) throws -> UInt32 {
        guard case .map(let entries, _) = g.nodes[Int(map)] else { throw BlockCallbackFailure() }
        for e in entries { if case .string(let i) = e.key, g.strings[Int(i)].stringIfWellFormed == key { return try ref(e.value) } }; throw BlockCallbackFailure()
    }
    private static func includes(_ g: Pele.PublicGraph, _ array: UInt32, _ id: UInt32) throws -> Bool {
        guard case .array(let count, _) = g.nodes[Int(array)] else { throw BlockCallbackFailure() }
        for i in 0..<count { if case .ref(let found) = try g.field(String(i), on: array), found == id { return true } }; return false
    }
    private static func text(_ g: inout Pele.PublicGraph, _ value: String) throws -> Pele.PublicGraph.Atom { .string(try g.intern(value)) }
    private static func setText(_ g: inout Pele.PublicGraph, _ key: String, _ value: String, on node: UInt32? = nil) throws {
        let a = try text(&g, value); try g.setField(key, value: a, on: node)
    }
    private static func object(_ g: inout Pele.PublicGraph, _ fields: [(String, Pele.PublicGraph.Atom)]) throws -> UInt32 {
        let id = UInt32(g.nodes.count); g.nodes.append(.object(properties: []))
        for (key, value) in fields { try g.setField(key, value: value, on: id) }; return id
    }
    private static func leaf(_ g: inout Pele.PublicGraph, _ name: String, _ label: String) throws -> UInt32 {
        let id = try text(&g, name), type = try text(&g, "square"), label = try text(&g, label)
        return try object(&g, [("id", id), ("type", type), ("label", label)])
    }
    private static func array(_ g: inout Pele.PublicGraph, _ values: [Pele.PublicGraph.Atom]) throws -> UInt32 {
        let id = UInt32(g.nodes.count); g.nodes.append(.array(length: 0, properties: []))
        for (i, a) in values.enumerated() { try g.setArrayElement(UInt32(i), value: a, on: id) }; return id
    }
    // Fixture uses JSON.stringify on acyclic grammar records. Undefined object
    // fields are omitted; array holes/undefined become null, as in the oracle.
    private static func encoded(_ g: Pele.PublicGraph, _ a: Pele.PublicGraph.Atom?) throws -> Any {
        switch a {
        case .string(let id): return try XCTUnwrap(g.strings[Int(id)].stringIfWellFormed)
        case .number(let n): return n
        case .bool(let b): return b
        case .ref(let id):
            let node = g.nodes[Int(id)]
            if case .array(let count, _) = node { return try (0..<count).map { try encoded(g, g.field(String($0), on: id)) } }
            if case .map(let entries, _) = node { return try entries.map { [try encoded(g, $0.key), try encoded(g, $0.value)] } }
            var record: [String: Any] = [:]
            for p in node.properties {
                if case .undefined = p.value { continue }
                record[try XCTUnwrap(g.strings[Int(p.key)].stringIfWellFormed)] = try encoded(g, p.value)
            }
            return record
        case nil, .undefined, .null: return NSNull()
        default: throw BlockCallbackFailure()
        }
    }
    private static func state(_ g: Pele.PublicGraph) throws -> [String: Any] {
        var out: [String: Any] = [:]
        for key in ["title", "accTitle", "accDescr", "root", "edges", "classes"] { out[key] = try encoded(g, g.field(key)) }; return out
    }
    private static func resources(_ state: BlockCallbackState? = nil) -> Pele.HostResources {
        var host = Pele.HostResources()
        host.textWidth = { request in MainActor.assumeIsolated {
            state?.trace.append(["width", request.text, bits(request.size), Int(request.style)])
            return Double(request.text.utf16.count) * request.size * 0.6
        }}
        return host
    }
    private func compare(_ actual: FullRenderedDiagram, _ expected: [String: Any]) throws {
        XCTAssertEqual(actual.svg, try XCTUnwrap(expected["svg"] as? String)); XCTAssertEqual(Self.bits(actual.width), expected["width"] as? String); XCTAssertEqual(Self.bits(actual.height), expected["height"] as? String); XCTAssertTrue(actual.links.isEmpty)
    }
    private static func mutate(_ document: BlockCallbackDocument, _ state: BlockCallbackState, _ mode: String) throws {
        state.fired = true; XCTAssertThrowsError(try document.close())
        var g = try document.snapshot().graph; let h = try handles(g)
        switch mode {
        case "baseline": break
        case "title-accessibility": try setText(&g, "title", "Changed title"); try setText(&g, "accTitle", "Changed access"); try setText(&g, "accDescr", "Changed description")
        case "children-replace": let n = try leaf(&g, "replacement", "Replacement"), a = try array(&g, [.ref(n)]); try g.setField("children", value: .ref(a), on: h.group)
        case "children-append": let n = try leaf(&g, "appended", "Appended"); try g.setArrayElement(2, value: .ref(n), on: h.children)
        case "current-node": try setText(&g, "label", "Changed current", on: h.a); try setText(&g, "id", "renamed-a", on: h.a); try setText(&g, "type", "diamond", on: h.a)
        case "later-node": try setText(&g, "label", "Changed later", on: h.b)
        case "grid": try g.setField("columns", value: .number(1), on: h.group); try g.setField("widthInColumns", value: .number(1), on: h.group)
        case "arrow": let value = try text(&g, "down"), a = try array(&g, [value]); try g.setField("directions", value: .ref(a), on: h.arrow)
        case "edge-label": try setText(&g, "label", "Changed edge", on: h.edge); try setText(&g, "id", "Changed edge ID", on: h.edge); try setText(&g, "arrowTypeEnd", "circle", on: h.edge)
        case "edges-replace": let a = try array(&g, []); try g.setField("edges", value: .ref(a))
        case "style":
            let classes = try ref(g.field("classes")), value = try lookup(g, classes, "hot")
            let a = try text(&g, "fill:#123456"), b = try text(&g, "color:#654321"), styles = try array(&g, [a,b]); try g.setField("styles", value: .ref(styles), on: value)
            let hot = try text(&g, "hot"), names = try array(&g, [hot]); try g.setField("classes", value: .ref(names), on: h.b)
        case "throw-object", "throw-undefined": try setText(&g, "title", "Before failure"); try setText(&g, "label", "Saved Alpha", on: h.a)
        case "nested": try setText(&g, "title", "Nested title")
        default: throw BlockCallbackFailure()
        }
        if mode != "baseline" { try document.apply(g) }
        if mode.hasPrefix("throw-") { throw state.failure }
        if mode == "nested" {
            state.nested = try document.render(resources()); state.nestedRevision = document.revision
            var latest = try document.snapshot().graph; try setText(&latest, "title", "After nested"); try document.apply(latest)
        }
    }
    func testBlockGridChildrenStylesEdgesCallbacksMatch56OracleRows() throws {
        var path = URL(fileURLWithPath: #filePath); for _ in 0..<5 { path.deleteLastPathComponent() }
        let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-block-live-render.json"))) as? [String: Any])
        XCTAssertEqual(fixture["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        let source = try XCTUnwrap(fixture["source"] as? String), rows = try XCTUnwrap(fixture["rows"] as? [[String: Any]]); XCTAssertEqual(rows.count, 56)
        for row in rows {
            let raw = try XCTUnwrap(row["raw"] as? Bool), responsive = try XCTUnwrap(row["responsive"] as? Bool), mode = try XCTUnwrap(row["mode"] as? String), engine = Pele.Engine()
            let document: BlockCallbackDocument = try raw ? .prepared(engine.parsePrepared("block", source: source)) : .normal(engine.parseMutable(source)); defer { try? document.close() }
            var initial = try document.snapshot().graph; try Self.setText(&initial, "title", "Initial title")
            try document.apply(initial)
            let original = try Self.handles(document.snapshot().graph), observed = BlockCallbackState(); var host = Self.resources()
            host.textWidth = { request in try MainActor.assumeIsolated {
                observed.trace.append(["width", request.text, Self.bits(request.size), Int(request.style)])
                let target = mode == "title-accessibility" ? "Initial title" : mode == "arrow" ? "Arrow" : mode == "edge-label" ? "Flow" : ["baseline", "children-replace", "grid", "nested"].contains(mode) ? "Group" : "Alpha"
                if !observed.fired && request.text == target { try Self.mutate(document, observed, mode) }
                return Double(request.text.utf16.count) * request.size * 0.6
            }}
            let first = try XCTUnwrap(row["first"] as? [String: Any])
            if mode.hasPrefix("throw-") { XCTAssertEqual(first["original"] as? Bool, true); XCTAssertThrowsError(try document.render(host, responsive: responsive)) { XCTAssertTrue(($0 as AnyObject) === observed.failure) } }
            else { try compare(document.render(host, responsive: responsive), first) }
            XCTAssertTrue(observed.fired); XCTAssertTrue(NSArray(array: observed.trace).isEqual(to: try XCTUnwrap(row["trace"] as? [[Any]])), "\(mode)/\(raw)/\(responsive) trace")
            let after = try document.snapshot(); XCTAssertEqual(after.revision, document.revision)
            XCTAssertTrue(NSDictionary(dictionary: try Self.state(after.graph)).isEqual(to: try XCTUnwrap(row["after"] as? [String: Any])), mode + " state")
            let originalReport = try XCTUnwrap(row["original"] as? [String: Any])
            for (name, id) in [("root", original.root), ("rootChildren", original.rootChildren), ("group", original.group), ("children", original.children), ("a", original.a), ("b", original.b), ("arrow", original.arrow), ("edges", original.edges), ("edge", original.edge)] {
                let actual = try Self.encoded(after.graph, .ref(id)), expected = try XCTUnwrap(originalReport[name])
                XCTAssertTrue(NSArray(array: [actual]).isEqual(to: [expected]), mode + " " + name)
            }
            let root = try Self.ref(after.graph.field("root")), rc = try Self.ref(after.graph.field("children", on: root)), children = try Self.ref(after.graph.field("children", on: original.group)), edges = try Self.ref(after.graph.field("edges"))
            let identity: [String: Bool] = ["root": root == original.root, "rootChildren": rc == original.rootChildren, "group": try Self.includes(after.graph, rc, original.group), "children": children == original.children, "a": try Self.includes(after.graph, children, original.a), "edges": edges == original.edges, "edge": try Self.includes(after.graph, edges, original.edge)]
            XCTAssertEqual(identity, try XCTUnwrap(row["identity"] as? [String: Bool]), mode + " identity")
            if mode == "nested" { try compare(XCTUnwrap(observed.nested), XCTUnwrap(row["nested"] as? [String: Any])); XCTAssertGreaterThan(document.revision, try XCTUnwrap(observed.nestedRevision)) }
            let recovery = BlockCallbackState(); try compare(document.render(Self.resources(recovery), responsive: responsive), XCTUnwrap(row["recovery"] as? [String: Any]))
            XCTAssertTrue(NSArray(array: recovery.trace).isEqual(to: try XCTUnwrap(row["recoveryTrace"] as? [[Any]])), mode + " recovery trace")
        }
    }
}
