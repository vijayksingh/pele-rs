import Foundation
import XCTest
import Pele

private final class PieCallbackFailure: Error {}
@MainActor private enum PieCallbackDocument {
    case normal(Pele.MutableDocument), prepared(Pele.PreparedDocument)
    var revision: UInt64 { switch self { case .normal(let d): d.revision; case .prepared(let d): d.revision } }
    func snapshot() throws -> Pele.PublicSnapshot { switch self { case .normal(let d): try d.snapshot(); case .prepared(let d): try d.snapshot() } }
    func apply(_ graph: Pele.PublicGraph) throws {
        switch self { case .normal(let d): _ = try d.apply(graph, expectedRevision: d.revision); case .prepared(let d): _ = try d.apply(graph, expectedRevision: d.revision) }
    }
    func render(_ resources: Pele.HostResources, responsive: Bool = false) throws -> FullRenderedDiagram {
        var options = Pele.Engine.Options(); options.maxWidth = responsive ? 200 : nil
        switch self { case .normal(let d): return try d.render(options: options, resources: resources); case .prepared(let d): return try d.render(options: options, resources: resources) }
    }
    func close() throws { switch self { case .normal(let d): try d.close(); case .prepared(let d): try d.close() } }
}
@MainActor private final class PieCallbackState {
    var fired = false
    var trace: [[Any]] = []
    var nested: FullRenderedDiagram?
    var nestedRevision: UInt64?
    let failure = PieCallbackFailure()
}

