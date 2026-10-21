import Foundation
import XCTest
import Pele

private final class OrderedConstructorError: Error {}
private final class OrderedConstructorTrace: @unchecked Sendable {
    private let lock = NSLock()
    private var rows: [[String]] = []
    func append(_ row: [String]) { lock.lock(); defer { lock.unlock() }; rows.append(row) }
    func snapshot() -> [[String]] { lock.lock(); defer { lock.unlock() }; return rows }
}
private final class OrderedConstructorHost: Pele.PublicHostCallable {
    var get: (Pele.JSString) throws -> Pele.PublicHostValue = { _ in .undefined }
    var call: (Pele.PublicHostValue, [Pele.PublicHostValue]) throws -> Pele.PublicHostValue = { _, _ in .undefined }
    func publicGet(_ key: Pele.JSString) throws -> Pele.PublicHostValue { try get(key) }
    func publicCall(receiver: Pele.PublicHostValue, arguments: [Pele.PublicHostValue]) throws -> Pele.PublicHostValue { try call(receiver, arguments) }
}
final class OrderedConstructorArgumentsTests: XCTestCase {
    private func fixture(_ name: String) throws -> [String: Any] {
        var url = URL(fileURLWithPath: #filePath); for _ in 0..<5 { url.deleteLastPathComponent() }
        let body = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: url.appendingPathComponent("fixtures/" + name))) as? [String: Any])
        XCTAssertEqual(body["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        return body
    }
    private func text(_ graph: Pele.PublicGraph, _ atom: Pele.PublicGraph.Atom?) throws -> String {
        guard case .string(let id) = atom else { throw PeleError(code: "test", message: "Expected string field") }
        return try XCTUnwrap(graph.strings[Int(id)].stringIfWellFormed)
    }
    private func argument(_ value: Any, graph: inout Pele.PublicGraph) throws -> Pele.PublicGraph.Atom {
        if let text = value as? String { return .string(try graph.intern(text)) }
        if let number = value as? NSNumber { return .number(number.doubleValue) }
        guard let object = value as? [String: Any] else { throw PeleError(code: "test", message: "Unsupported oracle argument") }
        if object["undefined"] as? Bool == true { return .undefined }
        let id = UInt32(graph.nodes.count); graph.nodes.append(.object(properties: []))
        var properties: [Pele.PublicGraph.Property] = []
        if let array = object["array"] as? [Any] {
            for (index, item) in array.enumerated() {
                let key = try graph.intern(String(index)), atom = try argument(item, graph: &graph)
                properties.append(.init(key: key, value: atom))
            }
            graph.nodes[Int(id)] = .array(length: UInt32(array.count), properties: properties)
        } else if let fields = object["object"] as? [[Any]] {
            for field in fields {
                let key = try graph.intern(XCTUnwrap(field[0] as? String)), atom = try argument(field[1], graph: &graph)
                properties.append(.init(key: key, value: atom))
            }
            graph.nodes[Int(id)] = .object(properties: properties)
        } else { throw PeleError(code: "test", message: "Unsupported oracle object") }
        return .ref(id)
    }
    private func assertRaw(_ document: Pele.PreparedDocument, expected: [String: Any], family: String) throws {
        let trace = OrderedConstructorTrace()
        var resources = Pele.HostResources()
        resources.textWidth = { request in
            trace.append([request.text, String(format: "%016llx", request.size.bitPattern)])
            return Double(request.text.utf16.count) * request.size * 0.6
        }
        let drawing = try document.render(resources: resources)
        XCTAssertEqual(expected["ok"] as? Bool, true, family)
        XCTAssertEqual(drawing.svg, expected["svg"] as? String, family)
        XCTAssertEqual(String(format: "%016llx", drawing.width.bitPattern), expected["width"] as? String, family)
        XCTAssertEqual(String(format: "%016llx", drawing.height.bitPattern), expected["height"] as? String, family)
        XCTAssertEqual(trace.snapshot(), expected["trace"] as? [[String]], family)
    }
    func testOrderedOptionsConstructorsReadOnlyRequestedFieldsAndPreserveOriginalErrors() throws {
        let oracle = try fixture("public-ordered-constructors.json")
        let rows = try XCTUnwrap(oracle["cases"] as? [[String: Any]])
        for (family, keys) in [("sequence", ["wrap", "sequenceWrap"]), ("block", ["warn"]), ("agentflow", ["lineOffset"])] {
            let registry = try Pele.Registry.allBuiltins()
            let factory = try XCTUnwrap(registry.captureConstructor(for: family, prototype: 1))
            let options = OrderedConstructorHost(); var reads: [String] = []
            options.get = { key in reads.append(key.stringReplacingInvalidCodeUnits); return .undefined }
            let construction = try factory.beginConstruct(arguments: [.host(options)])
            XCTAssertTrue(reads.isEmpty)
            let document = try construction.run()
            XCTAssertEqual(reads, keys); XCTAssertTrue(document.isConstructorCreated)
            XCTAssertThrowsError(try construction.run())
            try document.close(); try construction.close()
            let row = try XCTUnwrap(rows.first { $0["family"] as? String == family })
            let initial = try factory.constructPrepared(arguments: [])
            try assertRaw(initial, expected: XCTUnwrap((row["initial"] as? [String: Any])?["render"] as? [String: Any]), family: family)
            try initial.close()
            let populated = try factory.constructPrepared(arguments: [])
            let schema = try populated.schema()
            for action in try XCTUnwrap(row["actions"] as? [[String: Any]]) {
                var graph = try populated.snapshot().graph
                let name = try XCTUnwrap(action["name"] as? String)
                let method = try XCTUnwrap(schema.methods.first { $0.name == name }?.id)
                let args = try XCTUnwrap(action["args"] as? [Any]).map { try argument($0, graph: &graph) }
                try populated.apply(graph, expectedRevision: populated.revision)
                let result = try populated.invoke(method: method, receiver: graph.root, arguments: args)
                if case .thrown(let error) = result.result { XCTFail("\(family).\(name): \(error)") }
            }
            try assertRaw(populated, expected: XCTUnwrap((row["freshPopulated"] as? [String: Any])?["render"] as? [String: Any]), family: family)
            try populated.close()
            let warning = OrderedConstructorHost()
            options.get = { key in
                switch key.stringReplacingInvalidCodeUnits {
                case "wrap": return .bool(true)
                case "sequenceWrap": return .bool(false)
                case "lineOffset": return .number(9)
                case "warn": return .host(warning)
                default: throw PeleError(code: "test", message: "Unexpected options read")
                }
            }
            let configured = try factory.construct(firstArgument: .host(options))
            let configuredGraph = try configured.snapshot().graph
            switch family {
            case "sequence":
                guard case .bool(true) = try configuredGraph.field("wrapEnabled"), case .bool(false) = try configuredGraph.field("sequenceWrap") else { return XCTFail("Sequence option scalars changed") }
            case "agentflow":
                guard case .number(9) = try configuredGraph.field("lineOffset") else { return XCTFail("AgentFlow line offset changed") }
            default:
                let warn = try XCTUnwrap(configuredGraph.field("warn"))
                guard case .host(let retained) = try configured.resolveHostValue(warn) else { return XCTFail("Block warning identity lost") }
                XCTAssertTrue(retained === warning)
            }
            try configured.close()
            let original = OrderedConstructorError()
            options.get = { _ in throw original }
            let failed = try factory.beginConstruct(arguments: [.host(options)])
            factory.close(); try registry.close()
            XCTAssertThrowsError(try failed.run()) { XCTAssertTrue(($0 as AnyObject) === original) }
            let partial = try XCTUnwrap(failed.retainedReceiver)
            XCTAssertTrue(partial.isConstructorCreated)
            _ = try partial.snapshot()
            try partial.close(); try failed.close()
        }
    }
    func testClassMemberUsesCapturedBuiltinMethodsAndMatchesEveryPinnedConstructorCase() throws {
        let fixture = try fixture("public-class-member-constructor.json")
        let registry = try Pele.Registry.allBuiltins()
        let factory = try XCTUnwrap(registry.captureConstructor(for: "class", prototype: 2))
        try registry.close(); defer { factory.close() }
        for row in try XCTUnwrap(fixture["cases"] as? [[String: Any]]) {
            let input = try XCTUnwrap(row["input"] as? String), kind = try XCTUnwrap(row["kind"] as? String)
            let construction = try factory.beginConstruct(arguments: [.string(.init(input)), .string(.init(kind))])
            let document = try construction.run()
            let graph = try document.snapshot().graph
            guard case .ref(let root) = graph.root, case .instance(let prototype, let properties) = graph.nodes[Int(root)] else { return XCTFail("ClassMember prototype missing") }
            XCTAssertEqual(prototype, 2)
            let actual: [[Any]] = try properties.map { [try XCTUnwrap(graph.strings[Int($0.key)].stringIfWellFormed), try text(graph, $0.value), Int($0.flags)] }
            XCTAssertEqual(try JSONSerialization.data(withJSONObject: actual), try JSONSerialization.data(withJSONObject: XCTUnwrap(row["fields"])))
            try document.close(); try construction.close()
        }
    }
    func testClassMemberVirtualHelperCanEditItsProvisionalReceiverAndThrowWithoutRollback() throws {
        _ = try fixture("public-class-member-constructor.json")
        let registry = try Pele.Registry.allBuiltins()
        let factory = try XCTUnwrap(registry.captureConstructor(for: "class", prototype: 2))
        defer { factory.close(); try? registry.close() }
        for shouldThrow in [false, true] {
            let construction = try factory.beginConstruct(arguments: [.string(.init("input")), .string(.init("method"))])
            let replacement = OrderedConstructorHost(), original = OrderedConstructorError()
            replacement.call = { receiver, args in
                guard case .graphReference(let root) = receiver, case .string(let input) = args.first else { throw PeleError(code: "test", message: "Virtual constructor call lost values") }
                XCTAssertEqual(input, .init("input")); XCTAssertThrowsError(try construction.close())
                var graph = try construction.snapshot().graph
                let id = try graph.intern(" + custom "); try graph.setField("id", value: .string(id), on: root)
                try construction.apply(graph, expectedRevision: construction.revision)
                if shouldThrow { throw original }
                return .bool(true)
            }
            var graph = try construction.snapshot().graph
            let callable = try construction.retainHostValue(.host(replacement))
            try graph.setField("parseMethod", value: callable)
            try construction.apply(graph, expectedRevision: construction.revision)
            if shouldThrow {
                XCTAssertThrowsError(try construction.run()) { XCTAssertTrue(($0 as AnyObject) === original) }
            } else { _ = try construction.run() }
            let receiver = try XCTUnwrap(construction.retainedReceiver)
            let result = try receiver.snapshot().graph
            XCTAssertEqual(try text(result, result.field("id")), shouldThrow ? " + custom " : " + custom")
            try receiver.close(); try construction.close()
        }
    }
}
