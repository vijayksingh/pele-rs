import Foundation
import XCTest
import Pele

private final class ChartResourceFailure: Error {}
@MainActor private enum ChartCallbackDocument {
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
@MainActor private final class ChartCallbackState {
    var fired = false
    var closeBlocked = false
    var trace: [[Any]] = []
    var nested: FullRenderedDiagram?
    var nestedRevision: UInt64?
    let failure = ChartResourceFailure()
}
@MainActor final class ChartCallbackOwnershipTests: XCTestCase {
    private static func setText(_ graph: inout Pele.PublicGraph, _ key: String, _ text: String) throws {
        let id = try graph.intern(text); try graph.setField(key, value: .string(id))
    }
    private func pure() -> Pele.HostResources {
        var resources = Pele.HostResources(); resources.textWidth = { Double($0.text.utf16.count) * $0.size * 0.6 }; return resources
    }
    private func compare(_ drawing: FullRenderedDiagram, _ expected: [String: Any], _ id: String) throws {
        XCTAssertEqual(drawing.svg, try XCTUnwrap(expected["svg"] as? String), id)
        XCTAssertEqual(String(format: "%016llx", drawing.width.bitPattern), expected["width"] as? String, id)
        XCTAssertEqual(String(format: "%016llx", drawing.height.bitPattern), expected["height"] as? String, id)
        XCTAssertTrue(drawing.links.isEmpty, id)
    }
    func testPacketRadarAndQuadrantCallbacksRetainEditsOriginalErrorsAndNestedRendering() throws {
        var path = URL(fileURLWithPath: #filePath); for _ in 0..<5 { path.deleteLastPathComponent() }
        let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-chart-callback-ownership.json"))) as? [String: Any])
        XCTAssertEqual(fixture["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        let rows = try XCTUnwrap(fixture["rows"] as? [[String: Any]])
        XCTAssertEqual(rows.count, 36)
        for row in rows {
            let family = try XCTUnwrap(row["family"] as? String), source = try XCTUnwrap(row["source"] as? String)
            let raw = try XCTUnwrap(row["raw"] as? Bool), responsive = try XCTUnwrap(row["responsive"] as? Bool), mode = try XCTUnwrap(row["mode"] as? String)
            let id = "\(family)/\(raw)/\(responsive)/\(mode)"
            let engine = Pele.Engine(), state = ChartCallbackState()
            let document: ChartCallbackDocument = try raw ? .prepared(engine.parsePrepared(family, source: source)) : .normal(engine.parseMutable(source))
            defer { try? document.close() }
            let originalRoot = try document.snapshot().graph.root
            var resources = pure()
            resources.textWidth = { request in try MainActor.assumeIsolated {
                state.trace.append([request.text, String(format: "%016llx", request.size.bitPattern), Int(request.style)])
                if !state.fired {
                    state.fired = true
                    do { try document.close() } catch { state.closeBlocked = (error as? PeleError)?.message == "Mutable document is already in use" }
                    var graph = try document.snapshot().graph
                    for (key, text) in [("title", "Changed title"), ("accTitle", "Changed access"), ("accDescr", "Changed description")] {
                        try Self.setText(&graph, key, text)
                    }
                    try document.apply(graph)
                    if mode == "throw" { throw state.failure }
                    if mode == "nested" {
                        var inner = Pele.HostResources(); inner.textWidth = { Double($0.text.utf16.count) * $0.size * 0.6 }
                        state.nested = try document.render(inner); state.nestedRevision = document.revision
                        var latest = try document.snapshot().graph
                        try Self.setText(&latest, "title", "After nested"); try document.apply(latest)
                    }
                }
                return Double(request.text.utf16.count) * request.size * 0.6
            }}
            let expected = try XCTUnwrap(row["first"] as? [String: Any])
            if mode == "throw" {
                XCTAssertEqual(expected["original"] as? Bool, true)
                XCTAssertThrowsError(try document.render(resources, responsive: responsive)) { XCTAssertTrue(($0 as AnyObject) === state.failure, id) }
            } else { try compare(document.render(resources, responsive: responsive), expected, id) }
            XCTAssertTrue(state.fired, id); XCTAssertTrue(state.closeBlocked, id)
            XCTAssertTrue(NSArray(array: state.trace).isEqual(to: try XCTUnwrap(row["trace"] as? [[Any]])), id)
            let after = try document.snapshot()
            XCTAssertEqual(after.revision, document.revision, id)
            if case .ref(let before) = originalRoot, case .ref(let current) = after.graph.root { XCTAssertEqual(before, current, id) }
            else { XCTFail(id + " lost root identity") }
            for key in ["title", "accTitle", "accDescr"] {
                guard case .string(let value) = try after.graph.field(key) else { return XCTFail(id + " lost " + key) }
                XCTAssertEqual(after.graph.strings[Int(value)].stringIfWellFormed, row[key] as? String, id)
            }
            if mode == "nested" {
                try compare(XCTUnwrap(state.nested), XCTUnwrap(row["nested"] as? [String: Any]), id + "/nested")
                XCTAssertGreaterThan(document.revision, try XCTUnwrap(state.nestedRevision), id)
            }
            try compare(document.render(pure(), responsive: responsive), XCTUnwrap(row["recovery"] as? [String: Any]), id + "/recovery")
        }
    }
}
