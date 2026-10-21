import Foundation
import XCTest
import Pele

private final class RadarCallbackFailure: Error {}
@MainActor private enum RadarCallbackDocument {
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
@MainActor private final class RadarCallbackState {
    var fired = false
    var trace: [[Any]] = []
    var nested: FullRenderedDiagram?
    var nestedRevision: UInt64?
    let failure = RadarCallbackFailure()
}

@MainActor final class NativeRadarCallbackTests: XCTestCase {
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
    private static func ref(_ atom: Pele.PublicGraph.Atom?) throws -> UInt32 {
        guard case .ref(let id) = atom else { throw PeleError(code: "test", message: "Radar reference missing") }; return id
    }
    private static func setText(_ graph: inout Pele.PublicGraph, _ key: String, _ value: String, on node: UInt32? = nil) throws {
        let id = try graph.intern(value); try graph.setField(key, value: .string(id), on: node)
    }
    private static func input(_ graph: inout Pele.PublicGraph, _ value: Any) throws -> Pele.PublicGraph.Atom {
        if let string = value as? String { return .string(try graph.intern(string)) }
        if let number = value as? NSNumber { return .number(number.doubleValue) }
        let id = UInt32(graph.nodes.count); graph.nodes.append(.object(properties: []))
        var properties: [Pele.PublicGraph.Property] = []
        if let array = value as? [Any] {
            for (i, value) in array.enumerated() { let key = try graph.intern(String(i)), atom = try input(&graph, value); properties.append(.init(key: key, value: atom)) }
            graph.nodes[Int(id)] = .array(length: UInt32(array.count), properties: properties)
        } else if let object = value as? [String: Any] {
            for (name, value) in object.sorted(by: { $0.key < $1.key }) { let key = try graph.intern(name), atom = try input(&graph, value); properties.append(.init(key: key, value: atom)) }
            graph.nodes[Int(id)] = .object(properties: properties)
        } else { throw PeleError(code: "test", message: "Unexpected Radar fixture input") }
        return .ref(id)
    }
    private static func value(_ graph: Pele.PublicGraph, _ atom: Pele.PublicGraph.Atom) throws -> Any {
        switch atom {
        case .undefined, .null: return NSNull()
        case .bool(let b): return b
        case .number(let n): return n
        case .string(let id): return try XCTUnwrap(graph.strings[Int(id)].stringIfWellFormed)
        case .ref(let id):
            let node = graph.nodes[Int(id)]
            if case .array(let length, _) = node { return try (0..<length).map { try value(graph, graph.field(String($0), on: id) ?? .undefined) } }
            var object: [String: Any] = [:]
            for property in node.properties { object[try XCTUnwrap(graph.strings[Int(property.key)].stringIfWellFormed)] = try value(graph, property.value) }
            return object
        default: throw PeleError(code: "test", message: "Unexpected Radar fixture atom")
        }
    }
    private static func state(_ graph: Pele.PublicGraph) throws -> [String: Any] {
        var result: [String: Any] = [:]
        for key in ["title", "accTitle", "accDescr", "axes", "curves", "options"] { result[key] = try value(graph, graph.field(key) ?? .undefined) }
        return result
    }
    func testRadarCapturedArraysLateFieldsNestedCallbacksAndRecoveryMatchAllOracleRows() throws {
        var path = URL(fileURLWithPath: #filePath); for _ in 0..<5 { path.deleteLastPathComponent() }
        let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-radar-live-render.json"))) as? [String: Any])
        XCTAssertEqual(fixture["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        let source = try XCTUnwrap(fixture["source"] as? String), rows = try XCTUnwrap(fixture["rows"] as? [[String: Any]])
        XCTAssertEqual(rows.count, 40)
        for row in rows {
            let raw = try XCTUnwrap(row["raw"] as? Bool), responsive = try XCTUnwrap(row["responsive"] as? Bool), mode = try XCTUnwrap(row["mode"] as? String)
            let engine = Pele.Engine()
            let document: RadarCallbackDocument = try raw ? .prepared(engine.parsePrepared("radar", source: source)) : .normal(engine.parseMutable(source))
            defer { try? document.close() }
            let initial = try document.snapshot().graph
            let axes = try Self.ref(initial.field("axes")), curves = try Self.ref(initial.field("curves")), observed = RadarCallbackState()
            var resources = pure()
            resources.textWidth = { request in try MainActor.assumeIsolated {
                observed.trace.append([request.text, String(format: "%016llx", request.size.bitPattern), Int(request.style)])
                if !observed.fired && (mode != "legend-mutation" || request.text == "One") {
                    observed.fired = true
                    XCTAssertThrowsError(try document.close())
                    var graph = try document.snapshot().graph
                    let currentAxes = try Self.ref(graph.field("axes")), currentCurves = try Self.ref(graph.field("curves"))
                    switch mode {
                    case "axis-edit":
                        let first = try Self.ref(graph.field("0", on: currentAxes)), second = try Self.ref(graph.field("1", on: currentAxes))
                        try Self.setText(&graph, "name", "changed-a", on: first); try Self.setText(&graph, "name", "changed-b", on: second); try Self.setText(&graph, "label", "Changed Beta", on: second)
                    case "axes-replace":
                        let value = try Self.input(&graph, [["name": "x", "label": "X"], ["name": "y", "label": "Y"], ["name": "z", "label": "Z"]]); try graph.setField("axes", value: value)
                    case "curve-edit":
                        let curve = try Self.ref(graph.field("0", on: currentCurves))
                        try Self.setText(&graph, "name", "changed-one", on: curve); try Self.setText(&graph, "label", "Changed One", on: curve)
                        let entries = try Self.input(&graph, [3, 1, 2]); try graph.setField("entries", value: entries, on: curve)
                    case "curves-replace":
                        let value = try Self.input(&graph, [["name": "replacement", "label": "Replacement", "entries": [2, 3, 1]]]); try graph.setField("curves", value: value)
                    case "options":
                        let options = try Self.ref(graph.field("options"))
                        try graph.setField("ticks", value: .number(2), on: options); try graph.setField("min", value: .number(1), on: options); try graph.setField("max", value: .number(4), on: options)
                        try Self.setText(&graph, "graticule", "polygon", on: options); try graph.setField("showLegend", value: .bool(false), on: options)
                    case "legend-mutation":
                        let first = try Self.ref(graph.field("0", on: currentCurves)), second = try Self.ref(graph.field("1", on: currentCurves))
                        try Self.setText(&graph, "name", "late-one", on: first); try Self.setText(&graph, "label", "Late Two", on: second); try Self.setText(&graph, "title", "Late title")
                    case "title-accessibility":
                        try Self.setText(&graph, "title", "Changed title"); try Self.setText(&graph, "accTitle", "Changed access"); try Self.setText(&graph, "accDescr", "Changed description")
                    case "throw":
                        let second = try Self.ref(graph.field("1", on: currentAxes)); try Self.setText(&graph, "title", "Before throw"); try Self.setText(&graph, "label", "Saved Beta", on: second)
                    case "nested": try Self.setText(&graph, "title", "Nested title")
                    case "baseline": break
                    default: throw PeleError(code: "test", message: "Unknown Radar fixture mode")
                    }
                    if mode != "baseline" { try document.apply(graph) }
                    if mode == "throw" { throw observed.failure }
                    if mode == "nested" {
                        var inner = Pele.HostResources(); inner.textWidth = { Double($0.text.utf16.count) * $0.size * 0.6 }
                        observed.nested = try document.render(inner); observed.nestedRevision = document.revision
                        var latest = try document.snapshot().graph; try Self.setText(&latest, "title", "After nested"); try document.apply(latest)
                    }
                }
                return Double(request.text.utf16.count) * request.size * 0.6
            }}
            let first = try XCTUnwrap(row["first"] as? [String: Any])
            if mode == "throw" { XCTAssertEqual(first["original"] as? Bool, true); XCTAssertThrowsError(try document.render(resources, responsive: responsive)) { XCTAssertTrue(($0 as AnyObject) === observed.failure) } }
            else { try compare(document.render(resources, responsive: responsive), first) }
            XCTAssertTrue(observed.fired)
            XCTAssertTrue(NSArray(array: observed.trace).isEqual(to: try XCTUnwrap(row["trace"] as? [[Any]])), "\(mode)/\(raw)/\(responsive) trace")
            let after = try document.snapshot(); XCTAssertEqual(after.revision, document.revision)
            XCTAssertTrue(NSDictionary(dictionary: try Self.state(after.graph)).isEqual(to: try XCTUnwrap(row["after"] as? [String: Any])), mode + " state")
            XCTAssertEqual(try Self.ref(after.graph.field("axes")) == axes, row["axesCurrent"] as? Bool); XCTAssertEqual(try Self.ref(after.graph.field("curves")) == curves, row["curvesCurrent"] as? Bool)
            XCTAssertTrue(NSArray(array: try XCTUnwrap(Self.value(after.graph, .ref(axes)) as? [Any])).isEqual(to: try XCTUnwrap(row["originalAxes"] as? [Any])), mode + " original axes")
            XCTAssertTrue(NSArray(array: try XCTUnwrap(Self.value(after.graph, .ref(curves)) as? [Any])).isEqual(to: try XCTUnwrap(row["originalCurves"] as? [Any])), mode + " original curves")
            if mode == "nested" { try compare(XCTUnwrap(observed.nested), XCTUnwrap(row["nested"] as? [String: Any])); XCTAssertGreaterThan(document.revision, try XCTUnwrap(observed.nestedRevision)) }
            let recovery = RadarCallbackState(); var plain = pure()
            plain.textWidth = { request in MainActor.assumeIsolated { recovery.trace.append([request.text, String(format: "%016llx", request.size.bitPattern), Int(request.style)]); return Double(request.text.utf16.count) * request.size * 0.6 } }
            try compare(document.render(plain, responsive: responsive), XCTUnwrap(row["recovery"] as? [String: Any]))
            XCTAssertTrue(NSArray(array: recovery.trace).isEqual(to: try XCTUnwrap(row["recoveryTrace"] as? [[Any]])), mode + " recovery trace")
        }
    }
}
