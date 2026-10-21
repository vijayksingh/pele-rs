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
    private static func removeIndex(_ graph: inout Pele.PublicGraph, _ node: UInt32, _ index: UInt32) throws {
        guard case .array(let length, let properties) = graph.nodes[Int(node)] else { throw PeleError(code: "test", message: "Expected Array") }
        let keys=graph.strings
        graph.nodes[Int(node)] = .array(length: length, properties: properties.filter { keys[Int($0.key)].stringIfWellFormed != String(index) })
    }
    private static func truncate(_ graph: inout Pele.PublicGraph, _ node: UInt32, _ count: UInt32) throws {
        guard case .array(_, let properties) = graph.nodes[Int(node)] else { throw PeleError(code: "test", message: "Expected Array") }
        let keys=graph.strings
        graph.nodes[Int(node)] = .array(length: count, properties: properties.filter { property in
            guard let key=keys[Int(property.key)].stringIfWellFormed, let i=UInt32(key) else { return true }; return i<count
        })
    }
    func testRadarSparseAxesEmptyCurvesAndRecoveryMatch24OracleRows() throws {
        var path=URL(fileURLWithPath:#filePath); for _ in 0..<5 {path.deleteLastPathComponent()}
        let fixture=try XCTUnwrap(JSONSerialization.jsonObject(with:Data(contentsOf:path.appendingPathComponent("fixtures/public-radar-runtime-shapes.json"))) as? [String:Any])
        XCTAssertEqual(fixture["oracleSHA256"] as? String,"d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        let rows=try XCTUnwrap(fixture["rows"] as? [[String:Any]]),source=try XCTUnwrap(fixture["source"] as? String);XCTAssertEqual(rows.count,24)
        for row in rows {
            let mode=try XCTUnwrap(row["mode"] as? String),raw=try XCTUnwrap(row["raw"] as? Bool),engine=Pele.Engine()
            let document:RadarCallbackDocument=try raw ? .prepared(engine.parsePrepared("radar",source:source)) : .normal(engine.parseMutable(source))
            defer {try? document.close()}
            let initial=try document.snapshot().graph;var graph=initial
            let axes=try Self.ref(graph.field("axes")),curves=try Self.ref(graph.field("curves")),options=try Self.ref(graph.field("options"))
            if mode.hasPrefix("hole-"){try Self.removeIndex(&graph,axes,1)}
            if mode=="all-holes"{for i:UInt32 in 0..<3 {try Self.removeIndex(&graph,axes,i)}}
            if mode.contains("polygon"){try Self.setText(&graph,"graticule","polygon",on:options)}
            if mode=="hole-polygon-no-grid"{try graph.setField("ticks",value:.number(0),on:options)}
            if mode=="hole-skipped-curves"{for i in 0..<2 {let c=try Self.ref(graph.field(String(i),on:curves)),entries=try Self.input(&graph,[1,2]);try graph.setField("entries",value:entries,on:c)}}
            if mode=="hole-no-curves"{try Self.truncate(&graph,curves,0)}
            if mode.hasPrefix("empty-"){try Self.truncate(&graph,axes,0);try Self.truncate(&graph,curves,1);if mode != "empty-circle-skipped"{let c=try Self.ref(graph.field("0",on:curves)),entries=try Self.input(&graph,[Any]());try graph.setField("entries",value:entries,on:c)}}
            if mode=="empty-circle-no-legend"{try graph.setField("showLegend",value:.bool(false),on:options)}
            try document.apply(graph)
            let observed=RadarCallbackState();var host=pure()
            host.textWidth={ request in try MainActor.assumeIsolated {
                observed.trace.append([request.text,String(format:"%016llx",request.size.bitPattern),Int(request.style)])
                if !observed.fired {observed.fired=true;if mode.hasPrefix("callback-"){var current=try document.snapshot().graph;try Self.removeIndex(&current,axes,1);try Self.setText(&current,"title","Kept mutation");try document.apply(current);if mode=="callback-throw"{throw observed.failure}}}
                return Double(request.text.utf16.count)*request.size*0.6
            }}
            let first=try XCTUnwrap(row["first"] as? [String:Any])
            if first["ok"] as? Bool == true {try compare(document.render(host),first)}
            else if mode=="callback-throw" {XCTAssertThrowsError(try document.render(host)){XCTAssertTrue(($0 as AnyObject) === observed.failure)}}
            else {let expected=try XCTUnwrap(first["error"] as? [String:Any]);XCTAssertThrowsError(try document.render(host)){error in
                guard let failure=error as? Pele.PublicOperationFailure, case .javaScript(let name,let message)=failure.value else {return XCTFail("Expected TypeError: \(error)")}
                XCTAssertEqual(name,expected["name"] as? String);XCTAssertEqual(message.stringIfWellFormed,expected["message"] as? String)
            }}
            XCTAssertEqual(observed.fired,row["fired"] as? Bool);XCTAssertTrue(NSArray(array:observed.trace).isEqual(to:try XCTUnwrap(row["trace"] as? [[Any]])),mode)
            let after=try document.snapshot().graph
            XCTAssertTrue(NSDictionary(dictionary:try Self.state(after)).isEqual(to:try XCTUnwrap(row["after"] as? [String:Any])),mode)
            XCTAssertEqual(try Self.ref(after.field("axes")),axes);XCTAssertEqual(try Self.ref(after.field("curves")),curves)
            XCTAssertTrue(NSArray(array:try XCTUnwrap(Self.value(after,.ref(axes)) as? [Any])).isEqual(to:try XCTUnwrap(row["oldAxes"] as? [Any])))
            XCTAssertTrue(NSArray(array:try XCTUnwrap(Self.value(after,.ref(curves)) as? [Any])).isEqual(to:try XCTUnwrap(row["oldCurves"] as? [Any])))
            try document.apply(initial)
            let recovery=RadarCallbackState();var plain=pure();plain.textWidth={request in MainActor.assumeIsolated{recovery.trace.append([request.text,String(format:"%016llx",request.size.bitPattern),Int(request.style)]);return Double(request.text.utf16.count)*request.size*0.6}}
            try compare(document.render(plain),XCTUnwrap(row["recovery"] as? [String:Any]));XCTAssertTrue(NSArray(array:recovery.trace).isEqual(to:try XCTUnwrap(row["recoveryTrace"] as? [[Any]])))
        }
    }

}
