import Foundation
import XCTest
import Pele

private final class NumberTrace: @unchecked Sendable {
    private let lock = NSLock()
    private var calls: [ConfigValue] = []
    func append(_ record: ConfigValue) { lock.lock(); calls.append(record); lock.unlock() }
    var value: ConfigValue { lock.lock(); defer { lock.unlock() }; return .array(calls) }
}
private final class NumberClockFailure: Error {}
private final class NumberWidthFailure: Error {}
final class RenderNumberTests: XCTestCase {
    @discardableResult
    private func assertPublishedRevision(_ document: Pele.MutableDocument, after previous: UInt64, _ id: String = "", file: StaticString = #filePath, line: UInt = #line) throws -> UInt64 {
        let current = document.revision
        XCTAssertGreaterThan(current, previous, id, file: file, line: line)
        XCTAssertEqual(try document.snapshot().revision, current, id, file: file, line: line)
        XCTAssertEqual(document.revision, current, id, file: file, line: line)
        return current
    }
    private struct Number: Decodable {
        let kind: String
        let bits: String?
        func value() throws -> Double {
            XCTAssertEqual(kind, "number")
            return Double(bitPattern: try XCTUnwrap(UInt64(try XCTUnwrap(bits), radix: 16)))
        }
    }
    private struct Fixture: Decodable {
        struct Host: Decodable { let now: Number }
        struct Item: Decodable {
            struct Expected: Decodable {
                struct HostError: Decodable { let name: String; let message: String }
                // Oracle JSON represents nonfinite dimensions as null, without
                // the SDK's IEEE sidecars. Fixture bits below are authoritative.
                struct Drawing: Decodable { let type: String; let svg: String; let links: [DiagramLink] }
                let result: Drawing?
                let widthBits: String?
                let heightBits: String?
                let error: HostError?
                let calls: ConfigValue
                let graphBefore: ConfigValue
                let graphAfter: ConfigValue
            }
            let id: String
            let family: String
            let source: String
            let legacyOptions: [String: ConfigValue]
            let numeric: [String: Number]
            let values: [String: Number]?
            let hostNow: Number?
            let hostThrows: Bool?
            let boundary: String?
            let expected: Expected
        }
        let host: Host
        let cases: [Item]
    }
    private func fixture(_ name: String = "render-numbers.json") throws -> Fixture {
        let root = URL(fileURLWithPath: #filePath).deletingLastPathComponent().appendingPathComponent("../../../..").standardizedFileURL
        let url = root.appendingPathComponent("fixtures/pele/" + name)
        let data: Data
        if url.pathExtension == "gz" {
            let process = Process(), output = Pipe()
            process.executableURL = URL(fileURLWithPath: "/usr/bin/gzip")
            process.arguments = ["-dc", url.path]
            process.environment = ["PATH": "/usr/bin:/bin", "LANG": "C", "LC_ALL": "C", "TZ": "UTC"]
            process.standardOutput = output
            try process.run()
            data = output.fileHandleForReading.readDataToEndOfFile()
            process.waitUntilExit()
            guard process.terminationStatus == 0 else {
                throw NSError(domain: "RenderNumberFixtureGzip", code: Int(process.terminationStatus))
            }
        } else { data = try Data(contentsOf: url) }
        return try JSONDecoder().decode(Fixture.self, from: data)
    }
    private func hex(_ number: Double) -> String { String(format: "%016llx", number.bitPattern) }
    // Arithmetic can change NaN sign/payload. Transport and finite/zero bits are
    // strict; computed trace/graph NaNs compare by class, as the pinned contract.
    private func arithmetic(_ value: ConfigValue) -> ConfigValue {
        switch value {
        case .string(let value):
            if value.count == 16, let bits = UInt64(value, radix: 16), Double(bitPattern: bits).isNaN { return .string("computed-NaN") }
            return .string(value)
        case .array(let values): return .array(values.map(arithmetic))
        case .object(let values): return .object(values.mapValues(arithmetic))
        default: return value
        }
    }
    private func canonical(_ graph: Pele.PublicGraph) -> ConfigValue {
        var ids: [UInt32: UInt32] = [:], pending: [UInt32] = [], nodes: [ConfigValue] = []
        func units(_ value: Pele.JSString) -> ConfigValue { .array(value.codeUnits.map { .number(Double($0)) }) }
        func atom(_ value: Pele.PublicGraph.Atom) -> ConfigValue {
            switch value {
            case .undefined: return .array([.string("undefined")])
            case .null: return .array([.string("null")])
            case .bool(let v): return .array([.string("bool"), .bool(v)])
            case .number(let v): return .array([.string("number"), .string(String(format: "%016llx", v.bitPattern))])
            case .string(let id): return .array([.string("string"), units(graph.strings[Int(id)])])
            case .builtinFunction(let id): return .array([.string("builtinFunction"), .number(Double(id))])
            case .hostReference(let id): return .array([.string("hostReference"), .number(Double(id))])
            case .ref(let id):
                if ids[id] == nil { ids[id] = UInt32(pending.count); pending.append(id) }
                return .array([.string("ref"), .number(Double(ids[id]!))])
            }
        }
        let root = atom(graph.root)
        var cursor = 0
        while cursor < pending.count {
            let node = graph.nodes[Int(pending[cursor])]
            var value: [String: ConfigValue] = [:]
            switch node {
            case .error(let kind, _): value = ["kind": .string("error"), "errorType": .string(kind.rawValue)]
            case .instance(let prototype, _): value = ["kind": .string("instance"), "prototype": .number(Double(prototype))]
            case .object: value = ["kind": .string("object")]
            case .array(let length, _): value = ["kind": .string("array"), "length": .number(Double(length))]
            case .map(let entries, _): value = ["kind": .string("map"), "entries": .array(entries.map { .array([atom($0.key), atom($0.value)]) })]
            case .set(let values, _): value = ["kind": .string("set"), "values": .array(values.map(atom))]
            case .date(let ms, _): value = ["kind": .string("date"), "milliseconds": .string(String(format: "%016llx", ms.bitPattern))]
            }
            value["properties"] = .array(node.properties.map { .array([units(graph.strings[Int($0.key)]), atom($0.value), .number(Double($0.flags))]) })
            nodes.append(.object(value)); cursor += 1
        }
        return .object(["root": root, "nodes": .array(nodes)])
    }
    private func options(_ item: Fixture.Item) throws -> Pele.Engine.Options {
        var fields = item.legacyOptions; fields["config"] = fields["config"] ?? .object([:])
        var options = try JSONDecoder().decode(Pele.Engine.Options.self, from: JSONEncoder().encode(ConfigValue.object(fields)))
        for (field, number) in item.numeric {
            let value = try number.value()
            switch field {
            case "fontSize": options.fontSize = value
            case "maxWidth": options.maxWidth = value
            case "directionBreakpoint": options.directionBreakpoint = value
            case "padding": options.padding = value
            case "now": options.now = value
            default: XCTFail("Unknown numeric field: \(field)")
            }
        }
        if let date = item.values?["now"], date.kind == "date" {
            let raw = Double(bitPattern: try XCTUnwrap(UInt64(try XCTUnwrap(date.bits), radix: 16)))
            options.setNow(date: Date(timeIntervalSince1970: raw / 1000))
        }
        return options
    }
    private func resources(item: Fixture.Item, fixture: Fixture, trace: NumberTrace, failure: NumberClockFailure) throws -> Pele.HostResources {
        let now = try (item.hostNow ?? fixture.host.now).value(), throwsClock = item.hostThrows == true
        var host = Pele.HostResources(); host.timeZone = TimeZone(secondsFromGMT: 0)!
        host.nowMillis = {
            if throwsClock { trace.append(.array([.string("clock"), .string("throw")])); throw failure }
            trace.append(.array([.string("clock"), .string(String(format: "%016llx", now.bitPattern))])); return now
        }
        host.textWidth = { request in
            let value = Double(request.text.utf16.count) * request.size * (request.style & 1 != 0 ? 0.71 : 0.6)
            trace.append(.array([.string("width"), .array(request.text.utf16.map { .number(Double($0)) }), .string(String(format: "%016llx", request.size.bitPattern)), .number(Double(request.style)), .string(String(format: "%016llx", value.bitPattern))]))
            return value
        }
        return host
    }
    private func equal(_ actual: FullRenderedDiagram, expected: Fixture.Item.Expected, id: String) throws {
        let result = try XCTUnwrap(expected.result)
        XCTAssertEqual(actual.type, result.type, id); XCTAssertEqual(actual.svg, result.svg, id)
        XCTAssertEqual(actual.links.map { [$0.id, $0.href, $0.internal ? "true" : "false"] },
                       result.links.map { [$0.id, $0.href, $0.internal ? "true" : "false"] }, id)
        for (value, text) in [(actual.width, expected.widthBits), (actual.height, expected.heightBits)] {
            let bits = try XCTUnwrap(UInt64(try XCTUnwrap(text), radix: 16))
            if Double(bitPattern: bits).isNaN { XCTAssertTrue(value.isNaN, id) } else { XCTAssertEqual(value.bitPattern, bits, id) }
        }
    }
    func testPinnedNumbersAcrossSourceRetainedAndMutableDocuments() throws {
        let fixture = try fixture(); XCTAssertEqual(fixture.cases.count, 290)
        let excluded = fixture.cases.filter { $0.boundary != nil }
        XCTAssertEqual(excluded.count, 3)
        XCTAssertEqual(Set(excluded.map(\.family)), Set(["pie", "flowchart", "gantt"]))
        XCTAssertTrue(excluded.allSatisfy { $0.boundary == "runtime-null-now-outside-declared-packet" && $0.numeric["now"] == nil })
        var exercised = 0
        for item in fixture.cases where item.boundary == nil {
            try autoreleasepool {
                exercised += 1
                let settings = try options(item), engine = Pele.Engine()
                for lane in 0..<3 {
                    let trace = NumberTrace(), sentinel = NumberClockFailure()
                    let host = try resources(item: item, fixture: fixture, trace: trace, failure: sentinel)
                    if lane < 2 {
                        let retained = lane == 1 ? try engine.parse(item.source) : nil
                        if let error = item.expected.error {
                            XCTAssertEqual(error.name, "Error", item.id)
                            XCTAssertEqual(error.message, "render-numbers host clock stopped", item.id)
                            XCTAssertThrowsError(try retained.map { try engine.render($0, options: settings, resources: host) } ?? engine.render(item.source, options: settings, resources: host), item.id) {
                                XCTAssertTrue(($0 as? NumberClockFailure) === sentinel, item.id)
                            }
                        } else {
                            let rendered = try retained.map { try engine.render($0, options: settings, resources: host) } ?? engine.render(item.source, options: settings, resources: host)
                            try equal(rendered, expected: item.expected, id: "\(item.id)/\(lane)")
                        }
                    } else {
                        let document = try engine.parseMutable(item.source); defer { try? document.close() }
                        XCTAssertEqual(arithmetic(canonical(try document.snapshot().graph)), arithmetic(item.expected.graphBefore), item.id)
                        let beforeRender = document.revision
                        if item.expected.error != nil {
                            XCTAssertThrowsError(try document.renderOperation(options: settings, resources: host), item.id) { XCTAssertTrue(($0 as? NumberClockFailure) === sentinel, item.id) }
                            try assertPublishedRevision(document, after: beforeRender, item.id)
                        } else {
                            let outcome = try document.renderOperation(options: settings, resources: host)
                            XCTAssertEqual(outcome.revision, try assertPublishedRevision(document, after: beforeRender, item.id), item.id)
                            guard case .rendered(let rendered) = outcome.result else { return XCTFail("Expected drawing: \(item.id)") }
                            try equal(rendered, expected: item.expected, id: "\(item.id)/mutable")
                        }
                        XCTAssertEqual(arithmetic(canonical(try document.snapshot().graph)), arithmetic(item.expected.graphAfter), item.id)
                        if item.expected.error != nil {
                            var recovered = settings; recovered.now = 1_704_067_200_000
                            var safe = Pele.HostResources(); safe.timeZone = TimeZone(secondsFromGMT: 0)!
                            let beforeRecovery = document.revision
                            XCTAssertEqual(try document.render(options: recovered, resources: safe).type, item.family, item.id)
                            try assertPublishedRevision(document, after: beforeRecovery, item.id)
                        }
                    }
                    XCTAssertEqual(arithmetic(trace.value), arithmetic(item.expected.calls), "\(item.id)/\(lane)")
                }
            }
        }
        XCTAssertEqual(exercised, 287)
    }
    func testAllThirtyThreeFamiliesPreserveScalarOptionsInAllNativeLanes() throws {
        let fixture = try fixture("all-render-numbers.json.gz")
        XCTAssertEqual(fixture.cases.count, 1023)
        XCTAssertEqual(Set(fixture.cases.map(\.family)).count, 33)
        XCTAssertTrue(fixture.cases.allSatisfy { $0.boundary == nil && $0.expected.error == nil })
        var renderedCount = 0
        for item in fixture.cases {
            try autoreleasepool {
                let settings = try options(item), engine = Pele.Engine()
                for lane in 0..<3 {
                    let trace = NumberTrace(), sentinel = NumberClockFailure()
                    let host = try resources(item: item, fixture: fixture, trace: trace, failure: sentinel)
                    let id = "\(item.id)/native-lane-\(lane)"
                    if lane == 0 {
                        try equal(engine.render(item.source, options: settings, resources: host), expected: item.expected, id: id)
                    } else if lane == 1 {
                        let document = try engine.parse(item.source)
                        try equal(engine.render(document, options: settings, resources: host), expected: item.expected, id: id)
                    } else {
                        let document = try engine.parseMutable(item.source)
                        defer { try? document.close() }
                        let beforeRender = document.revision
                        let outcome = try document.renderOperation(options: settings, resources: host)
                        XCTAssertEqual(outcome.revision, try assertPublishedRevision(document, after: beforeRender, id), id)
                        guard case .rendered(let drawing) = outcome.result else {
                            return XCTFail("Expected pinned drawing: \(id)")
                        }
                        try equal(drawing, expected: item.expected, id: id)
                    }
                    XCTAssertEqual(arithmetic(trace.value), arithmetic(item.expected.calls), id)
                    renderedCount += 1
                }
            }
        }
        XCTAssertEqual(renderedCount, 3069)
    }
    func testDateTimeClipAndRawOptionOwnership() throws {
        for milliseconds in [1234.75, -1234.75, 0.0, -0.0, 8.64e15, 8.64e15 + 1, Double.infinity, Double.nan] {
            let date = Date(timeIntervalSince1970: milliseconds / 1000)
            var options = Pele.Engine.Options(); options.setNow(date: date)
            let actual = try XCTUnwrap(options.now), value = date.timeIntervalSince1970 * 1000
            if value.isFinite && abs(value) <= 8.64e15 {
                let clipped = value.rounded(.towardZero)
                XCTAssertEqual(actual.bitPattern, (clipped == 0 ? Double(0) : clipped).bitPattern)
            } else { XCTAssertTrue(actual.isNaN) }
        }
        var raw = Pele.Engine.Options(); raw.now = -0.5; raw.fontSize = Double(bitPattern: 0x7ff8000000000042)
        XCTAssertThrowsError(try JSONEncoder().encode(raw), "Options Codable remains unchanged outside render")
        _ = try Pele.Engine().render("pie\n\"A\":1", options: raw)
        XCTAssertEqual(raw.now?.bitPattern, (-0.5).bitPattern); XCTAssertEqual(raw.fontSize?.bitPattern, 0x7ff8000000000042)
        raw.config = ["nonfinite": .number(.nan)]
        XCTAssertThrowsError(try Pele.Engine().render("pie\n\"A\":1", options: raw), "Unmasked ConfigValue still uses its original encoder")
    }
    func testExactOutputCompositionAndOriginalHostErrorsPublishState() throws {
        let source = "gantt\ndateFormat YYYY-MM-DD\nsection S\nA :a,2024-01-01,1d"
        let engine = Pele.Engine(), immutable = try engine.parse(source), document = try engine.parseMutable(source)
        defer { try? document.close() }
        var settings = Pele.Engine.Options(); settings.now = .nan; settings.outputLimit = -1
        var host = Pele.HostResources(); host.timeZone = TimeZone(secondsFromGMT: 0)!
        host.nowMillis = { 1_704_067_200_000.5 }
        for rendered in [try engine.render(source, outputLimit: .number(.nan), options: settings, resources: host), try engine.render(immutable, outputLimit: .number(.infinity), options: settings, resources: host)] { XCTAssertEqual(rendered.type, "gantt") }
        let beforeRender = document.revision
        let failed = try document.renderOperation(outputLimit: .number(-0.0), options: settings, resources: host)
        XCTAssertEqual(failed.revision, try assertPublishedRevision(document, after: beforeRender))
        guard case .thrown(.pele(let error)) = failed.result else { return XCTFail("Expected output-limit result") }
        XCTAssertEqual(error.message, "Diagram output is longer than the limit of 0 characters.")
        let clock = NumberClockFailure(); host.nowMillis = { throw clock }
        XCTAssertThrowsError(try document.renderOperation(outputLimit: .undefined, options: settings, resources: host)) { XCTAssertTrue(($0 as? NumberClockFailure) === clock) }
        let clockRevision = try assertPublishedRevision(document, after: failed.revision)
        host.nowMillis = { 1_704_067_200_000.5 }; let width = NumberWidthFailure(); host.textWidth = { _ in throw width }
        XCTAssertThrowsError(try document.renderOperation(outputLimit: .undefined, options: settings, resources: host)) { XCTAssertTrue(($0 as? NumberWidthFailure) === width) }
        let widthRevision = try assertPublishedRevision(document, after: clockRevision)
        let scheduled = try document.snapshot().graph
        guard case .number(let scheduledNow) = try scheduled.field("now"),
              case .ref(let tasks) = try scheduled.field("tasks"),
              case .ref(let task) = try scheduled.field("0", on: tasks),
              case .bool(true) = try scheduled.field("processed", on: task),
              case .ref(let byId) = try scheduled.field("byId"),
              case .map(let entries, _) = scheduled.nodes[Int(byId)] else { return XCTFail("Scheduling mutations lost after text error") }
        XCTAssertEqual(scheduledNow.bitPattern, Double(1_704_067_200_000.5).bitPattern)
        guard case .ref(let cached) = entries.first?.value else { return XCTFail("Lost scheduled task cache") }
        XCTAssertEqual(task, cached)
        host.textWidth = nil
        XCTAssertEqual(try document.render(outputLimit: .null, options: settings, resources: host).type, "gantt")
        try assertPublishedRevision(document, after: widthRevision)
    }
}
