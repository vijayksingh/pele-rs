import Foundation
import XCTest
import Pele

private final class TimelineCallbackFailure: Error {}
@MainActor private enum TimelineCallbackDocument {
    case normal(Pele.MutableDocument), prepared(Pele.PreparedDocument)
    var revision: UInt64 { switch self { case .normal(let d): d.revision; case .prepared(let d): d.revision } }
    func snapshot() throws -> Pele.PublicSnapshot { switch self { case .normal(let d): try d.snapshot(); case .prepared(let d): try d.snapshot() } }
    func apply(_ graph: Pele.PublicGraph) throws {
        switch self { case .normal(let d): _ = try d.apply(graph, expectedRevision: d.revision); case .prepared(let d): _ = try d.apply(graph, expectedRevision: d.revision) }
    }
    func render(_ resources: Pele.HostResources, responsive: Bool = false) throws -> FullRenderedDiagram {
        var options = Pele.Engine.Options(); options.maxWidth = responsive ? 300 : nil
        switch self { case .normal(let d): return try d.render(options: options, resources: resources); case .prepared(let d): return try d.render(options: options, resources: resources) }
    }
    func close() throws { switch self { case .normal(let d): try d.close(); case .prepared(let d): try d.close() } }
}
@MainActor private final class TimelineCallbackState {
    var fired = false
    var trace: [[Any]] = []
    var nested: FullRenderedDiagram?
    var nestedRevision: UInt64?
    let failure = TimelineCallbackFailure()
}

