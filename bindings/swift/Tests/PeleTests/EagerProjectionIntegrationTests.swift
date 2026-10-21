import Foundation
import XCTest
import Pele
import CPele

/// Differential public feature slice: legacy C JSON contract versus Swift's eager
/// immutable API, including retained operations. No @testable or builder access.
final class EagerProjectionIntegrationTests: XCTestCase {
    private struct Envelope<T: Decodable>: Decodable {
        let version: Int
        let ok: Bool
        let result: T?
        let error: CompatibilityError?
    }
    private struct LegacyProjection: Decodable { let type: String; let model: ConfigValue }
    private func decode<T: Decodable>(_ handle: OpaquePointer, as: T.Type) throws -> T {
        let count = pele_result_len(handle)
        guard count <= 64 * 1024 * 1024, let bytes = pele_result_data(handle) else {
            throw PeleError(code: "internal", message: "Invalid native compatibility result")
        }
        let envelope = try JSONDecoder().decode(Envelope<T>.self, from: Data(bytes: bytes, count: count))
        XCTAssertEqual(envelope.version, 2)
        if let error = envelope.error { throw error }
        XCTAssertEqual(pele_result_status(handle), 0)
        XCTAssertTrue(envelope.ok)
        return try XCTUnwrap(envelope.result)
    }
    private func legacy(_ source: String, options: Pele.Engine.ParseOptions = .init()) throws -> OpaquePointer {
        let source = Data(source.utf8), options = try JSONEncoder().encode(options)
        return try XCTUnwrap(source.withUnsafeBytes { input in options.withUnsafeBytes { settings in
            pele_parse_v2(input.bindMemory(to: UInt8.self).baseAddress, input.count,
                          settings.bindMemory(to: UInt8.self).baseAddress, settings.count)
        }})
    }
    private func assertExact(_ actual: ConfigValue, _ expected: ConfigValue,
                             file: StaticString = #filePath, line: UInt = #line) {
        switch (actual, expected) {
        case (.number(let a), .number(let b)):
            XCTAssertEqual(a.bitPattern, b.bitPattern, file: file, line: line)
        case (.string(let a), .string(let b)):
            XCTAssertEqual(Array(a.utf16), Array(b.utf16), file: file, line: line)
        case (.array(let a), .array(let b)):
            XCTAssertEqual(a.count, b.count, file: file, line: line)
            for (a, b) in zip(a, b) { assertExact(a, b, file: file, line: line) }
        case (.object(let a), .object(let b)):
            XCTAssertEqual(a.count, b.count, file: file, line: line)
            for (key, value) in b {
                guard let actualKey = a.keys.first(where: { $0 == key }), let actualValue = a[actualKey] else {
                    XCTFail("Missing key \(key)", file: file, line: line); continue
                }
                XCTAssertEqual(Array(actualKey.utf16), Array(key.utf16), file: file, line: line)
                assertExact(actualValue, value, file: file, line: line)
            }
        default: XCTAssertEqual(actual, expected, file: file, line: line)
        }
    }
    private func compare(_ source: String, options: Pele.Engine.ParseOptions = .init()) throws -> Pele.Document {
        let old = try legacy(source, options: options)
        defer { pele_result_free(old) }
        let expected = try decode(old, as: LegacyProjection.self)
        // Engine is discarded before reading the model or running any operation.
        let document = try Pele.Engine().parse(source, options: options)
        XCTAssertEqual(document.type, expected.type)
        assertExact(document.model, expected.model)
        assertExact(try document.decodeModel(as: ConfigValue.self), expected.model)
        return document
    }
    private func menu(_ payload: String) -> String {
        "sequenceDiagram\nparticipant A\nproperties A: {\"payload\":\(payload)}\nA->>A: Ready"
    }
    func testJSONNormalizationUnicodeAndBatchBoundariesThroughPublicParse() throws {
        let payloads = [
            #"{"é":1,"e\u0301":2,"2":20,"1":10,"0":0,"01":1,"4294967295":5}"#,
            #"{"e\u0301":2,"é":1,"2":20,"1":10}"#,
            #"{"same":1,"same":2,"é":null,"e\u0301":3}"#,
            #"{"numbers":[-0,0,1e400,-1e400,9007199254740993,18446744073709551615,5e-324,1.7976931348623157e308,0.10000000000000002]}"#,
            #"{"strings":["","a\u0000b","\ud83d\ude80","é","e\u0301","\\\"\n\t"],"boolean":[false,true],"empty":[{},[],null]}"#,
            #"{"\ufeffkey":"\ufeffvalue","strings":["\ufeff","\ufeff\ufefftext","text\ufeff"]}"#,
            "[" + (0..<400).map { "{\"key\":\($0),\"nested\":[true,null,\"😀\"]}" }.joined(separator: ",") + "]",
            String(repeating: "[", count: 110) + "null" + String(repeating: "]", count: 110)
        ]
        for payload in payloads { _ = try compare(menu(payload)) }
    }
    func testRetainedRenderConfigAndQueryAfterEngineLifetime() throws {
        let source = "---\nconfig:\n  pie:\n    textPosition: 0.8\n---\npie\n\"One🚀\":2\n\"Two\":3"
        var options = Pele.Engine.ParseOptions()
        options.config = ["pie": .object(["showLegend": .bool(false)])]
        let document = try compare(source, options: options)
        let old = try legacy(source, options: options)
        defer { pele_result_free(old) }
        let rendered = try Pele.Engine().render(document)
        let oldRender = try XCTUnwrap(pele_render_document_v2(old, nil, 0))
        defer { pele_result_free(oldRender) }
        let expected = try decode(oldRender, as: FullRenderedDiagram.self)
        XCTAssertEqual(rendered.svg, expected.svg)
        XCTAssertEqual(rendered.width.bitPattern, expected.width.bitPattern)
        XCTAssertEqual(rendered.height.bitPattern, expected.height.bitPattern)
        let operation = Data("unsupported-projection-query".utf8)
        let oldQuery = try XCTUnwrap(operation.withUnsafeBytes { name in
            pele_document_query_v2(old, name.bindMemory(to: UInt8.self).baseAddress, name.count, nil, 0)
        })
        defer { pele_result_free(oldQuery) }
        var expectedError: CompatibilityError?
        do { _ = try decode(oldQuery, as: ConfigValue.self); XCTFail("Query should fail") }
        catch let error as CompatibilityError { expectedError = error }
        XCTAssertThrowsError(try document.query("unsupported-projection-query")) { error in
            guard let actual = error as? CompatibilityError, let expectedError else { return XCTFail("Wrong query error") }
            XCTAssertEqual(actual.message, expectedError.message)
            XCTAssertEqual(actual.code, expectedError.code)
        }
        // Escaped model remains an eager immutable value after native handle dies.
        let escapedModel: ConfigValue = try Pele.Engine().parse("pie\n\"A\":1").model
        XCTAssertNoThrow(try JSONEncoder().encode(escapedModel))
    }
    func testParseFailuresStayEagerAndMatchLegacy() throws {
        var tiny = Pele.Engine.ParseOptions(); tiny.limit = 3
        let cases: [(String, Pele.Engine.ParseOptions)] = [
            ("pie\n\"bad\":-1", .init()),
            ("pie\n\"A\":1", tiny),
            (menu(String(repeating: "[", count: 140) + "0" + String(repeating: "]", count: 140)), .init())
        ]
        for (source, options) in cases {
            let old = try legacy(source, options: options)
            defer { pele_result_free(old) }
            var expected: CompatibilityError?
            do { _ = try decode(old, as: LegacyProjection.self); XCTFail("Expected legacy failure") }
            catch let error as CompatibilityError { expected = error }
            XCTAssertThrowsError(try Pele.Engine().parse(source, options: options)) { error in
                guard let actual = error as? CompatibilityError, let expected else { return XCTFail("Wrong parse error: \(error)") }
                XCTAssertEqual(actual.name, expected.name)
                XCTAssertEqual(actual.message, expected.message)
                XCTAssertEqual(actual.code, expected.code)
                XCTAssertEqual(actual.type, expected.type)
                XCTAssertEqual(actual.line, expected.line)
                XCTAssertEqual(actual.column, expected.column)
                XCTAssertEqual(actual.snippet, expected.snippet)
            }
        }
    }
    func testProjectionDepthBoundaryMatchesLegacyAdmission() throws {
        var accepted = 0, rejected = 0
        for depth in 120...130 {
            let source = menu(String(repeating: "[", count: depth) + "0" + String(repeating: "]", count: depth))
            let old = try legacy(source)
            defer { pele_result_free(old) }
            switch Result(catching: { try decode(old, as: LegacyProjection.self) }) {
            case .success(let expected):
                accepted += 1
                let document = try Pele.Engine().parse(source)
                assertExact(document.model, expected.model)
            case .failure(let error):
                rejected += 1
                guard let expected = error as? CompatibilityError else { throw error }
                XCTAssertThrowsError(try Pele.Engine().parse(source)) { actual in
                    guard let actual = actual as? CompatibilityError else { return XCTFail("Wrong depth error") }
                    XCTAssertEqual(actual.code, expected.code)
                    XCTAssertEqual(actual.message, expected.message)
                }
            }
        }
        XCTAssertGreaterThan(accepted, 0)
        XCTAssertGreaterThan(rejected, 0)
    }
    func testExpandedModelBudgetFailsBeforePublicParseReturns() throws {
        // Gantt projects the same task under tasks and byId. Source stays below
        // the 16 MiB transport gate while its eager model exceeds 16 MiB.
        let source = "gantt\ndateFormat YYYY-MM-DD\n" + String(repeating: "x", count: 9 * 1024 * 1024)
            + " :task, 2025-01-01, 1d"
        var options = Pele.Engine.ParseOptions(); options.limit = 16 * 1024 * 1024
        let old = try legacy(source, options: options)
        defer { pele_result_free(old) }
        var expected: CompatibilityError?
        do { _ = try decode(old, as: LegacyProjection.self); XCTFail("Expected model budget failure") }
        catch let error as CompatibilityError { expected = error }
        XCTAssertEqual(expected?.code, "limit")
        XCTAssertTrue(expected?.message.contains("Parsed model JSON exceeds") == true)
        XCTAssertThrowsError(try Pele.Engine().parse(source, options: options)) { error in
            guard let actual = error as? CompatibilityError else { return XCTFail("Wrong projection budget error") }
            XCTAssertEqual(actual.message, expected?.message)
            XCTAssertEqual(actual.code, expected?.code)
            XCTAssertEqual(actual.type, expected?.type)
        }
    }
    func testCMetadataSizeAbortAndOwnedHandleContract() throws {
        struct Metadata: Decodable { let type: String; let projectionBytes: Int }
        let source = Data("sequenceDiagram\nparticipant A\nA->>A: Hello".utf8)
        let old = try legacy(String(decoding: source, as: UTF8.self))
        defer { pele_result_free(old) }
        let eager = try XCTUnwrap(source.withUnsafeBytes { input in
            pele_parse_eager_v1(input.bindMemory(to: UInt8.self).baseAddress, input.count, nil, 0, nil,
                { _, events, count in events != nil && count > 0 && count <= 128 ? 0 : 1 })
        })
        defer { pele_result_free(eager) }
        let metadata = try decode(eager, as: Metadata.self)
        XCTAssertEqual(metadata.type, "sequence")
        XCTAssertEqual(metadata.projectionBytes, pele_result_len(old))
        XCTAssertLessThan(pele_result_len(eager), 160)
        let aborted = try XCTUnwrap(source.withUnsafeBytes { input in
            pele_parse_eager_v1(input.bindMemory(to: UInt8.self).baseAddress, input.count, nil, 0, nil,
                { _, _, _ in 1 })
        })
        defer { pele_result_free(aborted) }
        XCTAssertThrowsError(try decode(aborted, as: Metadata.self))
        let badRender = try XCTUnwrap(pele_render_document_v2(aborted, nil, 0))
        defer { pele_result_free(badRender) }
        XCTAssertThrowsError(try decode(badRender, as: FullRenderedDiagram.self))
        let goodRender = try XCTUnwrap(pele_render_document_v2(eager, nil, 0))
        defer { pele_result_free(goodRender) }
        XCTAssertNoThrow(try decode(goodRender, as: FullRenderedDiagram.self))
    }

