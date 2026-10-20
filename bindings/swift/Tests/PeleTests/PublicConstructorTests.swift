import Foundation
import XCTest
import Pele

private final class ConstructorOverlayModel {}

private enum ConstructorDocument {
    case mutable(Pele.MutableDocument), prepared(Pele.PreparedDocument)
    var revision: UInt64 { switch self { case .mutable(let d): d.revision; case .prepared(let d): d.revision } }
    var constructorCreated: Bool { switch self { case .mutable(let d): d.isConstructorCreated; case .prepared(let d): d.isConstructorCreated } }
    func snapshot() throws -> Pele.PublicSnapshot { switch self { case .mutable(let d): try d.snapshot(); case .prepared(let d): try d.snapshot() } }
    func schema() throws -> Pele.PublicModelSchema { switch self { case .mutable(let d): try d.schema(); case .prepared(let d): try d.schema() } }
    func apply(_ graph: Pele.PublicGraph) throws { switch self { case .mutable(let d): _ = try d.apply(graph, expectedRevision: d.revision); case .prepared(let d): _ = try d.apply(graph, expectedRevision: d.revision) } }
    func invoke(_ method: UInt32, _ root: Pele.PublicGraph.Atom, _ args: [Pele.PublicGraph.Atom]) throws -> Pele.PublicInvocationOutcome {
        switch self { case .mutable(let d): try d.invoke(method: method, receiver: root, arguments: args); case .prepared(let d): try d.invoke(method: method, receiver: root, arguments: args) }
    }
    func render(_ resources: Pele.HostResources) throws -> FullRenderedDiagram { switch self { case .mutable(let d): try d.render(resources: resources); case .prepared(let d): try d.render(resources: resources) } }
    func close() throws { switch self { case .mutable(let d): try d.close(); case .prepared(let d): try d.close() } }
}

final class PublicConstructorTests: XCTestCase {
    private func fixture() throws -> [String: Any] {
        var path = URL(fileURLWithPath: #filePath)
        for _ in 0..<5 { path.deleteLastPathComponent() }
        let result = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-sankey-constructor.json"))) as? [String: Any])
        XCTAssertEqual(result["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        return result
    }
    private func value(_ value: Any) throws -> ConfigValue {
        try JSONDecoder().decode(ConfigValue.self, from: JSONSerialization.data(withJSONObject: value, options: [.fragmentsAllowed]))
    }
    private func canonical(_ graph: Pele.PublicGraph) -> ConfigValue {
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
            case .builtinFunction, .hostReference: XCTFail("Unexpected callable in constructor graph"); return .null
            }
        }
        let root = atom(graph.root)
        return .object(["root":root,"nodes":.array(nodes)])
    }

    private func compare(_ drawing: FullRenderedDiagram, _ expected: Any) throws {
        let expected = try XCTUnwrap(expected as? [String: Any])
        XCTAssertEqual(drawing.svg, expected["svg"] as? String)
        XCTAssertEqual(String(format: "%016llx", drawing.width.bitPattern), expected["width"] as? String)
        XCTAssertEqual(String(format: "%016llx", drawing.height.bitPattern), expected["height"] as? String)
        XCTAssertTrue(drawing.links.isEmpty)
    }
    func testCapturedConstructorBuildsIndependentModelsAndFirstHelpersWithoutParsing() throws {
        let expected = try fixture()
        let initial = try XCTUnwrap(expected["initial"] as? [String: Any])
        let populated = try XCTUnwrap(expected["populated"] as? [String: Any])
        var resources = Pele.HostResources()
        resources.textWidth = { Double($0.text.utf16.count) * $0.size * 0.6 }
        for prepared in [false, true] {
            let registry = try Pele.Registry.allBuiltins()
            let factory = try XCTUnwrap(registry.captureConstructor(for: "sankey", prototype: 1))
            XCTAssertEqual(factory.type, "sankey"); XCTAssertEqual(factory.prototype, 1)
            // Replacement affects future selection, not this captured registration.
            let overlay = Pele.Diagram(type: "sankey", parse: { _ in ConstructorOverlayModel() }, render: { _, _ in
                Pele.ProviderRendered(svg: "<svg/>", width: 1, height: 1)
            })
            try registry.register(overlay)
            XCTAssertNil(try registry.captureConstructor(for: "sankey", prototype: 1))
            try registry.close()
            let document: ConstructorDocument = try prepared ? .prepared(factory.constructPrepared()) : .mutable(factory.construct())
            let sibling = try factory.construct()
            defer { try? document.close(); try? sibling.close() }
            factory.close()
            XCTAssertThrowsError(try factory.construct()) { error in
                XCTAssertEqual((error as? PeleError)?.message, "Public constructor is closed")
            }
            XCTAssertTrue(document.constructorCreated); XCTAssertTrue(sibling.isConstructorCreated)
            XCTAssertEqual(document.revision, 0)
            XCTAssertEqual(canonical(try document.snapshot().graph), try value(initial["graph"]!))
            try compare(document.render(resources), initial["render"]!)
            var graph = try document.snapshot().graph
            let a = try graph.intern("A"), b = try graph.intern("B")
            try document.apply(graph)
            let schema = try document.schema()
            let find = try XCTUnwrap(schema.methods.first { $0.name == "findOrCreateNode" }?.id)
            let link = try XCTUnwrap(schema.methods.first { $0.name == "addLink" }?.id)
            func node(_ id: UInt32) throws -> Pele.PublicGraph.Atom {
                let result = try document.invoke(find, graph.root, [.string(id)])
                guard case .returned(let value) = result.result else { XCTFail("First helper failed"); return .undefined }
                return value
            }
            let first = try node(a), repeated = try node(a), second = try node(b)
            guard case .ref(let firstID) = first, case .ref(let repeatedID) = repeated else {
                return XCTFail("findOrCreateNode must return a retained node reference")
            }
            XCTAssertEqual(firstID, repeatedID)
            let linked = try document.invoke(link, graph.root, [first, second, .number(2)])
            guard case .returned(.undefined) = linked.result else { return XCTFail("addLink failed") }
            XCTAssertEqual(canonical(try document.snapshot().graph), try value(populated["graph"]!))
            try compare(document.render(resources), populated["render"]!)
            XCTAssertEqual(canonical(try sibling.snapshot().graph), try value(initial["graph"]!))
            // Native low-level normal rendering is an explicit API extension;
            // fixture also records Pele core's rejection of unparsed models.
            XCTAssertEqual((expected["coreError"] as? [String: Any])?["message"] as? String,
                           "render() takes diagram text or a model that parse() returned.")
        }
    }
    func testConstructorSelectionIsExplicitAndDoesNotInventCapabilities() throws {
        let registry = try Pele.Registry.allBuiltins()
        defer { try? registry.close() }
        XCTAssertNil(try registry.captureConstructor(for: "sankey", prototype: 999))
        XCTAssertNil(try registry.captureConstructor(for: "pie", prototype: 1))
        XCTAssertThrowsError(try registry.captureConstructor(for: "unknown-family", prototype: 1))
        let parsed = try registry.parseMutable("sankey\nA,B,1")
        defer { try? parsed.close() }
        XCTAssertFalse(parsed.isConstructorCreated)
    }
}
