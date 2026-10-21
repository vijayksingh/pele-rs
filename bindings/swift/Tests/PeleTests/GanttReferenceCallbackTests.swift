import Foundation
import XCTest
import Pele

private final class GanttReferenceError: Error {}
private final class GanttReferenceHost: Pele.PublicHostCallable {
    var get: (Pele.JSString) throws -> Pele.PublicHostValue = { _ in .undefined }
    var call: (Pele.PublicHostValue, [Pele.PublicHostValue]) throws -> Pele.PublicHostValue = { _, _ in .undefined }
    var primitive: (Pele.PublicHostHint) throws -> Pele.PublicHostValue = { _ in .undefined }
    func publicGet(_ key: Pele.JSString) throws -> Pele.PublicHostValue { try get(key) }
    func publicCall(receiver: Pele.PublicHostValue, arguments: [Pele.PublicHostValue]) throws -> Pele.PublicHostValue { try call(receiver, arguments) }
    func publicPrimitive(_ hint: Pele.PublicHostHint) throws -> Pele.PublicHostValue { try primitive(hint) }
}

final class GanttReferenceCallbackTests: XCTestCase {
    private func expected(_ value: Any) throws -> ConfigValue {
        try JSONDecoder().decode(ConfigValue.self, from: JSONSerialization.data(withJSONObject: value, options: [.fragmentsAllowed]))
    }
    private func canonical(_ graph: Pele.PublicGraph, result: Pele.PublicGraph.Atom) -> ConfigValue {
        var ids: [UInt32: UInt32] = [:], nodes: [ConfigValue] = []
        func string(_ id: UInt32) -> ConfigValue {
            let value = graph.strings[Int(id)].stringIfWellFormed
            XCTAssertNotNil(value)
            return .string(value ?? "invalid UTF16")
        }
        func atom(_ v: Pele.PublicGraph.Atom) -> ConfigValue {
            switch v {
            case .undefined: return .array([.string("undefined")])
            case .null: return .array([.string("null")])
            case .bool(let b): return .array([.string("bool"), .bool(b)])
            case .number(let n): return .array([.string("number"), .string(String(format: "%016llx", n.bitPattern))])
            case .string(let id): return .array([.string("string"), string(id)])
            case .ref(let id):
                if let known = ids[id] { return .array([.string("ref"), .number(Double(known))]) }
                let mapped = UInt32(nodes.count); ids[id] = mapped; nodes.append(.null)
                let node = graph.nodes[Int(id)]
                var row: [String: ConfigValue]
                switch node {
                case .instance(let prototype, _): XCTAssertEqual(prototype, 1); row = ["kind": .string("instance")]
                case .object: row = ["kind": .string("object")]
                case .array: row = ["kind": .string("array")]
                case .map(let entries, _): row = ["kind": .string("map"), "entries": .array(entries.map { .array([atom($0.key),atom($0.value)]) })]
                case .set(let values, _): row = ["kind": .string("set"), "values": .array(values.map(atom))]
                default: XCTFail("Unexpected constructor node"); row = [:]
                }
                var properties: [ConfigValue] = node.properties.map { .array([string($0.key),atom($0.value),.number(Double($0.flags))]) }
                if case .array(let length, _) = node,
                   !node.properties.contains(where: { graph.strings[Int($0.key)].stringIfWellFormed == "length" }) {
                    properties.append(.array([.string("length"), atom(.number(Double(length))), .number(2)]))
                }
                row["properties"] = .array(properties)
                nodes[Int(mapped)] = .object(row)
                return .array([.string("ref"), .number(Double(mapped))])
            case .builtinFunction(let id):
                XCTAssertEqual(id, 0); return .array([.string("function"), .number(2)])
            case .hostReference: return .array([.string("host"), .string("options")])
            }
        }
        let root = atom(graph.root)
        let returned = atom(result)
        return .object(["root":root,"nodes":.array(nodes),"result":returned])
    }