@MainActor final class NativePieCallbackTests: XCTestCase {
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
    private static func sections(_ graph: Pele.PublicGraph) throws -> UInt32 {
        guard case .ref(let id) = try graph.field("sections") else { throw PeleError(code: "test", message: "Pie sections Map missing") }; return id
    }
    private func entries(_ graph: Pele.PublicGraph, id: UInt32, bitValues: Bool) throws -> [[Any]] {
        guard case .map(let values, _) = graph.nodes[Int(id)] else { throw PeleError(code: "test", message: "Pie Map missing") }
        return try values.map { entry in
            guard case .string(let key) = entry.key, case .number(let n) = entry.value else { throw PeleError(code: "test", message: "Pie Map entry malformed") }
            return [try XCTUnwrap(graph.strings[Int(key)].stringIfWellFormed), bitValues ? bits(n) as Any : n as Any]
        }
    }
    private static func setText(_ graph: inout Pele.PublicGraph, _ key: String, _ text: String) throws {
        let id = try graph.intern(text); try graph.setField(key, value: .string(id))
    }
    private func state(_ graph: Pele.PublicGraph) throws -> [String: Any] {
        var result: [String: Any] = ["sections": try entries(graph, id: Self.sections(graph), bitValues: true)]
        for key in ["title", "accTitle", "accDescr"] {
            if case .string(let id) = try graph.field(key) { result[key] = try XCTUnwrap(graph.strings[Int(id)].stringIfWellFormed) }
            else { result[key] = NSNull() }
        }
        guard case .bool(let show) = try graph.field("showData") else { throw PeleError(code: "test", message: "Pie showData missing") }
        result["showData"] = show; return result
    }
    func testPieMapCaptureLateReadsOriginalErrorsAndNestedRenderingMatchAllOracleRows() throws {
        var path = URL(fileURLWithPath: #filePath); for _ in 0..<5 { path.deleteLastPathComponent() }
        let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-pie-live-render.json"))) as? [String: Any])
        XCTAssertEqual(fixture["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        let source = try XCTUnwrap(fixture["source"] as? String), rows = try XCTUnwrap(fixture["rows"] as? [[String: Any]])
        XCTAssertEqual(rows.count, 36)
        for row in rows {
            let raw = try XCTUnwrap(row["raw"] as? Bool), responsive = try XCTUnwrap(row["responsive"] as? Bool), mode = try XCTUnwrap(row["mode"] as? String)
            let engine = Pele.Engine()
            let document: PieCallbackDocument = try raw ? .prepared(engine.parsePrepared("pie", source: source)) : .normal(engine.parseMutable(source))
            defer { try? document.close() }
            let originalMap = try Self.sections(document.snapshot().graph), observed = PieCallbackState()
            var resources = pure()
            resources.textWidth = { request in try MainActor.assumeIsolated {
                observed.trace.append([request.text, String(format: "%016llx", request.size.bitPattern), Int(request.style)])
                if !observed.fired {
                    observed.fired = true
                    XCTAssertThrowsError(try document.close())
                    var graph = try document.snapshot().graph
                    switch mode {
                    case "title": try Self.setText(&graph, "title", "Changed title")
                    case "showData": try graph.setField("showData", value: .bool(true))
                    case "accessibility":
                        try Self.setText(&graph, "accTitle", "Changed access"); try Self.setText(&graph, "accDescr", "Changed description")
                    case "map-edit":
                        let id = try Self.sections(graph), delta = try graph.intern("Delta")
                        guard case .map(let old, let properties) = graph.nodes[Int(id)] else { throw observed.failure }
                        var edited: [Pele.PublicGraph.Entry] = []
                        for entry in old {
                            guard case .string(let key) = entry.key else { throw observed.failure }
                            let name = graph.strings[Int(key)].stringIfWellFormed
                            if name != "Gamma" { edited.append(.init(key: entry.key, value: name == "Beta" ? .number(20) : entry.value)) }
                        }
                        edited.append(.init(key: .string(delta), value: .number(4)))
                        graph.nodes[Int(id)] = .map(entries: edited, properties: properties)
                    case "map-replace":
                        let replacement = try graph.intern("Replacement"), id = UInt32(graph.nodes.count)
                        graph.nodes.append(.map(entries: [.init(key: .string(replacement), value: .number(8))], properties: []))
                        try graph.setField("sections", value: .ref(id))
                    case "throw-object", "throw-undefined":
                        try Self.setText(&graph, "title", "Persisted before throw"); try graph.setField("showData", value: .bool(true))
                        let key = try graph.intern("Added"), id = try Self.sections(graph)
                        guard case .map(var entries, let properties) = graph.nodes[Int(id)] else { throw observed.failure }
                        entries.append(.init(key: .string(key), value: .number(7))); graph.nodes[Int(id)] = .map(entries: entries, properties: properties)
                    case "nested-render": try Self.setText(&graph, "title", "Nested title")
                    case "baseline": break
                    default: throw PeleError(code: "test", message: "Unknown Pie fixture mode")
                    }
                    if mode != "baseline" { try document.apply(graph) }
                    // Swift has Error values, not JS undefined throws. Both oracle
                    // throw rows verify Swift's exact original Error identity.
                    if mode.hasPrefix("throw-") { throw observed.failure }
                    if mode == "nested-render" {
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
            XCTAssertEqual(try Self.sections(after.graph) == originalMap, row["originalMapStillCurrent"] as? Bool)
            XCTAssertTrue(NSArray(array: try entries(after.graph, id: originalMap, bitValues: false)).isEqual(to: try XCTUnwrap(row["originalMap"] as? [[Any]])), mode + " retained Map")
            if mode == "nested-render" {
                try compare(XCTUnwrap(observed.nested), XCTUnwrap(row["nested"] as? [String: Any]))
                XCTAssertGreaterThan(document.revision, try XCTUnwrap(observed.nestedRevision))
            }
            let recovery = PieCallbackState(); var plain = pure()
            plain.textWidth = { request in MainActor.assumeIsolated {
                recovery.trace.append([request.text, String(format: "%016llx", request.size.bitPattern), Int(request.style)])
                return Double(request.text.utf16.count) * request.size * 0.6
            }}
            try compare(document.render(plain, responsive: responsive), XCTUnwrap(row["recovery"] as? [String: Any]))
            XCTAssertTrue(NSArray(array: recovery.trace).isEqual(to: try XCTUnwrap(row["recoveryTrace"] as? [[Any]])), mode + " recovery trace")
        }
    }
}
