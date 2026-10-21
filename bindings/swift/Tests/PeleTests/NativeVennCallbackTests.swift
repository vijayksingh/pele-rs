import Foundation
import XCTest
import Pele

private final class VennCallbackFailure: Error {}
@MainActor private enum VennCallbackDocument {
    case normal(Pele.MutableDocument), prepared(Pele.PreparedDocument)
    var revision: UInt64 { switch self { case .normal(let d): d.revision; case .prepared(let d): d.revision } }
    func snapshot() throws -> Pele.PublicSnapshot { switch self { case .normal(let d): try d.snapshot(); case .prepared(let d): try d.snapshot() } }
    func apply(_ graph: Pele.PublicGraph) throws {
        switch self { case .normal(let d): _ = try d.apply(graph, expectedRevision: d.revision); case .prepared(let d): _ = try d.apply(graph, expectedRevision: d.revision) }
    }
    func render(_ resources: Pele.HostResources, responsive: Bool = false) throws -> FullRenderedDiagram {
        var options = Pele.Engine.Options(); options.maxWidth = responsive ? 200 : nil
        switch self { case .normal(let d): return try d.render(options: options, resources: resources); case .prepared(let d): return try d.render(options: options, resources: resources) }
    }
    func close() throws { switch self { case .normal(let d): try d.close(); case .prepared(let d): try d.close() } }
}
@MainActor private final class VennCallbackState {
    var fired = false
    var trace: [[Any]] = []
    var nested: FullRenderedDiagram?
    var nestedRevision: UInt64?
    let failure = VennCallbackFailure()
}

