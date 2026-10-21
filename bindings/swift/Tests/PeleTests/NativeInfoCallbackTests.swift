import Foundation
import XCTest
import Pele

private final class InfoCallbackFailure: Error {}

@MainActor private enum InfoCallbackDocument {
    case normal(Pele.MutableDocument)
    case prepared(Pele.PreparedDocument)

    var revision: UInt64 {
        switch self { case .normal(let d): d.revision; case .prepared(let d): d.revision }
    }
    func snapshot() throws -> Pele.PublicSnapshot {
        switch self { case .normal(let d): try d.snapshot(); case .prepared(let d): try d.snapshot() }
    }
    func apply(_ graph: Pele.PublicGraph) throws {
        switch self {
        case .normal(let d): _ = try d.apply(graph, expectedRevision: d.revision)
        case .prepared(let d): _ = try d.apply(graph, expectedRevision: d.revision)
        }
    }
    func render(_ resources: Pele.HostResources) throws -> FullRenderedDiagram {
        switch self { case .normal(let d): try d.render(resources: resources); case .prepared(let d): try d.render(resources: resources) }
    }
    func close() throws {
        switch self { case .normal(let d): try d.close(); case .prepared(let d): try d.close() }
    }
}

@MainActor private final class InfoCallbackState {
    var fired = false
    var closeBlocked = false
    var trace: [[String]] = []
    var nested: FullRenderedDiagram?
    var nestedRevision: UInt64?
    let failure = InfoCallbackFailure()
}

@MainActor final class NativeInfoCallbackTests: XCTestCase {
    private func resources() -> Pele.HostResources {
        var resources = Pele.HostResources()
        resources.textWidth = { Double($0.text.utf16.count) * $0.size * 0.6 }
        return resources
    }
    private func bits(_ value: Double) -> String { String(format: "%016llx", value.bitPattern) }
    private func compare(_ actual: FullRenderedDiagram, _ expected: [String: Any]) throws {
        XCTAssertEqual(actual.svg, try XCTUnwrap(expected["svg"] as? String))
        XCTAssertEqual(bits(actual.width), try XCTUnwrap(expected["width"] as? String))
        XCTAssertEqual(bits(actual.height), try XCTUnwrap(expected["height"] as? String))
        XCTAssertTrue(actual.links.isEmpty)
    }
    private func fixture() throws -> [[String: Any]] {
        var url = URL(fileURLWithPath: #filePath)
        for _ in 0..<5 { url.deleteLastPathComponent() }
        let data = try Data(contentsOf: url.appendingPathComponent("fixtures/public-info-live-render.json"))
        let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: data) as? [String: Any])
        XCTAssertEqual(fixture["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        return try XCTUnwrap(fixture["cases"] as? [[String: Any]])
    }

    func testInfoLiveCallbacksPreservePinnedOutputEditsOriginalErrorsAndNestedRendering() throws {
        let source = "info\ntitle Before\naccTitle: Before accessible\naccDescr: Before description"
        for prepared in [false, true] {
            for expected in try fixture() {
                let mode = try XCTUnwrap(expected["mode"] as? String)
                let engine = Pele.Engine()
                let document: InfoCallbackDocument = try prepared
                    ? .prepared(engine.parsePrepared("info", source: source))
                    : .normal(engine.parseMutable(source))
                defer { try? document.close() }
                let initial = try document.snapshot()
                guard case .ref(let root) = initial.graph.root else { return XCTFail("Info root missing") }
                let state = InfoCallbackState()
                var host = resources()
                host.textWidth = { request in try MainActor.assumeIsolated {
                    state.trace.append([request.text, String(format: "%016llx", request.size.bitPattern)])
                    if !state.fired && mode != "normal" && (mode != "version-mutate" || request.text == "Pele v1.6.0") {
                        state.fired = true
                        do { try document.close() }
                        catch { state.closeBlocked = (error as? PeleError)?.message == "Mutable document is already in use" }
                        var graph = try document.snapshot().graph
                        for (key, value) in [("title", "After"), ("accTitle", "After accessible"), ("accDescr", "After description")] {
                            let id = try graph.intern(value)
                            try graph.setField(key, value: .string(id))
                        }
                        try document.apply(graph)
                        if mode == "throw" { throw state.failure }
                        if mode == "nested" {
                            var inner = Pele.HostResources()
                            inner.textWidth = { Double($0.text.utf16.count) * $0.size * 0.6 }
                            state.nested = try document.render(inner)
                            state.nestedRevision = document.revision
                        }
                    }
                    return Double(request.text.utf16.count) * request.size * 0.6
                }}
                let result = try XCTUnwrap(expected["result"] as? [String: Any])
                if mode == "throw" {
                    XCTAssertEqual(result["original"] as? Bool, true)
                    XCTAssertThrowsError(try document.render(host)) { XCTAssertTrue(($0 as AnyObject) === state.failure) }
                } else {
                    XCTAssertEqual(result["ok"] as? Bool, true)
                    try compare(document.render(host), result)
                }
                XCTAssertEqual(state.trace, try XCTUnwrap(expected["trace"] as? [[String]]), "\(mode), prepared=\(prepared)")
                XCTAssertEqual(state.fired, mode != "normal")
                XCTAssertEqual(state.closeBlocked, mode != "normal")
                let snapshot = try document.snapshot()
                XCTAssertEqual(snapshot.revision, document.revision)
                guard case .ref(let afterRoot) = snapshot.graph.root else { return XCTFail("Info root lost") }
                XCTAssertEqual(afterRoot, root)
                let after = try XCTUnwrap(expected["after"] as? [String: Any])
                for key in ["title", "accTitle", "accDescr"] {
                    guard case .string(let id) = try snapshot.graph.field(key) else { return XCTFail("Info field lost: \(key)") }
                    XCTAssertEqual(snapshot.graph.strings[Int(id)].stringIfWellFormed, after[key] as? String)
                }
                if mode == "nested" {
                    try compare(XCTUnwrap(state.nested), XCTUnwrap(expected["nested"] as? [String: Any]))
                    XCTAssertGreaterThan(document.revision, try XCTUnwrap(state.nestedRevision))
                }
                try compare(document.render(resources()), XCTUnwrap(expected["recovery"] as? [String: Any]))
            }
        }
    }
}