    private func invoke(_ document: Pele.MutableDocument, _ method: UInt32, _ args: [Pele.PublicGraph.Atom]) throws -> Pele.PublicGraph.Atom {
        let root = try document.snapshot().graph.root
        let outcome = try document.invoke(method: method, receiver: root, arguments: args)
        switch outcome.result {
        case .returned(let value): return value
        case .thrown(let error): throw PeleError(code: "test", message: "Unexpected Gantt error: \(error)")
        }
    }
    private func strings(_ document: Pele.MutableDocument, _ values: [String]) throws -> [Pele.PublicGraph.Atom] {
        var graph = try document.snapshot().graph
        let atoms = try values.map { Pele.PublicGraph.Atom.string(try graph.intern($0)) }
        try document.apply(graph, expectedRevision: document.revision)
        return atoms
    }
    private func addTask(_ document: Pele.MutableDocument, _ title: String, _ data: String) throws {
        _ = try invoke(document, 15, strings(document, [title, data]))
    }
    private func ref(_ value: Pele.PublicGraph.Atom?) throws -> UInt32 {
        guard case .ref(let id) = value else { throw PeleError(code: "test", message: "Expected task reference") }; return id
    }
    private func firstTask(_ graph: Pele.PublicGraph) throws -> UInt32 {
        try ref(graph.field("0", on: ref(graph.field("tasks"))))
    }
    private func alias(_ graph: Pele.PublicGraph, _ result: Pele.PublicGraph.Atom, first: UInt32) throws -> Bool {
        try ref(graph.field("0", on: ref(result))) == first
    }
    func testReferencesKeepLazyKeywordWarningOrderTaskAliasesReentryAndOriginalErrors() throws {
        var path = URL(fileURLWithPath: #filePath); for _ in 0..<5 { path.deleteLastPathComponent() }
        let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-gantt-reference-warnings.json"))) as? [String: Any])
        XCTAssertEqual(fixture["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        let rows = try XCTUnwrap(fixture["cases"] as? [[String: Any]])
        let selected = ["known-only", "warn-false", "normal", "first-get-throw", "second-get-throw", "keyword-throw", "keyword-reentry", "warn-reentry", "warn-throw"]
        for id in selected {
            let row = try XCTUnwrap(rows.first { $0["id"] as? String == id })
            let first = try XCTUnwrap(row["first"] as? [String: Any]), recovery = try XCTUnwrap(row["recovery"] as? [String: Any])
            let registry = try Pele.Registry.allBuiltins(), options = GanttReferenceHost(), warning = GanttReferenceHost(), keyword = GanttReferenceHost()
            let factory = try XCTUnwrap(registry.captureConstructor(for: "gantt", prototype: 1))
            var trace: [[Any]] = [], reads = 0
            let original = GanttReferenceError()
            options.get = { key in
                XCTAssertEqual(key, .init("warn")); reads += 1; trace.append(["get", reads])
                if (id == "first-get-throw" && reads == 1) || (id == "second-get-throw" && reads == 2) { throw original }
                return id == "warn-false" ? .bool(false) : .host(warning)
            }
            let document = try factory.construct(firstArgument: .host(options))
            XCTAssertEqual(reads, 0, "Retained options must not be read during construction")
            factory.close(); try registry.close(); defer { try? document.close() }
            try addTask(document, "Alpha", ":a,2024-01-01,1d")
            try addTask(document, "Beta", ":b,2024-01-02,1d")
            let initial = try document.snapshot().graph, originalTask = try firstTask(initial)
            guard case .host(let retained) = try document.resolveHostValue(XCTUnwrap(initial.field("options"))) else { return XCTFail("Original options lost") }
            XCTAssertTrue(retained === options)
            keyword.primitive = { hint in
                if case .string = hint {} else { XCTFail("Keyword must request string coercion") }
                trace.append(["keyword"])
                if id == "keyword-throw" { throw original }
                if id == "keyword-reentry" {
                    _ = try self.invoke(document, 14, self.strings(document, ["Nested"]))
                    options.get = { _ in XCTFail("Warning callee must already be captured"); return .undefined }
                }
                return .string(.init(codeUnits: Array("after".utf16) + [0xd800]))
            }
            warning.call = { receiver, arguments in
                guard case .host(let owner) = receiver, case .string(let message) = arguments.first else { throw PeleError(code: "test", message: "Warning receiver/message lost") }
                XCTAssertTrue(owner === options)
                trace.append(["call", owner === options, message.codeUnits.map(Int.init)])
                XCTAssertThrowsError(try document.close())
                if id == "warn-reentry" {
                    try self.addTask(document, "Resolved", ":missing,2024-01-01,1d")
                    var graph = try document.snapshot().graph
                    let changed = try graph.intern("Changed")
                    try graph.setField("task", value: .string(changed), on: originalTask)
                    try document.apply(graph, expectedRevision: document.revision)
                }
                if id == "warn-throw" { throw original }
                return .host(GanttReferenceHost())
            }
            let keywordAtom = try document.retainHostValue(.host(keyword))
            let list = try XCTUnwrap(row["list"] as? String), listAtom = try XCTUnwrap(strings(document, [list]).first)
            var result: Pele.PublicGraph.Atom = .undefined
            if first["error"] as? String == "original" {
                XCTAssertThrowsError(try invoke(document, 27, [keywordAtom, listAtom])) { XCTAssertTrue(($0 as AnyObject) === original) }
            } else {
                XCTAssertTrue(first["error"] is NSNull)
                result = try invoke(document, 27, [keywordAtom, listAtom])
            }
            let after = try document.snapshot()
            XCTAssertEqual(after.revision, document.revision)
            XCTAssertEqual(canonical(after.graph, result: result), try expected(XCTUnwrap(first["graph"])), id + " partial graph")
            XCTAssertTrue(NSArray(array: trace).isEqual(to: try XCTUnwrap(first["trace"] as? [[Any]])), id + " trace")
            if let expectedAlias = first["returnedAlias"] as? Bool { XCTAssertEqual(try alias(after.graph, result, first: originalTask), expectedAlias) }
            options.get = { _ in .undefined }
            let recoveryArgs = try strings(document, ["after", list])
            let recovered = try invoke(document, 27, recoveryArgs)
            let final = try document.snapshot()
            XCTAssertEqual(canonical(final.graph, result: recovered), try expected(XCTUnwrap(recovery["graph"])), id + " recovery graph")
            XCTAssertEqual(try alias(final.graph, recovered, first: originalTask), recovery["returnedAlias"] as? Bool)
            XCTAssertEqual(final.revision, document.revision)
        }
    }
}
