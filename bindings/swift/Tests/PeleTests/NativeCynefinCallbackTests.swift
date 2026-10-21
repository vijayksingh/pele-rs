import Foundation
import XCTest
import Pele

private final class CynefinCallbackFailure: Error {}
@MainActor private enum CynefinCallbackDocument {
    case normal(Pele.MutableDocument), prepared(Pele.PreparedDocument)
    var revision: UInt64 { switch self { case .normal(let d): d.revision; case .prepared(let d): d.revision } }
    func snapshot() throws -> Pele.PublicSnapshot { switch self { case .normal(let d): try d.snapshot(); case .prepared(let d): try d.snapshot() } }
    func apply(_ graph: Pele.PublicGraph) throws { switch self { case .normal(let d): _ = try d.apply(graph, expectedRevision: d.revision); case .prepared(let d): _ = try d.apply(graph, expectedRevision: d.revision) } }
    func render(_ resources: Pele.HostResources, responsive: Bool) throws -> FullRenderedDiagram {
        var options = Pele.Engine.Options(); options.maxWidth = responsive ? 300 : nil
        switch self { case .normal(let d): return try d.render(options: options, resources: resources); case .prepared(let d): return try d.render(options: options, resources: resources) }
    }
    func close() throws { switch self { case .normal(let d): try d.close(); case .prepared(let d): try d.close() } }
}
@MainActor private final class CynefinCallbackState {
    var fired = false
    var trace: [[Any]] = []
    var nested: FullRenderedDiagram?
    var nestedRevision: UInt64?
    let failure = CynefinCallbackFailure()
}
@MainActor final class NativeCynefinCallbackTests: XCTestCase {
    private static func bits(_ n: Double) -> String { String(format: "%016llx", n.bitPattern) }
    private static func ref(_ value: Pele.PublicGraph.Atom?) throws -> UInt32 {
        guard case .ref(let id) = value else { throw PeleError(code: "test", message: "Expected Cynefin reference") }; return id
    }
    private static func domain(_ graph: Pele.PublicGraph, _ name: String) throws -> UInt32 {
        let map=try ref(graph.field("domains")); guard case .map(let entries,_) = graph.nodes[Int(map)] else {throw CynefinCallbackFailure()}
        for entry in entries {if case .string(let id)=entry.key,graph.strings[Int(id)].stringIfWellFormed==name{return try ref(entry.value)}}
        throw CynefinCallbackFailure()
    }
    private static func nodes(_ graph: Pele.PublicGraph) throws -> (domains:UInt32,items:UInt32,first:UInt32,transitions:UInt32,transition:UInt32) {
        let domains=try ref(graph.field("domains")),complex=try domain(graph,"complex"),items=try ref(graph.field("items",on:complex)),first=try ref(graph.field("0",on:items)),transitions=try ref(graph.field("transitions")),transition=try ref(graph.field("0",on:transitions));return(domains,items,first,transitions,transition)
    }
    private static func setText(_ graph: inout Pele.PublicGraph, _ key: String, _ value: String, on node: UInt32? = nil) throws {
        let id = try graph.intern(value); try graph.setField(key, value: .string(id), on: node)
    }
    private static func array(_ graph: inout Pele.PublicGraph, _ values: [Pele.PublicGraph.Atom]) throws -> UInt32 {
        let id = UInt32(graph.nodes.count); graph.nodes.append(.array(length: 0, properties: []))
        for (i, value) in values.enumerated() { try graph.setArrayElement(UInt32(i), value: value, on: id) }; return id
    }
    private static func encoded(_ graph: Pele.PublicGraph, _ atom: Pele.PublicGraph.Atom) throws -> Any {
        switch atom {
        case .undefined: return NSNull()
        case .null: return NSNull()
        case .bool(let b): return b
        case .number(let n): return n
        case .string(let id): return try XCTUnwrap(graph.strings[Int(id)].stringIfWellFormed)
        case .ref(let id):
            if case .array(let length, _) = graph.nodes[Int(id)] {
                return try (0..<length).map { try encoded(graph, graph.field(String($0), on: id) ?? .undefined) }
            }
            if case .map(let entries, _) = graph.nodes[Int(id)] { return try entries.map { entry in [try encoded(graph,entry.key),try encoded(graph,entry.value)] } }
            if case .set(let entries, _) = graph.nodes[Int(id)] { return ["set": try entries.map { try encoded(graph,$0) }] }
            var object: [String: Any] = [:]
            for p in graph.nodes[Int(id)].properties { object[try XCTUnwrap(graph.strings[Int(p.key)].stringIfWellFormed)] = try encoded(graph, p.value) }; return object
        default: throw CynefinCallbackFailure()
        }
    }
    private static func state(_ graph:Pele.PublicGraph) throws -> [String:Any] {
        var result:[String:Any]=[:];for key in ["title","accTitle","accDescr","domains","transitions"]{result[key]=try encoded(graph,graph.field(key) ?? .undefined)};return result
    }
    private static func item(_ graph:inout Pele.PublicGraph,_ label:String) throws -> UInt32 {
        let id=UInt32(graph.nodes.count);graph.nodes.append(.object(properties:[]));try setText(&graph,"label",label,on:id);return id
    }
    private static func transition(_ graph:inout Pele.PublicGraph,_ from:String,_ to:String,_ label:String) throws -> UInt32 {
        let id=try item(&graph,label);try setText(&graph,"from",from,on:id);try setText(&graph,"to",to,on:id);return id
    }
    private static func resources(_ state: CynefinCallbackState? = nil) -> Pele.HostResources {
        var host = Pele.HostResources()
        host.textWidth = { request in MainActor.assumeIsolated {
            state?.trace.append([request.text, bits(request.size), Int(request.style)])
            return Double(request.text.utf16.count) * request.size * 0.6
        }}
        host.icon = { name in MainActor.assumeIsolated {
            state?.trace.append(["icon", name]); return Pele.TrustedSVG(trustedMarkup: "<path d=\"M0,0H2\"/>")
        }}
        return host
    }
    private func compare(_ actual: FullRenderedDiagram, _ expected: [String: Any]) throws {
        XCTAssertEqual(actual.svg, try XCTUnwrap(expected["svg"] as? String)); XCTAssertEqual(Self.bits(actual.width), expected["width"] as? String); XCTAssertEqual(Self.bits(actual.height), expected["height"] as? String); XCTAssertTrue(actual.links.isEmpty)
    }
    private static func mutate(_ document: CynefinCallbackDocument, _ state: CynefinCallbackState, _ mode: String, _ responsive: Bool) throws {
        state.fired = true; XCTAssertThrowsError(try document.close())
        var graph = try document.snapshot().graph; let n = try nodes(graph)
        switch mode {
        case "baseline":break
        case "title-accessibility":try setText(&graph,"title","Changed title");try setText(&graph,"accTitle","Changed access");try setText(&graph,"accDescr","Changed description")
        case "domain-replace":
            let item=try item(&graph,"Replacement"),items=try array(&graph,[.ref(item)]),id=UInt32(graph.nodes.count);graph.nodes.append(.object(properties:[]));try setText(&graph,"name","complex",on:id);try graph.setField("items",value:.ref(items),on:id);let name=try graph.intern("complex"),map=UInt32(graph.nodes.count);graph.nodes.append(.map(entries:[.init(key:.string(name),value:.ref(id))],properties:[]));try graph.setField("domains",value:.ref(map))
        case "next-domain":let domain=try domain(graph,"complicated"),items=try ref(graph.field("items",on:domain)),item=try ref(graph.field("0",on:items));try setText(&graph,"label","Changed Gamma",on:item)
        case "items-replace":let item=try item(&graph,"Replacement items"),items=try array(&graph,[.ref(item)]),domain=try domain(graph,"complex");try graph.setField("items",value:.ref(items),on:domain)
        case "captured-label":try setText(&graph,"label","After captured Alpha",on:n.first);let second=try ref(graph.field("1",on:n.items));try setText(&graph,"label","Changed Beta",on:second)
        case "append":let item=try item(&graph,"Appended");try graph.setArrayElement(2,value:.ref(item),on:n.items)
        case "confusion-shrink":let domain=try domain(graph,"confusion"),items=try ref(graph.field("items",on:domain));guard case .array(_,let old)=graph.nodes[Int(items)] else{throw CynefinCallbackFailure()};let properties=old.filter{graph.strings[Int($0.key)].stringIfWellFormed != "3"};graph.nodes[Int(items)] = .array(length:3,properties:properties)
        case "transitions-replace":let row=try transition(&graph,"complicated","chaotic","Replacement transition"),list=try array(&graph,[.ref(row)]);try graph.setField("transitions",value:.ref(list))
        case "transition-append":let row=try transition(&graph,"complex","chaotic","Appended transition");try graph.setArrayElement(2,value:.ref(row),on:n.transitions)
        case "transition-fields":try setText(&graph,"from","chaotic",on:n.transition);try setText(&graph,"to","complicated",on:n.transition);try setText(&graph,"label","Changed after capture",on:n.transition)
        case "throw":try setText(&graph,"title","Before throw");try setText(&graph,"label","Saved Alpha",on:n.first)
        case "nested":try setText(&graph,"title","Nested title")
        default:throw CynefinCallbackFailure()
        }
        if mode != "baseline" {try document.apply(graph)}
        // Swift has Error values, not JS undefined throws; both rows retain
        // the actual original Swift Error identity through C15 cleanup.
        if mode == "throw" { throw state.failure }
        if mode == "nested" {
            state.nested = try document.render(resources(), responsive: false); state.nestedRevision = document.revision
            var latest = try document.snapshot().graph; try setText(&latest, "title", "After nested"); try document.apply(latest)
        }
    }
    func testCynefinCapturedDomainsTransitionsErrorsAndRecoveryMatch52OracleRows() throws {
        var path = URL(fileURLWithPath: #filePath); for _ in 0..<5 { path.deleteLastPathComponent() }
        let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-cynefin-live-render.json"))) as? [String: Any])
        XCTAssertEqual(fixture["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        let rows = try XCTUnwrap(fixture["rows"] as? [[String: Any]]), source = try XCTUnwrap(fixture["source"] as? String); XCTAssertEqual(rows.count, 52)
        for row in rows {
            let raw = try XCTUnwrap(row["raw"] as? Bool), responsive = try XCTUnwrap(row["responsive"] as? Bool), mode = try XCTUnwrap(row["mode"] as? String), engine = Pele.Engine()
            let document: CynefinCallbackDocument = try raw ? .prepared(engine.parsePrepared("cynefin", source: source)) : .normal(engine.parseMutable(source)); defer { try? document.close() }
            var initial = try document.snapshot().graph;try Self.setText(&initial,"title","Initial");try document.apply(initial)
            let original = try Self.nodes(document.snapshot().graph), observed = CynefinCallbackState(); var host = Self.resources()
            host.textWidth = { request in try MainActor.assumeIsolated {
                observed.trace.append([request.text, Self.bits(request.size), Int(request.style)])
                let target = mode == "transition-fields" ? "Move one" : ["items-replace","captured-label","append"].contains(mode) ? "Alpha" : mode == "confusion-shrink" ? "One" : "Complex";let hit=request.text==target
                if !observed.fired && hit { try Self.mutate(document, observed, mode, responsive) }
                return Double(request.text.utf16.count) * request.size * 0.6
            }}
            host.icon = { name in try MainActor.assumeIsolated {
                observed.trace.append(["icon", name]); if !observed.fired && mode.hasPrefix("icon-") { try Self.mutate(document, observed, mode, responsive) }
                return Pele.TrustedSVG(trustedMarkup: "<path d=\"M0,0H2\"/>")
            }}
            let first = try XCTUnwrap(row["first"] as? [String: Any])
            if mode == "throw" { XCTAssertEqual(first["original"] as? Bool, true); XCTAssertThrowsError(try document.render(host, responsive: responsive)) { XCTAssertTrue(($0 as AnyObject) === observed.failure) } }
            else { try compare(document.render(host, responsive: responsive), first) }
            XCTAssertTrue(observed.fired); XCTAssertTrue(NSArray(array: observed.trace).isEqual(to: try XCTUnwrap(row["trace"] as? [[Any]])), "\(mode)/\(raw)/\(responsive) trace")
            let after = try document.snapshot(); XCTAssertEqual(after.revision, document.revision)
            XCTAssertTrue(NSDictionary(dictionary:try Self.state(after.graph)).isEqual(to:try XCTUnwrap(row["after"] as? [String:Any])),mode+" state")
            for (name,id) in [("originalItems",original.items),("originalFirst",original.first),("originalTransitions",original.transitions),("originalTransition",original.transition)] {
                let actual=try Self.encoded(after.graph,.ref(id)),expected=try XCTUnwrap(row[name]);XCTAssertTrue(NSArray(object:actual).isEqual(to:[expected]),mode+name)
            }
            XCTAssertEqual(try Self.ref(after.graph.field("domains"))==original.domains,row["domainsCurrent"] as? Bool)
            let complex=try Self.domain(after.graph,"complex");XCTAssertEqual(try Self.ref(after.graph.field("items",on:complex))==original.items,row["itemsCurrent"] as? Bool)
            XCTAssertEqual(try Self.ref(after.graph.field("transitions"))==original.transitions,row["transitionsCurrent"] as? Bool)
            if mode == "nested" { try compare(XCTUnwrap(observed.nested), XCTUnwrap(row["nested"] as? [String: Any])); XCTAssertGreaterThan(document.revision, try XCTUnwrap(observed.nestedRevision)) }
            let recovery = CynefinCallbackState(); try compare(document.render(Self.resources(recovery), responsive: responsive), XCTUnwrap(row["recovery"] as? [String: Any]))
            XCTAssertTrue(NSArray(array: recovery.trace).isEqual(to: try XCTUnwrap(row["recoveryTrace"] as? [[Any]])), mode + " recovery trace")
        }
    }
}
