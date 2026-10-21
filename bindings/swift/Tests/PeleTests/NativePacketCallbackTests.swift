import Foundation
import XCTest
import Pele

private final class PacketCallbackFailure: Error {}
@MainActor private enum PacketCallbackDocument {
    case normal(Pele.MutableDocument), prepared(Pele.PreparedDocument)
    var revision: UInt64 { switch self { case .normal(let d): d.revision; case .prepared(let d): d.revision } }
    func snapshot() throws -> Pele.PublicSnapshot { switch self { case .normal(let d): try d.snapshot(); case .prepared(let d): try d.snapshot() } }
    func apply(_ graph: Pele.PublicGraph) throws {
        switch self { case .normal(let d): _ = try d.apply(graph, expectedRevision: d.revision); case .prepared(let d): _ = try d.apply(graph, expectedRevision: d.revision) }
    }
    func render(_ resources: Pele.HostResources, responsive: Bool = false) throws -> FullRenderedDiagram {
        var options = Pele.Engine.Options(); options.maxWidth = responsive ? 220 : nil
        switch self { case .normal(let d): return try d.render(options: options, resources: resources); case .prepared(let d): return try d.render(options: options, resources: resources) }
    }
    func close() throws { switch self { case .normal(let d): try d.close(); case .prepared(let d): try d.close() } }
}
@MainActor private final class PacketCallbackState {
    var fired = false
    var trace: [[Any]] = []
    var nested: FullRenderedDiagram?
    var nestedRevision: UInt64?
    let failure = PacketCallbackFailure()
}

