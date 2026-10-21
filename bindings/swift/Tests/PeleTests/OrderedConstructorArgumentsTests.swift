import Foundation
import XCTest
import Pele

private final class OrderedConstructorError: Error {}
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
    func testOrderedOptionsConstructorsReadOnlyRequestedFieldsAndPreserveOriginalErrors() throws {
        _ = try fixture("public-ordered-constructors.json")
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
