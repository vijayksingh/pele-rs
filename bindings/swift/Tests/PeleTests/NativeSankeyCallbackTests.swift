import Foundation
import XCTest
import CryptoKit
import Pele

private final class NativeCallbackFailure: Error {}

@MainActor private enum CallbackDocument {
    case mutable(Pele.MutableDocument)
    case prepared(Pele.PreparedDocument)

    var revision: UInt64 {
        switch self { case .mutable(let d): d.revision; case .prepared(let d): d.revision }
    }
    func snapshot() throws -> Pele.PublicSnapshot {
        switch self { case .mutable(let d): try d.snapshot(); case .prepared(let d): try d.snapshot() }
    }
    func schema() throws -> Pele.PublicModelSchema {
        switch self { case .mutable(let d): try d.schema(); case .prepared(let d): try d.schema() }
    }
    func apply(_ graph: Pele.PublicGraph) throws {
        switch self {
        case .mutable(let d): _ = try d.apply(graph, expectedRevision: d.revision)
        case .prepared(let d): _ = try d.apply(graph, expectedRevision: d.revision)
        }
    }
    func invoke(_ method: UInt32, root: Pele.PublicGraph.Atom, argument: Pele.PublicGraph.Atom) throws -> Pele.PublicInvocationOutcome {
        switch self {
        case .mutable(let d): try d.invoke(method: method, receiver: root, arguments: [argument])
        case .prepared(let d): try d.invoke(method: method, receiver: root, arguments: [argument])
        }
    }
    func render(_ resources: Pele.HostResources) throws -> FullRenderedDiagram {
        switch self { case .mutable(let d): try d.render(resources: resources); case .prepared(let d): try d.render(resources: resources) }
    }
    func close() throws {
        switch self { case .mutable(let d): try d.close(); case .prepared(let d): try d.close() }
    }
}

@MainActor private final class NativeCallbackState {
    let document: CallbackDocument
    let method: UInt32
    let failure = NativeCallbackFailure()
    var entered = false
    var closeBlocked = false
    var nested: FullRenderedDiagram?
    var nestedRevision: UInt64?
    var helperError: [String: String]?
    var innerOriginal = false
    var measurements: [[String: Any]] = []

    init(_ document: CallbackDocument) throws {
        self.document = document
        guard let method = try document.schema().methods.first(where: { $0.name == "findOrCreateNode" })?.id else {
            throw PeleError(code: "test", message: "Sankey method missing")
        }
        self.method = method
    }

    func appendNode() throws {
        var graph = try document.snapshot().graph
        let label = try graph.intern("D")
        try document.apply(graph)
        let outcome = try document.invoke(method, root: graph.root, argument: .string(label))
        guard case .returned(.ref) = outcome.result else {
            throw PeleError(code: "test", message: "Sankey helper failed during callback")
        }
    }
}

@MainActor final class NativeSankeyCallbackTests: XCTestCase {
    private let source = "sankey\nA,B,5\nB,C,3"

    private func pureResources() -> Pele.HostResources {
        var resources = Pele.HostResources()
        resources.textWidth = { Double($0.text.utf16.count) * $0.size * 0.6 }
        return resources
    }

    private func fixture(_ mode: String) throws -> [String: Any] {
        try fixture(mode, file: "native-sankey-callbacks.json")
    }

