import AppKit
import Foundation
import Pele
import XCTest

private final class PresentedCallbackFailure: Error {}
@MainActor private final class PresentedCallbackState {
    let document: Pele.MutableDocument
    init(_ document: Pele.MutableDocument) { self.document = document }
    var fired = false
    var trace: [[String]] = []
    var nested: FullRenderedDiagram?
    let failure = PresentedCallbackFailure()
}

@MainActor final class NativePresentationCallbackTests: XCTestCase {
    private static func host() -> Pele.HostResources {
        var host = Pele.HostResources()
        host.textWidth = { Double($0.text.utf16.count) * $0.size * 0.6 }
        host.timeZone = TimeZone(secondsFromGMT: 0)!
        return host
    }
    private static func set(_ graph: inout Pele.PublicGraph, _ key: String, _ text: String) throws {
        let value = try graph.intern(text)
        try graph.setField(key, value: .string(value))
    }
    private func compare(_ drawing: FullRenderedDiagram, _ expected: [String: Any]) throws {
        XCTAssertEqual(drawing.svg, expected["svg"] as? String)
        XCTAssertEqual(String(format: "%016llx", drawing.width.bitPattern), expected["width"] as? String)
        XCTAssertEqual(String(format: "%016llx", drawing.height.bitPattern), expected["height"] as? String)
        XCTAssertTrue(drawing.links.isEmpty)
    }
    func testMutablePresentationUsesPinnedInfoPhasesAndRetainsCallbackEditsOnOriginalThrow() throws {
        var url = URL(fileURLWithPath: #filePath)
        for _ in 0..<5 { url.deleteLastPathComponent() }
        let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf:
            url.appendingPathComponent("fixtures/public-info-live-render.json"))) as? [String: Any])
        XCTAssertEqual(fixture["oracleSHA256"] as? String,
            "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        let rows = try XCTUnwrap(fixture["cases"] as? [[String: Any]])
        XCTAssertEqual(rows.count, 5)
        for row in rows {
            let mode = try XCTUnwrap(row["mode"] as? String)
            let document = try Pele.Engine().parseMutable("info\ntitle Before\naccTitle: Before accessible\naccDescr: Before description")
            defer { try? document.close() }
            let state = PresentedCallbackState(document)
            var resources = Self.host()
            resources.textWidth = { request in try MainActor.assumeIsolated {
                let document = state.document
                state.trace.append([request.text, String(format: "%016llx", request.size.bitPattern)])
                if !state.fired && mode != "normal" && (mode != "version-mutate" || request.text == "Pele v1.6.0") {
                    state.fired = true
                    XCTAssertThrowsError(try document.close())
                    var graph = try document.snapshot().graph
                    for (key, text) in [("title", "After"), ("accTitle", "After accessible"), ("accDescr", "After description")] {
                        try Self.set(&graph, key, text)
                    }
                    try document.apply(graph, expectedRevision: document.revision)
                    if mode == "throw" { throw state.failure }
                    if mode == "nested" {
                        state.nested = try document.renderPresentation(resources: Self.host())
                    }
                }
                return Double(request.text.utf16.count) * request.size * 0.6
            }}
            if mode == "throw" {
                XCTAssertThrowsError(try document.renderPresentation(resources: resources)) {
                    XCTAssertTrue(($0 as AnyObject) === state.failure)
                }
            } else {
                let drawing = try document.renderPresentation(resources: resources)
                try compare(drawing, XCTUnwrap(row["result"] as? [String: Any]))
                guard case .unavailable(.unsupportedDiagram) = drawing.presentation else {
                    return XCTFail("Info must retain explicit presentation unavailability")
                }
            }
            XCTAssertEqual(state.fired, mode != "normal")
            XCTAssertEqual(state.trace, row["trace"] as? [[String]])
            let snapshot = try document.snapshot()
            XCTAssertEqual(document.revision, snapshot.revision)
            for key in ["title", "accTitle", "accDescr"] {
                guard case .string(let id) = try snapshot.graph.field(key) else { return XCTFail("Missing field") }
                XCTAssertEqual(snapshot.graph.strings[Int(id)].stringIfWellFormed,
                    (row["after"] as? [String: Any])?[key] as? String)
            }
            if mode == "nested" { try compare(XCTUnwrap(state.nested), XCTUnwrap(row["nested"] as? [String: Any])) }
            try compare(document.renderPresentation(resources: Self.host()), XCTUnwrap(row["recovery"] as? [String: Any]))
        }
    }
    private func png() throws -> Data {
        let bitmap = try XCTUnwrap(NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: 1, pixelsHigh: 1,
            bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
            colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0))
        let bytes = try XCTUnwrap(bitmap.bitmapData)
        bytes[0] = 255; bytes[1] = 0; bytes[2] = 0; bytes[3] = 255
        return try XCTUnwrap(bitmap.representation(using: .png, properties: [:]))
    }
    func testNativeImageApprovalAllowsNestedRenderAndPublishesAccessibleStateOnSuccessOrThrow() throws {
        let data = try png()
        for fail in [false, true] {
            let document = try Pele.Engine().parseMutable("flowchart LR\naccTitle: Before\nA@{img: \"https://example.test/picture.png\",label: \"Picture\",w:80,h:60} --> B[Done]")
            defer { try? document.close() }
            let state = PresentedCallbackState(document)
            let images = Pele.NativeResources { uri in
                XCTAssertEqual(uri, "https://example.test/picture.png")
                XCTAssertFalse(state.fired)
                state.fired = true
                XCTAssertThrowsError(try document.close())
                var graph = try document.snapshot().graph
                try Self.set(&graph, "accTitle", "Saved accessible")
                try Self.set(&graph, "title", "Saved title")
                try document.apply(graph, expectedRevision: document.revision)
                var nestedOptions = Pele.Engine.Options(); nestedOptions.images = false
                state.nested = try document.renderPresentation(options: nestedOptions, resources: Self.host())
                if fail { throw state.failure }
                return data
            }
            if fail {
                XCTAssertThrowsError(try document.renderNative(nativeResources: images, resources: Self.host())) {
                    XCTAssertTrue(($0 as AnyObject) === state.failure)
                }
            } else {
                let drawing = try document.renderNative(nativeResources: images, resources: Self.host())
                XCTAssertTrue(drawing.svg.contains("data:image/png;base64,"))
                guard case .available(let frame) = drawing.presentation else { return XCTFail("Missing native frame") }
                XCTAssertFalse(frame.text.isEmpty)
                XCTAssertEqual(frame.width, drawing.width)
            }
            XCTAssertTrue(state.fired)
            XCTAssertNotNil(state.nested)
            let current = try document.snapshot()
            XCTAssertEqual(current.revision, document.revision)
            guard case .string(let id) = try current.graph.field("title") else { return XCTFail("Lost callback edit") }
            XCTAssertEqual(current.graph.strings[Int(id)].stringIfWellFormed, "Saved title")
            images.imageLoader = { _ in data }; images.invalidate()
            let recovery = try document.renderNative(nativeResources: images, resources: Self.host())
            XCTAssertTrue(recovery.svg.contains("data:image/png;base64,"))
            guard case .available(let frame) = recovery.presentation else { return XCTFail("Missing recovery frame") }
            XCTAssertEqual(frame.title, "Saved accessible")
            XCTAssertTrue(recovery.svg.contains("Saved title"))
        }
    }
    func testParsedGanttInvalidDateDispatchesCalendarOutsideOwnerBorrowAndRetainsOriginalError() throws {
        let document = try Pele.Engine().parseMutable("gantt\ndateFormat YYYY-MM-DD\nexcludes weekends\nAlpha :a,2024-01-01,1d")
        defer { try? document.close() }
        var graph = try document.snapshot().graph
        let date = UInt32(graph.nodes.count)
        graph.nodes.append(.date(milliseconds: 1_704_585_600_000, properties: []))
        try document.apply(graph, expectedRevision: document.revision)
        let state = PresentedCallbackState(document)
        var resources = Self.host()
        resources.calendarFields = { _ in try MainActor.assumeIsolated {
            let document = state.document
            XCTAssertThrowsError(try document.close())
            state.fired = true
            var edited = try document.snapshot().graph
            try Self.set(&edited, "title", "Saved calendar")
            try document.apply(edited, expectedRevision: document.revision)
            state.nested = try document.render(resources: Self.host())
            throw state.failure
        }}
        XCTAssertThrowsError(try document.invoke(method: 20, receiver: graph.root,
            arguments: [.ref(date)], resources: resources)) {
            XCTAssertTrue(($0 as AnyObject) === state.failure)
        }
        XCTAssertTrue(state.fired); XCTAssertNotNil(state.nested)
        let current = try document.snapshot()
        XCTAssertEqual(current.revision, document.revision)
        guard case .string(let id) = try current.graph.field("title") else { return XCTFail("Lost calendar edit") }
        XCTAssertEqual(current.graph.strings[Int(id)].stringIfWellFormed, "Saved calendar")
        let recovered = try document.invoke(method: 20, receiver: current.graph.root,
            arguments: [.ref(date)], resources: Self.host())
        guard case .returned(.bool(true)) = recovered.result else { return XCTFail("Sunday must remain excluded") }
    }
}