    func testCAbortAfterPublishedBatchesDiscardsDocumentAndKeepsParseErrorOrder() throws {
        let source = Data(("sequenceDiagram\nparticipant A\n" +
            (0..<300).map { "A->>A: Message \($0)" }.joined(separator: "\n")).utf8)
        var callbacks = 0
        let aborted = try XCTUnwrap(withUnsafeMutablePointer(to: &callbacks) { counter in
            source.withUnsafeBytes { input in
                pele_parse_eager_v1(input.bindMemory(to: UInt8.self).baseAddress, input.count,
                    nil, 0, UnsafeMutableRawPointer(counter), { context, events, count in
                        guard let context, events != nil, count > 0, count <= 128 else { return 1 }
                        let counter = context.assumingMemoryBound(to: Int.self)
                        counter.pointee += 1
                        return counter.pointee < 2 ? 0 : 1
                    })
            }
        })
        defer { pele_result_free(aborted) }
        XCTAssertEqual(callbacks, 2)
        XCTAssertNotEqual(pele_result_status(aborted), 0)
        let render = try XCTUnwrap(pele_render_document_v2(aborted, nil, 0))
        defer { pele_result_free(render) }
        XCTAssertThrowsError(try decode(render, as: FullRenderedDiagram.self))

        let invalid = "pie\n\"bad\":-1"
        let old = try legacy(invalid)
        defer { pele_result_free(old) }
        var expected: CompatibilityError?
        do { _ = try decode(old, as: LegacyProjection.self); XCTFail("Expected parser error") }
        catch let error as CompatibilityError { expected = error }
        let input = Data(invalid.utf8)
        let missingCallback = try XCTUnwrap(input.withUnsafeBytes { bytes in
            pele_parse_eager_v1(bytes.bindMemory(to: UInt8.self).baseAddress, bytes.count,
                nil, 0, nil, nil)
        })
        defer { pele_result_free(missingCallback) }
        do { _ = try decode(missingCallback, as: LegacyProjection.self); XCTFail("Expected parser error") }
        catch let error as CompatibilityError {
            XCTAssertEqual(error.code, expected?.code)
            XCTAssertEqual(error.message, expected?.message)
        }
    }
}