    private func fixture(_ mode: String, file: String) throws -> [String: Any] {
        var path = URL(fileURLWithPath: #filePath)
        for _ in 0..<5 { path.deleteLastPathComponent() }
        let data = try Data(contentsOf: path.appendingPathComponent("fixtures/" + file))
        let record = try XCTUnwrap(JSONSerialization.jsonObject(with: data) as? [String: Any])
        XCTAssertEqual(record["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        return try XCTUnwrap((record["rows"] as? [[String: Any]])?.first { $0["mode"] as? String == mode })
    }

    private func compare(_ drawing: FullRenderedDiagram, _ expected: [String: Any]) throws {
        XCTAssertEqual(SHA256.hash(data: Data(drawing.svg.utf8)).map { String(format: "%02x", $0) }.joined(), expected["svgSHA256"] as? String)
        XCTAssertEqual(drawing.width, try XCTUnwrap(expected["width"] as? Double))
        XCTAssertEqual(drawing.height, try XCTUnwrap(expected["height"] as? Double))
        XCTAssertTrue(drawing.links.isEmpty)
    }

    private func run(_ mode: String) throws {
        let expected = try fixture(mode)
        for prepared in [false, true] {
            let engine = Pele.Engine()
            let document: CallbackDocument = try prepared
                ? .prepared(engine.parsePrepared("sankey", source: source))
                : .mutable(engine.parseMutable(source))
            defer { try? document.close() }
            let state = try NativeCallbackState(document)
            var resources = pureResources()
            resources.textWidth = { request in try MainActor.assumeIsolated {
                state.measurements.append(["text": request.text, "size": request.size, "style": Int(request.style)])
                if !state.entered {
                    state.entered = true
                    do { try state.document.close() }
                    catch { state.closeBlocked = (error as? PeleError)?.message == "Mutable document is already in use" }
                    if mode == "throw" {
                        var graph = try state.document.snapshot().graph
                        guard case .ref(let nodes) = try graph.field("nodes"), case .ref(let node) = try graph.field("1", on: nodes) else {
                            throw PeleError(code: "test", message: "Sankey node missing")
                        }
                        let label = try graph.intern("Changed B"), title = try graph.intern("Changed title")
                        try graph.setField("id", value: .string(label), on: node)
                        try graph.setField("title", value: .string(title))
                        try state.document.apply(graph)
                        throw state.failure
                    }
                    try state.appendNode()
                    if mode == "nested" {
                        var nestedResources = Pele.HostResources()
                        nestedResources.textWidth = { Double($0.text.utf16.count) * $0.size * 0.6 }
                        state.nested = try state.document.render(nestedResources)
                        state.nestedRevision = state.document.revision
                    }
                }
                return Double(request.text.utf16.count) * request.size * 0.6
            }}
            if mode == "throw" {
                XCTAssertThrowsError(try document.render(resources)) { XCTAssertTrue(($0 as AnyObject) === state.failure) }
            } else {
                try compare(document.render(resources), XCTUnwrap(expected["first"] as? [String: Any]))
            }
            XCTAssertTrue(state.entered)
            XCTAssertTrue(state.closeBlocked)
            XCTAssertEqual(try JSONSerialization.data(withJSONObject: state.measurements, options: .sortedKeys),
                           try JSONSerialization.data(withJSONObject: XCTUnwrap(expected["measurements"]), options: .sortedKeys))
            let snapshot = try document.snapshot()
            XCTAssertEqual(snapshot.revision, document.revision)
            guard case .ref(let nodes) = try snapshot.graph.field("nodes"), case .array(let length, let properties) = snapshot.graph.nodes[Int(nodes)] else {
                return XCTFail("Sankey nodes missing after callback")
            }
            let nodeNames = try (0..<length).map { index -> String in
                guard case .ref(let node) = try snapshot.graph.field(String(index), on: nodes),
                      case .string(let label) = try snapshot.graph.field("id", on: node),
                      let text = snapshot.graph.strings[Int(label)].stringIfWellFormed else {
                    throw PeleError(code: "test", message: "Sankey node identity lost")
                }
                return text
            }
            _ = properties
            XCTAssertEqual(nodeNames, (expected["nodes"] as? [[String: Any]])?.compactMap { $0["id"] as? String })
            if mode == "nested" {
                try compare(XCTUnwrap(state.nested), XCTUnwrap(expected["nested"] as? [String: Any]))
                XCTAssertGreaterThan(document.revision, try XCTUnwrap(state.nestedRevision))
            }
            try compare(document.render(pureResources()), XCTUnwrap(expected["recovery"] as? [String: Any]))
        }
    }

    func testHelperMutationDuringNormalAndPreparedRender() throws { try run("helper") }
    func testCallbackOriginalErrorKeepsEditsAndRecovers() throws { try run("throw") }
    func testNestedRenderKeepsCurrentRevisionAndRejectsClose() throws { try run("nested") }

    func testCaughtHelperFailureRetainsOrphanAndRepairsNormalAndPreparedModels() throws {
        let expected = try fixture("partial-helper-readonly-length", file: "native-sankey-reentrant-errors.json")
        for prepared in [false, true] {
            let engine = Pele.Engine()
            let document: CallbackDocument = try prepared
                ? .prepared(engine.parsePrepared("sankey", source: source))
                : .mutable(engine.parseMutable(source))
            defer { try? document.close() }
            let state = try NativeCallbackState(document)
            var resources = pureResources()
            resources.textWidth = { request in try MainActor.assumeIsolated {
                if !state.entered {
                    state.entered = true
                    var graph = try document.snapshot().graph
                    guard case .ref(let nodes) = try graph.field("nodes"),
                          case .array(let length, let properties) = graph.nodes[Int(nodes)] else { throw state.failure }
                    let lengthKey = try graph.intern("length"), label = try graph.intern("D")
                    let kept = properties.filter { graph.strings[Int($0.key)].stringIfWellFormed != "length" }
                    graph.nodes[Int(nodes)] = .array(length: length, properties: kept + [.init(key: lengthKey, value: .number(Double(length)), flags: 0)])
                    try document.apply(graph)
                    let outcome = try document.invoke(state.method, root: graph.root, argument: .string(label))
                    guard case .thrown(.javaScript(let name, let message)) = outcome.result,
                          let text = message.stringIfWellFormed else { throw state.failure }
                    state.helperError = ["name": name, "message": text]
                }
                return Double(request.text.utf16.count) * request.size * 0.6
            }}
            try compare(document.render(resources), XCTUnwrap(expected["first"] as? [String: Any]))
            XCTAssertEqual(state.helperError, expected["helperError"] as? [String: String])
            var graph = try document.snapshot().graph
            guard case .ref(let lookup) = try graph.field("lookup"),
                  case .map(let entries, _) = graph.nodes[Int(lookup)],
                  let orphan = entries.first(where: { entry in
                      guard case .string(let key) = entry.key else { return false }
                      return graph.strings[Int(key)].stringIfWellFormed == "D"
                  })?.value,
                  case .ref(let nodes) = try graph.field("nodes"),
                  case .array(let length, _) = graph.nodes[Int(nodes)] else { return XCTFail("Partial helper state missing") }
            XCTAssertEqual(length, 3)
            var references: [Pele.PublicGraph.Atom] = try (0..<length).map { try XCTUnwrap(graph.field(String($0), on: nodes)) }
            XCTAssertFalse(references.contains { if case .ref(let a) = $0, case .ref(let b) = orphan { return a == b }; return false })
            references.append(orphan)
            let keys = try references.indices.map { try graph.intern(String($0)) }
            let replacement = UInt32(graph.nodes.count)
            graph.nodes.append(.array(length: UInt32(references.count), properties: references.indices.map { .init(key: keys[$0], value: references[$0]) }))
            try graph.setField("nodes", value: .ref(replacement))
            try document.apply(graph)
            try compare(document.render(pureResources()), XCTUnwrap(expected["recovery"] as? [String: Any]))
        }
    }

    func testCaughtInnerResourceErrorDoesNotPoisonOuterRender() throws {
        let expected = try fixture("nested-render-original-throw", file: "native-sankey-reentrant-errors.json")
        for prepared in [false, true] {
            let engine = Pele.Engine()
            let document: CallbackDocument = try prepared
                ? .prepared(engine.parsePrepared("sankey", source: source))
                : .mutable(engine.parseMutable(source))
            defer { try? document.close() }
            let state = try NativeCallbackState(document)
            var resources = pureResources()
            resources.textWidth = { request in MainActor.assumeIsolated {
                if !state.entered {
                    state.entered = true
                    var inner = Pele.HostResources()
                    inner.textWidth = { _ in throw state.failure }
                    do { _ = try document.render(inner) }
                    catch { state.innerOriginal = (error as AnyObject) === state.failure }
                    state.nestedRevision = document.revision
                }
                return Double(request.text.utf16.count) * request.size * 0.6
            }}
            try compare(document.render(resources), XCTUnwrap(expected["first"] as? [String: Any]))
            XCTAssertTrue(state.entered)
            XCTAssertTrue(state.innerOriginal)
            XCTAssertGreaterThan(document.revision, try XCTUnwrap(state.nestedRevision))
            try compare(document.render(pureResources()), XCTUnwrap(expected["recovery"] as? [String: Any]))
        }
    }
}
