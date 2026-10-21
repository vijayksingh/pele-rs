import Foundation
import XCTest
import Pele

private final class XYChartCallbackFailure: Error {}
@MainActor private enum XYChartCallbackDocument {
    case normal(Pele.MutableDocument), prepared(Pele.PreparedDocument)
    var revision: UInt64 { switch self { case .normal(let d): d.revision; case .prepared(let d): d.revision } }
    func snapshot() throws -> Pele.PublicSnapshot { switch self { case .normal(let d): try d.snapshot(); case .prepared(let d): try d.snapshot() } }
    func apply(_ graph: Pele.PublicGraph) throws {
        switch self { case .normal(let d): _ = try d.apply(graph, expectedRevision: d.revision); case .prepared(let d): _ = try d.apply(graph, expectedRevision: d.revision) }
    }
    func invoke(method: UInt32, receiver: Pele.PublicGraph.Atom, arguments: [Pele.PublicGraph.Atom]) throws {
        switch self { case .normal(let d): _ = try d.invoke(method: method, receiver: receiver, arguments: arguments); case .prepared(let d): _ = try d.invoke(method: method, receiver: receiver, arguments: arguments) }
    }
    func render(_ resources: Pele.HostResources, responsive: Bool = false) throws -> FullRenderedDiagram {
        var options = Pele.Engine.Options(); options.maxWidth = responsive ? 320 : nil; options.config = ["xyChart": .object(["showDataLabel": .bool(true)])]
        switch self { case .normal(let d): return try d.render(options: options, resources: resources); case .prepared(let d): return try d.render(config: options.config, options: options, resources: resources) }
    }
    func close() throws { switch self { case .normal(let d): try d.close(); case .prepared(let d): try d.close() } }
}
@MainActor private final class XYChartCallbackState {
    var fired = false
    var hits = 0
    var trace: [[Any]] = []
    var nested: FullRenderedDiagram?
    var nestedRevision: UInt64?
    let failure = XYChartCallbackFailure()
}

