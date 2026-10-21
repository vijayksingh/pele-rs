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
    private func blockInput(_ value: Any, graph: inout Pele.PublicGraph) throws -> Pele.PublicGraph.Atom {
        if value is NSNull { return .undefined }
        if let text = value as? String { return .string(try graph.intern(text)) }
        if let number = value as? NSNumber { return .number(number.doubleValue) }
        var fields: [(String, Any)], length: UInt32?
        if let array = value as? [Any] { fields = array.enumerated().map { (String($0.offset), $0.element) }; length = UInt32(array.count) }
        else if let object = value as? [String: Any] { fields = object.sorted { $0.key < $1.key }.map { ($0.key, $0.value) } }
        else { throw PeleError(code: "test", message: "Unexpected Block fixture value") }
        let id = UInt32(graph.nodes.count); graph.nodes.append(.object(properties: []))
        var properties: [Pele.PublicGraph.Property] = []
        for (name, value) in fields {
            let key = try graph.intern(name), atom = try blockInput(value, graph: &graph)
            properties.append(.init(key: key, value: atom))
        }
        graph.nodes[Int(id)] = length.map { .array(length: $0, properties: properties) } ?? .object(properties: properties)
        return .ref(id)
    }
    private func blockState(_ graph: Pele.PublicGraph, statements: Pele.PublicGraph.Atom) throws -> [String: Any] {
        func id(_ atom: Pele.PublicGraph.Atom) throws -> UInt32 {
            guard case .ref(let id) = atom else { throw PeleError(code: "test", message: "Expected Block reference") }; return id
        }
        func at(_ atom: Pele.PublicGraph.Atom, _ key: String) throws -> Pele.PublicGraph.Atom { try graph.field(key, on: id(atom)) ?? .undefined }
        func list(_ atom: Pele.PublicGraph.Atom) throws -> [Pele.PublicGraph.Atom] {
            guard case .array(let length, _) = graph.nodes[Int(try id(atom))] else { throw PeleError(code: "test", message: "Expected Block array") }
            return try (0..<length).map { try at(atom, String($0)) }
        }
        func value(_ atom: Pele.PublicGraph.Atom) throws -> Any {
            switch atom {
            case .undefined, .null: return NSNull()
            case .bool(let v): return v
            case .number(let v): return v
            case .string(let i): return try XCTUnwrap(graph.strings[Int(i)].stringIfWellFormed)
            case .ref(let i):
                if case .array = graph.nodes[Int(i)] { return try list(atom).map(value) }
                var object: [String: Any] = [:]
                for property in graph.nodes[Int(i)].properties {
                    if case .undefined = property.value { continue }
                    object[try XCTUnwrap(graph.strings[Int(property.key)].stringIfWellFormed)] = try value(property.value)
                }
                return object
            default: throw PeleError(code: "test", message: "Unexpected callable in Block state")
            }
        }
        func same(_ a: Pele.PublicGraph.Atom, _ b: Pele.PublicGraph.Atom) -> Bool {
            if case .ref(let a) = a, case .ref(let b) = b { return a == b }; return false
        }
        let root = try at(graph.root, "root"), blocks = try at(graph.root, "blocks")
        guard case .map(let entries, _) = graph.nodes[Int(try id(at(graph.root, "byId")))],
              case .set(let pending, _) = graph.nodes[Int(try id(at(graph.root, "pending")))] else { throw PeleError(code: "test", message: "Missing Block collections") }
        var aliases: [[Any]] = []
        for statement in try list(statements) {
            guard let name = try value(at(statement, "id")) as? String else { continue }
            if let entry = try entries.first(where: { try value($0.key) as? String == name }) { aliases.append([name, same(entry.value, statement)]) }
        }
        return ["root": try value(root), "blocks": try value(blocks), "edges": try value(at(graph.root, "edges")),
                "byId": try entries.map { [try value($0.key), try value($0.value)] }, "pending": try pending.map(value),
                "ids": try value(at(graph.root, "ids")), "colors": try value(at(graph.root, "colors")), "spaces": try value(at(graph.root, "spaces")),
                "statements": try value(statements), "blocksAliasRoot": same(blocks, try at(root, "children")), "byIdAlias": aliases]
    }
    func testBlockRetainedWarningPreservesNestedHelpersPartialHierarchyAndRecovery() throws {
        var path = URL(fileURLWithPath: #filePath); for _ in 0..<5 { path.deleteLastPathComponent() }
        let oracle = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-block-live-warning.json"))) as? [String: Any])
        XCTAssertEqual(oracle["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        let cases = try XCTUnwrap(oracle["cases"] as? [[String: Any]])
        for mode in ["normal", "throw", "mutate", "nested"] {
            let expected = try XCTUnwrap(cases.first { $0["mode"] as? String == mode })
            let expectedTrace = try XCTUnwrap(expected["trace"] as? [[Any]])
            let input = try XCTUnwrap((expectedTrace.first?[3] as? [String: Any])?["statements"])
            let registry = try Pele.Registry.allBuiltins(), options = HostArgumentsObject(), warning = HostArgumentsObject()
            let factory = try XCTUnwrap(registry.captureConstructor(for: "block", prototype: 1))
            var gets = 0
            options.get = { key in gets += 1; XCTAssertEqual(key, .init("warn")); return .host(warning) }
            let document = try factory.construct(firstArgument: .host(options))
            factory.close(); try registry.close(); defer { try? document.close() }
            XCTAssertEqual(gets, 1)
            var graph = try document.snapshot().graph
            guard case .host(let retained) = try document.resolveHostValue(XCTUnwrap(graph.field("warn"))) else { return XCTFail("Missing retained warning") }
            XCTAssertTrue(retained === warning)
            let statements = try blockInput(input, graph: &graph), receiver = graph.root
            try document.apply(graph, expectedRevision: document.revision)
            let schema = try document.schema()
            let hierarchy = try XCTUnwrap(schema.methods.first { $0.name == "setHierarchy" }?.id)
            let named = try XCTUnwrap(schema.methods.first { $0.name == "named" }?.id)
            let original = HostArgumentsError(); var trace: [[Any]] = [], calls = 0
            warning.call = { current, args in
                guard case .graphReference(let currentID) = current, case .ref(let rootID) = receiver,
                      case .string(let message) = args.first else { throw PeleError(code: "test", message: "Block warning receiver/value lost") }
                XCTAssertEqual(currentID, rootID); calls += 1
                var graph = try document.snapshot().graph
                trace.append(["warn", currentID == rootID, message.stringReplacingInvalidCodeUnits, try self.blockState(graph, statements: statements)])
                XCTAssertThrowsError(try document.close())
                if mode == "throw" { throw original }
                if mode == "mutate" {
                    guard case .ref(let listID) = statements else { throw PeleError(code: "test", message: "Missing statements") }
                    if calls == 1 {
                        guard case .ref(let a) = try graph.field("1", on: listID) else { throw PeleError(code: "test", message: "Missing A") }
                        let kind = try graph.intern("column-setting"); try graph.setField("type", value: .string(kind), on: a)
                        try graph.setField("columns", value: .number(4), on: a); try document.apply(graph, expectedRevision: document.revision)
                    } else if calls == 2 {
                        guard case .ref(let b) = try graph.field("2", on: listID) else { throw PeleError(code: "test", message: "Missing B") }
                        let label = try graph.intern("Changed"), name = try graph.intern("Pending")
                        try graph.setField("label", value: .string(label), on: b); try document.apply(graph, expectedRevision: document.revision)
                        try self.returned(document.invoke(method: named, receiver: receiver, arguments: [.string(name)]))
                    }
                } else if mode == "nested" && calls == 1 {
                    let nested = try self.blockInput([["id": "Nested", "type": "square", "widthInColumns": 1]], graph: &graph)
                    try document.apply(graph, expectedRevision: document.revision)
                    try self.returned(document.invoke(method: hierarchy, receiver: receiver, arguments: [nested]))
                }
                return .host(HostArgumentsObject())
            }
            if mode == "throw" {
                XCTAssertThrowsError(try document.invoke(method: hierarchy, receiver: receiver, arguments: [statements])) { XCTAssertTrue(($0 as AnyObject) === original) }
            } else { try returned(document.invoke(method: hierarchy, receiver: receiver, arguments: [statements])) }
            XCTAssertEqual(gets, 1, "Warning is retained, not reacquired from constructor options")
            XCTAssertTrue(NSArray(array: trace).isEqual(to: expectedTrace), mode + " callback trace")
            XCTAssertTrue(NSDictionary(dictionary: try blockState(document.snapshot().graph, statements: statements)).isEqual(to: try XCTUnwrap(expected["after"] as? [String: Any])), mode + " partial state")
            var repair = try document.snapshot().graph; try repair.setField("warn", value: .undefined)
            try document.apply(repair, expectedRevision: document.revision)
            try returned(document.invoke(method: hierarchy, receiver: receiver, arguments: [statements]))
            XCTAssertTrue(NSDictionary(dictionary: try blockState(document.snapshot().graph, statements: statements)).isEqual(to: try XCTUnwrap(expected["recovered"] as? [String: Any])), mode + " recovered state")
            XCTAssertEqual(try document.snapshot().revision, document.revision)
        }
    }

}
