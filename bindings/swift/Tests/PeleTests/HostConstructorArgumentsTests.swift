import Foundation
import XCTest
import Pele

private final class HostArgumentsError: Error {}
private final class HostArgumentsObject: Pele.PublicHostCallable {
    var get: (Pele.JSString) throws -> Pele.PublicHostValue = { _ in .undefined }
    var call: (Pele.PublicHostValue, [Pele.PublicHostValue]) throws -> Pele.PublicHostValue = { _, _ in .undefined }
    var primitive: (Pele.PublicHostHint) throws -> Pele.PublicHostValue = { _ in .string(.init("[object Object]")) }
    func publicGet(_ key: Pele.JSString) throws -> Pele.PublicHostValue { try get(key) }
    func publicCall(receiver: Pele.PublicHostValue, arguments: [Pele.PublicHostValue]) throws -> Pele.PublicHostValue { try call(receiver, arguments) }
    func publicPrimitive(_ hint: Pele.PublicHostHint) throws -> Pele.PublicHostValue { try primitive(hint) }
}
private enum HostArgumentsDocument {
    case mutable(Pele.MutableDocument), prepared(Pele.PreparedDocument)
    var revision: UInt64 { switch self { case .mutable(let d): d.revision; case .prepared(let d): d.revision } }
    func snapshot() throws -> Pele.PublicSnapshot { switch self { case .mutable(let d): try d.snapshot(); case .prepared(let d): try d.snapshot() } }
    func apply(_ graph: Pele.PublicGraph) throws { switch self { case .mutable(let d): _ = try d.apply(graph, expectedRevision: d.revision); case .prepared(let d): _ = try d.apply(graph, expectedRevision: d.revision) } }
    func invoke(_ method: UInt32, _ arguments: [Pele.PublicGraph.Atom]) throws -> Pele.PublicInvocationOutcome {
        let root = try snapshot().graph.root
        switch self { case .mutable(let d): return try d.invoke(method: method, receiver: root, arguments: arguments); case .prepared(let d): return try d.invoke(method: method, receiver: root, arguments: arguments) }
    }
    func retain(_ value: Pele.PublicHostValue) throws -> Pele.PublicGraph.Atom { switch self { case .mutable(let d): try d.retainHostValue(value); case .prepared(let d): try d.retainHostValue(value) } }
    func resolve(_ atom: Pele.PublicGraph.Atom) throws -> Pele.PublicHostValue { switch self { case .mutable(let d): try d.resolveHostValue(atom); case .prepared(let d): try d.resolveHostValue(atom) } }
    func close() throws { switch self { case .mutable(let d): try d.close(); case .prepared(let d): try d.close() } }
}
final class HostConstructorArgumentsTests: XCTestCase {
    private func fixture(_ name: String, id: String) throws -> [String: Any] {
        var url = URL(fileURLWithPath: #filePath)
        for _ in 0..<5 { url.deleteLastPathComponent() }
        let body = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: url.appendingPathComponent("fixtures/" + name))) as? [String: Any])
        XCTAssertEqual(body["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        return try XCTUnwrap((body["cases"] as? [[String: Any]])?.first { $0["id"] as? String == id })
    }
    private func returned(_ outcome: Pele.PublicInvocationOutcome) throws {
        if case .thrown(let error) = outcome.result { throw PeleError(code: "test", message: "Unexpected operation error: \(error)") }
    }
    private func vertex(_ document: HostArgumentsDocument, _ label: String, styled: Bool) throws -> Pele.PublicInvocationOutcome {
        var graph = try document.snapshot().graph
        let label = try graph.intern(label)
        var arguments: [Pele.PublicGraph.Atom] = [.string(label)]
        if styled {
            let style = UInt32(graph.nodes.count); graph.nodes.append(.array(length: 0, properties: []))
            arguments += [.undefined, .undefined, .ref(style)]
        }
        try document.apply(graph)
        return try document.invoke(1, arguments)
    }
    func testFlowAndSwimlaneOptionsStayLiveAcrossNestedHelpersOriginalThrowsAndReplacement() throws {
        let expected = try fixture("public-flow-arguments.json", id: "warn-reentry")
        let trace = try XCTUnwrap(expected["trace"] as? [[Any]])
        let message = try XCTUnwrap(trace.last?.last as? String)
        for family in ["flowchart", "swimlane"] { for prepared in [false, true] {
            let registry = try Pele.Registry.allBuiltins()
            let factory = try XCTUnwrap(registry.captureConstructor(for: family, prototype: 1))
            let options = HostArgumentsObject(), warning = HostArgumentsObject()
            var reads: [String] = [], warnings: [String] = []
            options.get = { key in reads.append(key.stringReplacingInvalidCodeUnits); return key == .init("warn") ? .host(warning) : .undefined }
            let document: HostArgumentsDocument = try prepared ? .prepared(factory.constructPrepared(firstArgument: .host(options))) : .mutable(factory.construct(firstArgument: .host(options)))
            XCTAssertTrue(reads.isEmpty, "Constructor must not read retained options")
            factory.close(); try registry.close()
            defer { try? document.close() }
            guard case .host(let retained) = try document.resolve(XCTUnwrap(try document.snapshot().graph.field("options"))) else { return XCTFail("Retained options missing") }
            XCTAssertTrue(retained === options)
            warning.call = { receiver, arguments in
                guard case .host(let receiver) = receiver else { throw PeleError(code: "test", message: "Warning receiver lost") }
                XCTAssertTrue(receiver === options)
                guard case .string(let value) = arguments.first else { throw PeleError(code: "test", message: "Warning argument lost") }
                warnings.append(value.stringReplacingInvalidCodeUnits)
                XCTAssertThrowsError(try document.close())
                try self.returned(self.vertex(document, "B", styled: false))
                return .number(Double(bitPattern: 0x8000000000000000))
            }
            try returned(vertex(document, "A", styled: true))
            XCTAssertEqual(reads, ["warn"]); XCTAssertEqual(warnings, [message])
            let graph = try document.snapshot().graph
            guard case .ref(let vertices) = try graph.field("nodes") else { return XCTFail("Missing vertices") }
            if case .map(let entries, _) = graph.nodes[Int(vertices)] { XCTAssertEqual(entries.count, 2) } else { XCTFail("Missing vertex Map") }
            let original = HostArgumentsError()
            warning.call = { _, _ in try self.returned(self.vertex(document, "D", styled: false)); throw original }
            XCTAssertThrowsError(try vertex(document, "C", styled: true)) { XCTAssertTrue(($0 as AnyObject) === original) }
            let partial = try document.snapshot()
            XCTAssertEqual(document.revision, partial.revision)
            guard case .ref(let id) = try partial.graph.field("nodes"), case .map(let entries, _) = partial.graph.nodes[Int(id)] else { return XCTFail("Missing partial graph") }
            XCTAssertEqual(entries.count, 3, "Nested D survives; outer C stops before insertion")
            var repair = partial.graph; try repair.setField("options", value: .null); try document.apply(repair)
            let nullOptions = try vertex(document, "E", styled: true)
            if case .thrown(.javaScript(let name, _)) = nullOptions.result { XCTAssertEqual(name, "TypeError") }
            else { XCTFail("Null options must preserve native JavaScript TypeError") }
            try repair.setField("options", value: document.retain(.host(options))); try document.apply(repair)
            warning.call = { _, _ in .undefined }
            try returned(vertex(document, "E", styled: true))
        }}
    }
    func testGanttWarningCapturesSecondCallableBeforeProducerAndForwardsLosslessReturn() throws {
        _ = try fixture("public-gantt-constructor.json", id: "changing-options")
        for prepared in [false, true] {
            let registry = try Pele.Registry.allBuiltins(), options = HostArgumentsObject()
            let first = HostArgumentsObject(), second = HostArgumentsObject(), producer = HostArgumentsObject()
            let factory = try XCTUnwrap(registry.captureConstructor(for: "gantt", prototype: 1))
            let document: HostArgumentsDocument = try prepared ? .prepared(factory.constructPrepared(firstArgument: .host(options))) : .mutable(factory.construct(firstArgument: .host(options)))
            defer { try? document.close(); try? registry.close() }; factory.close()
            var events: [String] = [], gets = 0
            let malformed = Pele.JSString(codeUnits: [0xd800, 0, 0xdc00])
            options.get = { key in XCTAssertEqual(key, .init("warn")); gets += 1; events.append("get\(gets)"); return .host(gets == 1 ? first : second) }
            first.call = { _, _ in XCTFail("First warning only tested for truthiness"); return .undefined }
            second.call = { receiver, args in
                events.append("warn")
                guard case .host(let owner) = receiver, case .string(let text) = args.first else { throw PeleError(code: "test", message: "Warning forwarding failed") }
                XCTAssertTrue(owner === options); XCTAssertEqual(text, malformed)
                return .host(HostArgumentsObject())
            }
            producer.call = { receiver, args in
                if case .undefined = receiver {} else { XCTFail("Producer receiver") }
                XCTAssertTrue(args.isEmpty); events.append("producer")
                var graph = try document.snapshot().graph
                let section = try graph.intern("nested section"); try document.apply(graph)
                try self.returned(document.invoke(14, [.string(section)]))
                options.get = { _ in XCTFail("Captured second callable must be reused"); return .undefined }
                return .string(malformed)
            }
            let producerAtom = try document.retain(.host(producer))
            try returned(document.invoke(24, [producerAtom]))
            XCTAssertEqual(events, ["get1", "get2", "producer", "warn"])
            let final = try document.snapshot()
            XCTAssertEqual(final.revision, document.revision)
            XCTAssertTrue(final.graph.strings.contains(.init("nested section")))
            XCTAssertFalse(final.graph.strings.contains(malformed), "Producer return must not be copied into model strings")
        }
    }
}