@MainActor final class NativeVennCallbackTests: XCTestCase {
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
        guard case .ref(let id) = atom else { throw PeleError(code: "test", message: "Venn reference missing") }; return id
    }
    private static func setText(_ graph: inout Pele.PublicGraph, _ key: String, _ value: String, on node: UInt32? = nil) throws {
        let id = try graph.intern(value); try graph.setField(key, value: .string(id), on: node)
    }
    private static func append(_ graph: inout Pele.PublicGraph, on node:UInt32, value:Pele.PublicGraph.Atom) throws {
        guard case .array(let length,_) = graph.nodes[Int(node)] else {throw PeleError(code:"test",message:"Venn array missing")}
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
        } else { throw PeleError(code: "test", message: "Unexpected Venn fixture input") }
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
            if case .map(let entries, _) = node { return try entries.map { [try value(graph,$0.key),try value(graph,$0.value)] } }
            var object: [String: Any] = [:]
            for property in node.properties { object[try XCTUnwrap(graph.strings[Int(property.key)].stringIfWellFormed)] = try value(graph, property.value) }
            return object
        default: throw PeleError(code: "test", message: "Unexpected Venn fixture atom")
        }
    }
    private static func state(_ graph: Pele.PublicGraph) throws -> [String: Any] {
        var result: [String: Any] = [:]
        for key in ["title", "subsets", "textNodes", "styles"] { result[key] = try value(graph, graph.field(key) ?? .undefined) }
        return result
    }
    func testVennCapturedArraysLateFieldsNestedCallbacksAndRecoveryMatchAllOracleRows() throws {
        var path = URL(fileURLWithPath: #filePath); for _ in 0..<5 { path.deleteLastPathComponent() }
        let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-venn-live-render.json"))) as? [String: Any])
        XCTAssertEqual(fixture["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        let source = try XCTUnwrap(fixture["source"] as? String), rows = try XCTUnwrap(fixture["rows"] as? [[String: Any]])
        XCTAssertEqual(rows.count, 22)
        for row in rows {
            let raw = try XCTUnwrap(row["raw"] as? Bool), responsive = false, mode = try XCTUnwrap(row["mode"] as? String)
            let engine = Pele.Engine()
            let document: VennCallbackDocument = try raw ? .prepared(engine.parsePrepared("venn", source: source)) : .normal(engine.parseMutable(source))
            defer { try? document.close() }
            let initial = try document.snapshot().graph
            let subsets = try Self.ref(initial.field("subsets")), texts = try Self.ref(initial.field("textNodes")), observed = VennCallbackState()
            var resources = pure()
            resources.textWidth = { request in try MainActor.assumeIsolated {
                observed.trace.append([request.text, String(format: "%016llx", request.size.bitPattern), Int(request.style)])
                if !observed.fired && (mode != "texts-append" || request.text == "Note") {
                    observed.fired = true
                    XCTAssertThrowsError(try document.close())
                    var graph = try document.snapshot().graph
                    let currentSubsets = try Self.ref(graph.field("subsets")), currentTexts = try Self.ref(graph.field("textNodes"))
                    switch mode {
                    case "subset-edit":
                        let second = try Self.ref(graph.field("1", on: currentSubsets)); try Self.setText(&graph,"label","Changed Beta",on:second); try graph.setField("size",value:.number(20),on:second)
                    case "subsets-replace":
                        let value = try Self.input(&graph,[["sets":["A"],"size":20,"label":"New Alpha"],["sets":["B"],"size":10,"label":"New Beta"],["sets":["A","B"],"size":3,"label":"New Both"]]); try graph.setField("subsets",value:value)
                    case "subset-append":
                        let value = try Self.input(&graph,["sets":["A"],"size":10,"label":"Last Alpha"]); try Self.append(&graph,on:currentSubsets,value:value)
                    case "texts-edit":
                        let first = try Self.ref(graph.field("0",on:currentTexts)); try Self.setText(&graph,"label","Changed Note",on:first); try Self.setText(&graph,"id","changed-note",on:first)
                    case "texts-replace":
                        let value = try Self.input(&graph,[["sets":["B"],"id":"other","label":"Other"]]); try graph.setField("textNodes",value:value)
                    case "texts-append":
                        let value = try Self.input(&graph,["sets":["B"],"id":"appended","label":"Appended"]); try Self.append(&graph,on:currentTexts,value:value)
                    case "styles":
                        let styles = try Self.ref(graph.field("styles")), first = try Self.ref(graph.field("0",on:styles)), map = try Self.ref(graph.field("styles",on:first))
                        guard case .map(var entries,let properties) = graph.nodes[Int(map)] else { throw PeleError(code:"test",message:"Venn style map missing") }
                        entries[0].value = .string(try graph.intern("blue")); graph.nodes[Int(map)] = .map(entries:entries,properties:properties)
                        let key = try graph.intern("font-weight"), value = try graph.intern("bold"), newMap = UInt32(graph.nodes.count)
                        graph.nodes.append(.map(entries:[.init(key:.string(key),value:.string(value))],properties:[]))
                        let style = try Self.input(&graph,["targets":["B"]]); try graph.setField("styles",value:.ref(newMap),on:Self.ref(style)); try Self.append(&graph,on:styles,value:style)
                    case "title": try Self.setText(&graph,"title","Changed title")
                    case "throw":
                        let second = try Self.ref(graph.field("1",on:currentSubsets)); try Self.setText(&graph,"title","Before throw"); try Self.setText(&graph,"label","Saved Beta",on:second)
                    case "nested": try Self.setText(&graph,"title","Nested title")
                    case "baseline": break
                    default: throw PeleError(code:"test",message:"Unknown Venn fixture mode")
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
            XCTAssertEqual(try Self.ref(after.graph.field("subsets")) == subsets, row["subsetsCurrent"] as? Bool); XCTAssertEqual(try Self.ref(after.graph.field("textNodes")) == texts, row["textsCurrent"] as? Bool)
            XCTAssertTrue(NSArray(array: try XCTUnwrap(Self.value(after.graph, .ref(subsets)) as? [Any])).isEqual(to: try XCTUnwrap(row["originalSubsets"] as? [Any])), mode + " original subsets")
            XCTAssertTrue(NSArray(array: try XCTUnwrap(Self.value(after.graph, .ref(texts)) as? [Any])).isEqual(to: try XCTUnwrap(row["originalTexts"] as? [Any])), mode + " original texts")
            if mode == "nested" { try compare(XCTUnwrap(observed.nested), XCTUnwrap(row["nested"] as? [String: Any])); XCTAssertGreaterThan(document.revision, try XCTUnwrap(observed.nestedRevision)) }
            let recovery = VennCallbackState(); var plain = pure()
            plain.textWidth = { request in MainActor.assumeIsolated { recovery.trace.append([request.text, String(format: "%016llx", request.size.bitPattern), Int(request.style)]); return Double(request.text.utf16.count) * request.size * 0.6 } }
            try compare(document.render(plain, responsive: responsive), XCTUnwrap(row["recovery"] as? [String: Any]))
            XCTAssertTrue(NSArray(array: recovery.trace).isEqual(to: try XCTUnwrap(row["recoveryTrace"] as? [[Any]])), mode + " recovery trace")
        }
    }
}