@MainActor final class NativeTimelineCallbackTests: XCTestCase {
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
        guard case .ref(let id) = atom else { throw PeleError(code: "test", message: "Timeline reference missing") }; return id
    }
    private static func setText(_ graph: inout Pele.PublicGraph, _ key: String, _ value: String, on node: UInt32? = nil) throws {
        let id = try graph.intern(value); try graph.setField(key, value: .string(id), on: node)
    }
    private static func append(_ graph: inout Pele.PublicGraph, on node:UInt32, value:Pele.PublicGraph.Atom) throws {
        guard case .array(let length,_) = graph.nodes[Int(node)] else {throw PeleError(code:"test",message:"Timeline array missing")}
        try graph.setField(String(length),value:value,on:node)
        graph.nodes[Int(node)] = .array(length:length+1,properties:graph.nodes[Int(node)].properties)
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
        } else { throw PeleError(code: "test", message: "Unexpected Timeline fixture input") }
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
        default: throw PeleError(code: "test", message: "Unexpected Timeline fixture atom")
        }
    }
    private static func state(_ graph: Pele.PublicGraph) throws -> [String: Any] {
        var result: [String: Any] = [:]
        for key in ["title", "accTitle", "accDescr", "direction", "sections", "periods"] { result[key] = try value(graph, graph.field(key) ?? .undefined) }
        return result
    }
    func testTimelineCapturedArraysLateFieldsNestedCallbacksAndRecoveryMatchAllOracleRows() throws {
        var path = URL(fileURLWithPath: #filePath); for _ in 0..<5 { path.deleteLastPathComponent() }
        let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-timeline-live-render.json"))) as? [String: Any])
        XCTAssertEqual(fixture["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        let source = try XCTUnwrap(fixture["source"] as? String), rows = try XCTUnwrap(fixture["rows"] as? [[String: Any]])
        XCTAssertEqual(rows.count, 52)
        for row in rows {
            let raw = try XCTUnwrap(row["raw"] as? Bool), responsive = try XCTUnwrap(row["responsive"] as? Bool), mode = try XCTUnwrap(row["mode"] as? String)
            let engine = Pele.Engine()
            let document: TimelineCallbackDocument = try raw ? .prepared(engine.parsePrepared("timeline", source: source)) : .normal(engine.parseMutable(source))
            defer { try? document.close() }
            if mode == "refit" { var graph=try document.snapshot().graph; let periods=try Self.ref(graph.field("periods")), first=try Self.ref(graph.field("0",on:periods)); try Self.setText(&graph,"text",String(repeating:"A",count:80),on:first); try document.apply(graph) }
            let initial = try document.snapshot().graph
            let periods = try Self.ref(initial.field("periods")), sections = try Self.ref(initial.field("sections")), firstPeriod=try Self.ref(initial.field("0",on:periods)), events=try Self.ref(initial.field("events",on:firstPeriod)), observed = TimelineCallbackState()
            var resources = pure()
            resources.textWidth = { request in try MainActor.assumeIsolated {
                observed.trace.append([request.text, String(format: "%016llx", request.size.bitPattern), Int(request.style)])
                if !observed.fired && (mode != "events-append" || request.text == "Event One") {
                    observed.fired = true
                    XCTAssertThrowsError(try document.close())
                    var graph = try document.snapshot().graph
                    let currentPeriods = try Self.ref(graph.field("periods")), first = try Self.ref(graph.field("0",on:currentPeriods)), second = try Self.ref(graph.field("1",on:currentPeriods))
                    switch mode {
                    case "period-edit":
                        try Self.setText(&graph,"text","Changed first",on:first); try Self.setText(&graph,"text","Changed second",on:second); try graph.setField("sectionIndex",value:.number(1),on:first)
                    case "period-replace":
                        let value=try Self.input(&graph,["text":"Replacement","events":["Replacement event"],"section":"Before","sectionIndex":0]); try graph.setField("1",value:value,on:currentPeriods)
                    case "periods-replace":
                        let value=try Self.input(&graph,[["text":"New root period","events":["New root event"],"section":"Before","sectionIndex":0]]); try graph.setField("periods",value:value)
                    case "period-append":
                        let value=try Self.input(&graph,["text":"Appended","events":["Appended event"],"section":"Later","sectionIndex":1]); try Self.append(&graph,on:currentPeriods,value:value)
                    case "events-edit":
                        let events=try Self.ref(graph.field("events",on:second)); try Self.setText(&graph,"0","Changed event",on:events)
                    case "events-replace":
                        let value=try Self.input(&graph,["New events","Second event"]); try graph.setField("events",value:value,on:first)
                    case "events-append":
                        let events=try Self.ref(graph.field("events",on:first)), value=try Self.input(&graph,"Appended event"); try Self.append(&graph,on:events,value:value)
                    case "sections":
                        let value=try Self.input(&graph,["Changed section","Changed later"]); try graph.setField("sections",value:value)
                    case "title-accessibility":
                        try Self.setText(&graph,"title","Changed title"); try Self.setText(&graph,"accTitle","Changed access"); try Self.setText(&graph,"accDescr","Changed description"); try Self.setText(&graph,"direction","TD")
                    case "refit": try Self.setText(&graph,"text","After refit",on:first); try Self.setText(&graph,"text","Changed second",on:second)
                    case "throw": try Self.setText(&graph,"title","Before throw"); try Self.setText(&graph,"text","Saved period",on:second)
                    case "nested": try Self.setText(&graph,"title","Nested title")
                    case "baseline": break
                    default: throw PeleError(code:"test",message:"Unknown Timeline fixture mode")
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
            XCTAssertEqual(try Self.ref(after.graph.field("periods")) == periods, row["periodsCurrent"] as? Bool); XCTAssertEqual(try Self.ref(after.graph.field("sections")) == sections, row["sectionsCurrent"] as? Bool)
            XCTAssertTrue(NSArray(array: try XCTUnwrap(Self.value(after.graph, .ref(periods)) as? [Any])).isEqual(to: try XCTUnwrap(row["originalPeriods"] as? [Any])), mode + " original periods")
            XCTAssertTrue(NSArray(array: try XCTUnwrap(Self.value(after.graph, .ref(sections)) as? [Any])).isEqual(to: try XCTUnwrap(row["originalSections"] as? [Any])), mode + " original sections")
            XCTAssertTrue(NSArray(array: try XCTUnwrap(Self.value(after.graph,.ref(events)) as? [Any])).isEqual(to:try XCTUnwrap(row["originalEvents"] as? [Any])),mode + " original events")
            let currentPeriods=try Self.ref(after.graph.field("periods")), currentFirst=try Self.ref(after.graph.field("0",on:currentPeriods))
            XCTAssertEqual(try Self.ref(after.graph.field("events",on:currentFirst)) == events,row["eventsCurrent"] as? Bool)
            if mode == "nested" { try compare(XCTUnwrap(observed.nested), XCTUnwrap(row["nested"] as? [String: Any])); XCTAssertGreaterThan(document.revision, try XCTUnwrap(observed.nestedRevision)) }
            let recovery = TimelineCallbackState(); var plain = pure()
            plain.textWidth = { request in MainActor.assumeIsolated { recovery.trace.append([request.text, String(format: "%016llx", request.size.bitPattern), Int(request.style)]); return Double(request.text.utf16.count) * request.size * 0.6 } }
            try compare(document.render(plain, responsive: responsive), XCTUnwrap(row["recovery"] as? [String: Any]))
            XCTAssertTrue(NSArray(array: recovery.trace).isEqual(to: try XCTUnwrap(row["recoveryTrace"] as? [[Any]])), mode + " recovery trace")
        }
    }
}