@MainActor final class NativeXYChartCallbackTests: XCTestCase {
    private static func bits(_ n: Double) -> String { String(format: "%016llx", n.bitPattern) }
    private static func ref(_ a: Pele.PublicGraph.Atom?) throws -> UInt32 {
        guard case .ref(let id) = a else { throw XYChartCallbackFailure() }; return id
    }
    private struct Handles {let x:UInt32,categories:UInt32,y:UInt32,plots:UInt32,line:UInt32,bar:UInt32,data:UInt32,labels:UInt32}
    private static func handles(_ g:Pele.PublicGraph)throws->Handles{
      let x=try ref(g.field("xAxis")),categories=try ref(g.field("categories",on:x)),y=try ref(g.field("yAxis")),plots=try ref(g.field("plots")),line=try ref(g.field("0",on:plots)),bar=try ref(g.field("1",on:plots)),data=try ref(g.field("data",on:line)),labels=try ref(g.field("pointLabels",on:line))
      return Handles(x:x,categories:categories,y:y,plots:plots,line:line,bar:bar,data:data,labels:labels)
    }
    private static func includes(_ g: Pele.PublicGraph, _ array: UInt32, _ id: UInt32) throws -> Bool {
        guard case .array(let count, _) = g.nodes[Int(array)] else { throw XYChartCallbackFailure() }
        for i in 0..<count { if case .ref(let found) = try g.field(String(i), on: array), found == id { return true } }; return false
    }
    private static func text(_ g: inout Pele.PublicGraph, _ value: String) throws -> Pele.PublicGraph.Atom { .string(try g.intern(value)) }
    private static func setText(_ g: inout Pele.PublicGraph, _ key: String, _ value: String, on node: UInt32? = nil) throws {
        let a = try text(&g, value); try g.setField(key, value: a, on: node)
    }
    private static func object(_ g: inout Pele.PublicGraph, _ fields: [(String, Pele.PublicGraph.Atom)]) throws -> UInt32 {
        let id = UInt32(g.nodes.count); g.nodes.append(.object(properties: []))
        for (key, value) in fields { try g.setField(key, value: value, on: id) }; return id
    }
    private static func plot(_ g:inout Pele.PublicGraph,_ title:String,_ values:[Double],_ labels:[String])throws->UInt32{
      var points:[Pele.PublicGraph.Atom]=[]
      for (i,v) in values.enumerated(){let key=try text(&g,String(UnicodeScalar(97+i)!)),point=try array(&g,[key,.number(v)]);points.append(.ref(point))}
      let data=try array(&g,points);var names:[Pele.PublicGraph.Atom]=[];for label in labels{names.append(try text(&g,label))}
      let labels=try array(&g,names),title=try text(&g,title),kind=try text(&g,"line")
      return try object(&g,[("type",kind),("title",title),("data",.ref(data)),("pointLabels",.ref(labels))])
    }
    private static func array(_ g: inout Pele.PublicGraph, _ values: [Pele.PublicGraph.Atom]) throws -> UInt32 {
        let id = UInt32(g.nodes.count); g.nodes.append(.array(length: 0, properties: []))
        for (i, a) in values.enumerated() { try g.setArrayElement(UInt32(i), value: a, on: id) }; return id
    }
    // Fixture uses JSON.stringify on acyclic grammar records. Undefined object
    // fields are omitted; array holes/undefined become null, as in the oracle.
    private static func encoded(_ g: Pele.PublicGraph, _ a: Pele.PublicGraph.Atom?) throws -> Any {
        switch a {
        case .string(let id): return try XCTUnwrap(g.strings[Int(id)].stringIfWellFormed)
        case .number(let n): return n.isFinite ? n : NSNull()
        case .bool(let b): return b
        case .ref(let id):
            let node = g.nodes[Int(id)]
            if case .array(let count, _) = node { return try (0..<count).map { try encoded(g, g.field(String($0), on: id)) } }
            if case .map(let entries, _) = node { return try entries.map { [try encoded(g, $0.key), try encoded(g, $0.value)] } }
            var record: [String: Any] = [:]
            for p in node.properties {
                if case .undefined = p.value { continue }
                record[try XCTUnwrap(g.strings[Int(p.key)].stringIfWellFormed)] = try encoded(g, p.value)
            }
            return record
        case nil, .undefined, .null: return NSNull()
        default: throw XYChartCallbackFailure()
        }
    }
    private static func state(_ g: Pele.PublicGraph) throws -> [String: Any] {
        var out: [String: Any] = [:]
        for key in ["title", "accTitle", "accDescr", "orientation", "xAxis", "yAxis", "hasSetYAxis", "plots"] { out[key] = try encoded(g, g.field(key)) }; return out
    }
    private static func resources(_ state: XYChartCallbackState? = nil) -> Pele.HostResources {
        var host = Pele.HostResources()
        host.textWidth = { request in MainActor.assumeIsolated {
            state?.trace.append(["width", request.text, bits(request.size), Int(request.style)])
            return Double(request.text.utf16.count) * request.size * 0.6
        }}
        return host
    }
    private func compare(_ actual: FullRenderedDiagram, _ expected: [String: Any]) throws {
        XCTAssertEqual(actual.svg, try XCTUnwrap(expected["svg"] as? String)); XCTAssertEqual(Self.bits(actual.width), expected["width"] as? String); XCTAssertEqual(Self.bits(actual.height), expected["height"] as? String); XCTAssertTrue(actual.links.isEmpty)
    }
    private static func mutate(_ document: XYChartCallbackDocument, _ state: XYChartCallbackState, _ mode: String) throws {
        state.fired = true; XCTAssertThrowsError(try document.close())
        var g = try document.snapshot().graph; let h = try handles(g)
        switch mode {
        case "baseline": break
        case "early-title": try setText(&g,"title","Early title")
        case "legend-current": try setText(&g,"title","Changed current",on:h.line);try setText(&g,"type","bar",on:h.line)
        case "legend-next": try setText(&g,"title","Changed later",on:h.bar)
        case "plots-replace": let p=try plot(&g,"Replacement",[3,7],["R1","R2"]),a=try array(&g,[.ref(p)]);try g.setField("plots",value:.ref(a))
        case "plots-append": let p=try plot(&g,"Appended",[2,4],["A1","A2"]);try g.setArrayElement(2,value:.ref(p),on:h.plots)
        case "axis-replace":
          let kind=try text(&g,"linear"),title=try text(&g,"Replacement axis"),x=try object(&g,[("type",kind),("title",title),("min",.number(0)),("max",.number(2))])
          let name=try text(&g,"Replacement value"),y=try object(&g,[("type",kind),("title",name),("min",.number(0)),("max",.number(30))]);try g.setField("xAxis",value:.ref(x));try g.setField("yAxis",value:.ref(y))
        case "category-current":try setText(&g,"0","Changed current",on:h.categories);try setText(&g,"1","Changed later",on:h.categories)
        case "categories-append":let v=try text(&g,"Apr");try g.setArrayElement(3,value:v,on:h.categories)
        case "bounds":try g.setField("min",value:.number(-20),on:h.y);try g.setField("max",value:.number(40),on:h.y);try g.setField("hasSetYAxis",value:.bool(true))
        case "point-labels":try setText(&g,"0","Changed point",on:h.labels);try setText(&g,"1","Changed later point",on:h.labels)
        case "data-value":let point=try ref(g.field("1",on:h.data));try g.setField("1",value:.number(18),on:point);let data=try ref(g.field("data",on:h.bar)),p=try ref(g.field("0",on:data));try g.setField("1",value:.number(-8),on:p)
        case "data-replace":let p=try plot(&g,"",[1,3,7],[]),data=try ref(g.field("data",on:p));try g.setField("data",value:.ref(data),on:h.line)
        case "late-point":let point=try ref(g.field("1",on:h.data));try g.setField("1",value:.number(19),on:point);try setText(&g,"1","Paint changed",on:h.labels)
        case "title-accessibility":try setText(&g,"title","Changed title");try setText(&g,"accTitle","Changed access");try setText(&g,"accDescr","Changed description")
        case "throw-object","throw-undefined":try setText(&g,"title","Before failure");let point=try ref(g.field("1",on:h.data));try g.setField("1",value:.number(17),on:point)
        case "nested":try setText(&g,"title","Nested title")
        default:throw XYChartCallbackFailure()
        }
        if mode != "baseline" { try document.apply(g) }
        if mode.hasPrefix("throw-") { throw state.failure }
        if mode == "nested" {
            state.nested = try document.render(resources()); state.nestedRevision = document.revision
            var latest = try document.snapshot().graph; try setText(&latest, "title", "After nested"); try document.apply(latest)
        }
    }
    func testXYChartCapturedAxesSeriesLabelsCallbacksMatch72OracleRows() throws {
        var path = URL(fileURLWithPath: #filePath); for _ in 0..<5 { path.deleteLastPathComponent() }
        let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-xychart-live-render.json"))) as? [String: Any])
        XCTAssertEqual(fixture["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        let source = try XCTUnwrap(fixture["source"] as? String), rows = try XCTUnwrap(fixture["rows"] as? [[String: Any]]); XCTAssertEqual(rows.count, 72)
        for row in rows {
            let raw = try XCTUnwrap(row["raw"] as? Bool), responsive = try XCTUnwrap(row["responsive"] as? Bool), mode = try XCTUnwrap(row["mode"] as? String), engine = Pele.Engine()
            let document: XYChartCallbackDocument = try raw ? .prepared(engine.parsePrepared("xychart", source: source)) : .normal(engine.parseMutable(source)); defer { try? document.close() }
            var initial = try document.snapshot().graph; if responsive { try Self.setText(&initial, "orientation", "horizontal") }
            try document.apply(initial)
            let original = try Self.handles(document.snapshot().graph), observed = XYChartCallbackState(); var host = Self.resources()
            host.textWidth = { request in try MainActor.assumeIsolated {
                observed.trace.append(["width", request.text, Self.bits(request.size), Int(request.style)])
                let target=mode=="early-title" ? "0" : mode=="title-accessibility" ? "Initial title" : ["category-current","categories-append"].contains(mode) ? "Jan" : ["point-labels","data-value","data-replace","late-point"].contains(mode) ? "PointA" : "Forecast"
                if !observed.fired && request.text==target { observed.hits+=1; if mode != "late-point" || observed.hits==2 {try Self.mutate(document,observed,mode)} }
                return Double(request.text.utf16.count) * request.size * 0.6
            }}
            let first = try XCTUnwrap(row["first"] as? [String: Any])
            if mode.hasPrefix("throw-") { XCTAssertEqual(first["original"] as? Bool, true); XCTAssertThrowsError(try document.render(host, responsive: responsive)) { XCTAssertTrue(($0 as AnyObject) === observed.failure) } }
            else { try compare(document.render(host, responsive: responsive), first) }
            XCTAssertTrue(observed.fired); XCTAssertTrue(NSArray(array: observed.trace).isEqual(to: try XCTUnwrap(row["trace"] as? [[Any]])), "\(mode)/\(raw)/\(responsive) trace")
            let after = try document.snapshot(); XCTAssertEqual(after.revision, document.revision)
            XCTAssertTrue(NSDictionary(dictionary: try Self.state(after.graph)).isEqual(to: try XCTUnwrap(row["after"] as? [String: Any])), mode + " state")
            let originalReport = try XCTUnwrap(row["original"] as? [String: Any])
            for(name,id)in[("x",original.x),("categories",original.categories),("y",original.y),("plots",original.plots),("line",original.line),("bar",original.bar),("data",original.data),("labels",original.labels)]{
              let actual=try Self.encoded(after.graph,.ref(id)),expected=try XCTUnwrap(originalReport[name]);XCTAssertTrue(NSArray(array:[actual]).isEqual(to:[expected]),mode+" "+name)
            }
            let x=try Self.ref(after.graph.field("xAxis")),plots=try Self.ref(after.graph.field("plots"))
            let cat=try after.graph.field("categories",on:x);let categoriesSame:Bool;if case .ref(let id)=cat{categoriesSame=id==original.categories}else{categoriesSame=false}
            let identity:[String:Bool]=["x":x==original.x,"categories":categoriesSame,"y":try Self.ref(after.graph.field("yAxis"))==original.y,"plots":plots==original.plots,"line":try Self.includes(after.graph,plots,original.line),"bar":try Self.includes(after.graph,plots,original.bar),"data":try Self.ref(after.graph.field("data",on:original.line))==original.data,"labels":try Self.ref(after.graph.field("pointLabels",on:original.line))==original.labels]
            XCTAssertEqual(identity,try XCTUnwrap(row["identity"]as?[String:Bool]),mode+" identity")
            if mode == "nested" { try compare(XCTUnwrap(observed.nested), XCTUnwrap(row["nested"] as? [String: Any])); XCTAssertGreaterThan(document.revision, try XCTUnwrap(observed.nestedRevision)) }
            let recovery = XYChartCallbackState(); try compare(document.render(Self.resources(recovery), responsive: responsive), XCTUnwrap(row["recovery"] as? [String: Any]))
            XCTAssertTrue(NSArray(array: recovery.trace).isEqual(to: try XCTUnwrap(row["recoveryTrace"] as? [[Any]])), mode + " recovery trace")
        }
    }
}