@MainActor final class NativePacketCallbackTests: XCTestCase {
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
        guard case .ref(let id) = value else { throw PeleError(code: "test", message: "Expected Packet reference") }; return id
    }
    private static func first(_ graph: Pele.PublicGraph) throws -> (rows: UInt32, row: UInt32, block: UInt32) {
        let rows = try ref(graph.field("rows")), row = try ref(graph.field("0", on: rows)), block = try ref(graph.field("0", on: row)); return (rows, row, block)
    }
    private static func setText(_ graph: inout Pele.PublicGraph, _ key: String, _ text: String, on object: UInt32? = nil) throws {
        let id = try graph.intern(text); try graph.setField(key, value: .string(id), on: object)
    }
    private static func block(_ graph: inout Pele.PublicGraph, _ start: Double, _ end: Double, _ count: Double, _ label: String) throws -> UInt32 {
        let id = UInt32(graph.nodes.count); graph.nodes.append(.object(properties: []))
        for (key, value) in [("start", start), ("end", end), ("bits", count)] { try graph.setField(key, value: .number(value), on: id) }
        try setText(&graph, "label", label, on: id); return id
    }
    private static func array(_ graph: inout Pele.PublicGraph, _ values: [Pele.PublicGraph.Atom]) throws -> UInt32 {
        let id = UInt32(graph.nodes.count); graph.nodes.append(.array(length: 0, properties: []))
        for (i, value) in values.enumerated() { try graph.setArrayElement(UInt32(i), value: value, on: id) }; return id
    }
    private func state(_ graph: Pele.PublicGraph) throws -> [String: Any] {
        func value(_ atom: Pele.PublicGraph.Atom?) throws -> Any {
            switch atom { case .string(let i): return try XCTUnwrap(graph.strings[Int(i)].stringIfWellFormed)
            case .number(let n): return bits(n)
            default: return NSNull() }
        }
        var result: [String: Any] = [:]
        for key in ["title", "accTitle", "accDescr", "bitsPerRow"] { result[key] = try value(graph.field(key)) }
        let rows = try Self.ref(graph.field("rows")); guard case .array(let length, _) = graph.nodes[Int(rows)] else { throw PacketCallbackFailure() }
        var output: [[[String: Any]]] = []
        for i in 0..<length {
            let row = try Self.ref(graph.field(String(i), on: rows)); guard case .array(let count, _) = graph.nodes[Int(row)] else { throw PacketCallbackFailure() }
            var line: [[String: Any]] = []
            for j in 0..<count {
                let block = try Self.ref(graph.field(String(j), on: row)); var record: [String: Any] = [:]
                for key in ["start", "end", "bits", "label"] { record[key] = try value(graph.field(key, on: block)) }; line.append(record)
            }
            output.append(line)
        }
        result["rows"] = output; return result
    }
    func testPacketLiveFieldsCopiedResponsiveRowsOriginalErrorsAndNestedRendersMatch48OracleRows() throws {
        var path = URL(fileURLWithPath: #filePath); for _ in 0..<5 { path.deleteLastPathComponent() }
        let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-packet-live-render.json"))) as? [String: Any])
        XCTAssertEqual(fixture["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        let source = try XCTUnwrap(fixture["source"] as? String), rows = try XCTUnwrap(fixture["rows"] as? [[String: Any]])
        XCTAssertEqual(rows.count, 48)
        for row in rows {
            let raw = try XCTUnwrap(row["raw"] as? Bool), responsive = try XCTUnwrap(row["responsive"] as? Bool), mode = try XCTUnwrap(row["mode"] as? String)
            let engine = Pele.Engine()
            let document: PacketCallbackDocument = try raw ? .prepared(engine.parsePrepared("packet", source: source)) : .normal(engine.parseMutable(source))
            defer { try? document.close() }
            let original = try Self.first(document.snapshot().graph), observed = PacketCallbackState()
            var resources = pure()
            resources.textWidth = { request in try MainActor.assumeIsolated {
                observed.trace.append([request.text, String(format: "%016llx", request.size.bitPattern), Int(request.style)])
                if !observed.fired && (!["current-label", "current-geometry", "append-block"].contains(mode) || request.text == "Alpha") {
                    observed.fired = true
                    XCTAssertThrowsError(try document.close())
                    var graph = try document.snapshot().graph
                    let initial = try Self.first(graph)
                    switch mode {
                    case "title": try Self.setText(&graph, "title", "Changed title")
                    case "current-label": try Self.setText(&graph, "label", "Changed Alpha", on: initial.block)
                    case "later-label":
                        let second = try Self.ref(graph.field("1", on: initial.row)); try Self.setText(&graph, "label", "Changed Beta", on: second)
                    case "current-geometry":
                        try graph.setField("start", value: .number(1), on: initial.block); try graph.setField("end", value: .number(6), on: initial.block)
                    case "append-block":
                        let block = try Self.block(&graph, 64, 71, 8, "Added")
                        guard case .array(let length, _) = graph.nodes[Int(initial.row)] else { throw observed.failure }
                        try graph.setArrayElement(length, value: .ref(block), on: initial.row)
                    case "replace-row":
                        let block = try Self.block(&graph, 0, 31, 32, "Replacement row"), row = try Self.array(&graph, [.ref(block)])
                        try graph.setArrayElement(0, value: .ref(row), on: initial.rows)
                    case "replace-rows":
                        let block = try Self.block(&graph, 0, 15, 16, "Replacement rows"), row = try Self.array(&graph, [.ref(block)]), rows = try Self.array(&graph, [.ref(row)])
                        try graph.setField("rows", value: .ref(rows))
                    case "accessibility":
                        try Self.setText(&graph, "accTitle", "Changed access"); try Self.setText(&graph, "accDescr", "Changed description")
                    case "throw-object", "throw-undefined":
                        try Self.setText(&graph, "title", "Before failure"); try Self.setText(&graph, "label", "Persisted label", on: initial.block)
                    case "nested": try Self.setText(&graph, "title", "Nested title")
                    case "baseline": break
                    default: throw PeleError(code: "test", message: "Unknown Packet fixture mode")
                    }
                    if mode != "baseline" { try document.apply(graph) }
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
            XCTAssertEqual(latest.rows == original.rows, identity["rows"])
            XCTAssertEqual(latest.block == original.block, identity["first"])
            if mode == "nested" {
                try compare(XCTUnwrap(observed.nested), XCTUnwrap(row["nested"] as? [String: Any]))
                XCTAssertGreaterThan(document.revision, try XCTUnwrap(observed.nestedRevision))
            }
            let recovery = PacketCallbackState(); var plain = pure()
            plain.textWidth = { request in MainActor.assumeIsolated {
                recovery.trace.append([request.text, String(format: "%016llx", request.size.bitPattern), Int(request.style)])
                return Double(request.text.utf16.count) * request.size * 0.6
            }}
            try compare(document.render(plain, responsive: responsive), XCTUnwrap(row["recovery"] as? [String: Any]))
            XCTAssertTrue(NSArray(array: recovery.trace).isEqual(to: try XCTUnwrap(row["recoveryTrace"] as? [[Any]])), mode + " recovery trace")
        }
    }
}
