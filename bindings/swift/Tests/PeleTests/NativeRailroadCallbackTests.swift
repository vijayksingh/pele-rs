import Foundation
import XCTest
import Pele

private final class RailroadCallbackFailure: Error {}
@MainActor private enum RailroadCallbackDocument {
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
        var options = Pele.Engine.Options(); options.maxWidth = responsive ? 180 : nil
        switch self { case .normal(let d): return try d.render(options: options, resources: resources); case .prepared(let d): return try d.render(options: options, resources: resources) }
    }
    func close() throws { switch self { case .normal(let d): try d.close(); case .prepared(let d): try d.close() } }
}
@MainActor private final class RailroadCallbackState {
    var fired = false
    var trace: [[Any]] = []
    var nested: FullRenderedDiagram?
    var nestedRevision: UInt64?
    let failure = RailroadCallbackFailure()
}

@MainActor final class NativeRailroadCallbackTests: XCTestCase {
    private static func bits(_ n: Double) -> String { String(format: "%016llx", n.bitPattern) }
    private static func ref(_ a: Pele.PublicGraph.Atom?) throws -> UInt32 {
        guard case .ref(let id) = a else { throw RailroadCallbackFailure() }; return id
    }
    private struct Handles {
        let rules: UInt32, rule: UInt32, definition: UInt32, children: UInt32, first: UInt32, repetition: UInt32
    }
    private static func handles(_ g: Pele.PublicGraph) throws -> Handles {
        let rules = try ref(g.field("rules")), rule = try ref(g.field("0", on: rules)), definition = try ref(g.field("definition", on: rule))
        let children = try ref(g.field("elements", on: definition)), first = try ref(g.field("0", on: children)), repetition = try ref(g.field("2", on: children))
        return Handles(rules: rules, rule: rule, definition: definition, children: children, first: first, repetition: repetition)
    }
    private static func text(_ g: inout Pele.PublicGraph, _ value: String) throws -> Pele.PublicGraph.Atom { .string(try g.intern(value)) }
    private static func setText(_ g: inout Pele.PublicGraph, _ key: String, _ value: String, on node: UInt32? = nil) throws {
        let a = try text(&g, value); try g.setField(key, value: a, on: node)
    }
    private static func object(_ g: inout Pele.PublicGraph, _ fields: [(String, Pele.PublicGraph.Atom)]) throws -> UInt32 {
        let id = UInt32(g.nodes.count); g.nodes.append(.object(properties: []))
        for (key, value) in fields { try g.setField(key, value: value, on: id) }; return id
    }
    private static func terminal(_ g: inout Pele.PublicGraph, _ value: String) throws -> UInt32 {
        let type = try text(&g, "terminal"), value = try text(&g, value)
        return try object(&g, [("type", type), ("value", value)])
    }
    private static func rule(_ g: inout Pele.PublicGraph, _ name: String, _ value: String) throws -> UInt32 {
        let name = try text(&g, name), definition = try terminal(&g, value)
        return try object(&g, [("name", name), ("definition", .ref(definition))])
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
            var record: [String: Any] = [:]
            for p in node.properties {
                if case .undefined = p.value { continue }
                record[try XCTUnwrap(g.strings[Int(p.key)].stringIfWellFormed)] = try encoded(g, p.value)
            }
            return record
        case nil, .undefined, .null: return NSNull()
        default: throw RailroadCallbackFailure()
        }
    }
    private static func state(_ g: Pele.PublicGraph) throws -> [String: Any] {
        var out: [String: Any] = [:]
        for key in ["title", "accTitle", "accDescr", "rules"] { out[key] = try encoded(g, g.field(key)) }; return out
    }
    private static func resources(_ state: RailroadCallbackState? = nil) -> Pele.HostResources {
        var host = Pele.HostResources()
        host.textWidth = { request in MainActor.assumeIsolated {
            state?.trace.append([request.text, bits(request.size), Int(request.style)])
            return Double(request.text.utf16.count) * request.size * 0.6
        }}
        return host
    }
    private func compare(_ actual: FullRenderedDiagram, _ expected: [String: Any]) throws {
        XCTAssertEqual(actual.svg, try XCTUnwrap(expected["svg"] as? String)); XCTAssertEqual(Self.bits(actual.width), expected["width"] as? String); XCTAssertEqual(Self.bits(actual.height), expected["height"] as? String); XCTAssertTrue(actual.links.isEmpty)
    }
    private static func mutate(_ document: RailroadCallbackDocument, _ state: RailroadCallbackState, _ mode: String) throws {
        state.fired = true; XCTAssertThrowsError(try document.close())
        var g = try document.snapshot().graph; let h = try handles(g)
        switch mode {
        case "baseline": break
        case "title-accessibility": try setText(&g, "title", "Changed title"); try setText(&g, "accTitle", "Changed access"); try setText(&g, "accDescr", "Changed description")
        case "rules-replace": let r = try rule(&g, "Replacement", "new"), a = try array(&g, [.ref(r)]); try g.setField("rules", value: .ref(a))
        case "rule-name": try setText(&g, "name", "Changed entry", on: h.rule)
        case "definition-replace": let n = try terminal(&g, "Replacement definition"); try g.setField("definition", value: .ref(n), on: h.rule)
        case "element-text": try setText(&g, "value", "After captured Alpha", on: h.first); let second = try ref(g.field("1", on: h.children)); try setText(&g, "name", "Changed Other", on: second)
        case "children-replace": let n = try terminal(&g, "Replacement child"), a = try array(&g, [.ref(n)]); try g.setField("elements", value: .ref(a), on: h.definition)
        case "children-append": let n = try terminal(&g, "Appended child"); try g.setArrayElement(3, value: .ref(n), on: h.children)
        case "type-change": try setText(&g, "type", "choice", on: h.definition); try g.setField("alternatives", value: .ref(h.children), on: h.definition)
        case "repetition": try g.setField("min", value: .number(2), on: h.repetition); try g.setField("max", value: .number(4), on: h.repetition)
        case "rules-append": let n = try rule(&g, "Appended", "new"); try g.setArrayElement(2, value: .ref(n), on: h.rules)
        case "shared-identity": try setText(&g, "value", "Second occurrence", on: h.first)
        case "throw": try setText(&g, "title", "Before throw"); try setText(&g, "value", "Saved Alpha", on: h.first)
        case "nested": try setText(&g, "title", "Nested title")
        default: throw RailroadCallbackFailure()
        }
        if mode != "baseline" { try document.apply(g) }
        if mode == "throw" { throw state.failure }
        if mode == "nested" {
            state.nested = try document.render(resources()); state.nestedRevision = document.revision
            var latest = try document.snapshot().graph; try setText(&latest, "title", "After nested"); try document.apply(latest)
        }
    }
    func testRailroadRetainedGrammarOccurrencesFoldingErrorsMatch56OracleRows() throws {
        var path = URL(fileURLWithPath: #filePath); for _ in 0..<5 { path.deleteLastPathComponent() }
        let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-railroad-live-render.json"))) as? [String: Any])
        XCTAssertEqual(fixture["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        let source = try XCTUnwrap(fixture["source"] as? String), rows = try XCTUnwrap(fixture["rows"] as? [[String: Any]]); XCTAssertEqual(rows.count, 56)
        for row in rows {
            let raw = try XCTUnwrap(row["raw"] as? Bool), responsive = try XCTUnwrap(row["responsive"] as? Bool), mode = try XCTUnwrap(row["mode"] as? String), engine = Pele.Engine()
            let document: RailroadCallbackDocument = try raw ? .prepared(engine.parsePrepared("railroad", source: source)) : .normal(engine.parseMutable(source)); defer { try? document.close() }
            var initial = try document.snapshot().graph; try Self.setText(&initial, "title", "Initial")
            if mode == "shared-identity" { let h = try Self.handles(initial); try initial.setArrayElement(1, value: .ref(h.first), on: h.children) }
            try document.apply(initial)
            let original = try Self.handles(document.snapshot().graph), observed = RailroadCallbackState(); var host = Self.resources()
            host.textWidth = { request in try MainActor.assumeIsolated {
                observed.trace.append([request.text, Self.bits(request.size), Int(request.style)])
                let target = mode == "repetition" ? "Beta" : mode == "rules-append" ? "entry" : ["title-accessibility", "nested", "baseline"].contains(mode) ? "Initial" : "Alpha"
                if !observed.fired && request.text == target { try Self.mutate(document, observed, mode) }
                return Double(request.text.utf16.count) * request.size * 0.6
            }}
            let first = try XCTUnwrap(row["first"] as? [String: Any])
            if mode == "throw" { XCTAssertEqual(first["original"] as? Bool, true); XCTAssertThrowsError(try document.render(host, responsive: responsive)) { XCTAssertTrue(($0 as AnyObject) === observed.failure) } }
            else { try compare(document.render(host, responsive: responsive), first) }
            XCTAssertTrue(observed.fired); XCTAssertTrue(NSArray(array: observed.trace).isEqual(to: try XCTUnwrap(row["trace"] as? [[Any]])), "\(mode)/\(raw)/\(responsive) trace")
            let after = try document.snapshot(); XCTAssertEqual(after.revision, document.revision)
            XCTAssertTrue(NSDictionary(dictionary: try Self.state(after.graph)).isEqual(to: try XCTUnwrap(row["after"] as? [String: Any])), mode + " state")
            for (name, id) in [("originalRules", original.rules), ("originalRule", original.rule), ("originalDefinition", original.definition), ("originalChildren", original.children), ("originalFirst", original.first)] {
                let actual = try Self.encoded(after.graph, .ref(id)), expected = try XCTUnwrap(row[name])
                XCTAssertTrue(NSArray(array: [actual]).isEqual(to: [expected]), mode + " " + name)
            }
            let currentRules = try Self.ref(after.graph.field("rules")), currentRule = try Self.ref(after.graph.field("0", on: currentRules)), currentDefinition = try Self.ref(after.graph.field("definition", on: currentRule))
            XCTAssertEqual(original.rules == currentRules, row["rulesCurrent"] as? Bool); XCTAssertEqual(original.rule == currentRule, row["ruleCurrent"] as? Bool); XCTAssertEqual(original.definition == currentDefinition, row["definitionCurrent"] as? Bool)
            let currentChildren = try after.graph.field("elements", on: currentDefinition)
            if case .ref(let id) = currentChildren { XCTAssertEqual(original.children == id, row["childrenCurrent"] as? Bool) } else { XCTAssertEqual(row["childrenCurrent"] as? Bool, false) }
            if mode == "nested" { try compare(XCTUnwrap(observed.nested), XCTUnwrap(row["nested"] as? [String: Any])); XCTAssertGreaterThan(document.revision, try XCTUnwrap(observed.nestedRevision)) }
            let recovery = RailroadCallbackState(); try compare(document.render(Self.resources(recovery), responsive: responsive), XCTUnwrap(row["recovery"] as? [String: Any]))
            XCTAssertTrue(NSArray(array: recovery.trace).isEqual(to: try XCTUnwrap(row["recoveryTrace"] as? [[Any]])), mode + " recovery trace")
        }
    }
}
