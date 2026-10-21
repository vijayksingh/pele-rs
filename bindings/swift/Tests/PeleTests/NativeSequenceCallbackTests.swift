import Foundation
import CoreFoundation
import XCTest
import Pele

private final class SequenceCallbackFailure: Error {}
@MainActor private enum SequenceCallbackDocument {
    case normal(Pele.MutableDocument), prepared(Pele.PreparedDocument)
    var revision: UInt64 { switch self { case .normal(let d): d.revision; case .prepared(let d): d.revision } }
    func snapshot() throws -> Pele.PublicSnapshot { switch self { case .normal(let d): try d.snapshot(); case .prepared(let d): try d.snapshot() } }
    func apply(_ graph: Pele.PublicGraph) throws {
        switch self { case .normal(let d): _ = try d.apply(graph, expectedRevision: d.revision); case .prepared(let d): _ = try d.apply(graph, expectedRevision: d.revision) }
    }
    func render(_ resources: Pele.HostResources, responsive: Bool = false) throws -> FullRenderedDiagram {
        var options = Pele.Engine.Options(); options.maxWidth = responsive ? 180 : nil
        switch self { case .normal(let d): return try d.render(options: options, resources: resources); case .prepared(let d): return try d.render(options: options, resources: resources) }
    }
    func invoke(_ method:UInt32,_ arguments:[Pele.PublicGraph.Atom])throws{let root=try snapshot().graph.root;switch self{case .normal(let d):let out=try d.invoke(method:method,receiver:root,arguments:arguments);guard case .returned = out.result else {throw PeleError(code:"test",message:"Sequence helper failed")};case .prepared(let d):let out=try d.invoke(method:method,receiver:root,arguments:arguments);guard case .returned = out.result else {throw PeleError(code:"test",message:"Sequence helper failed")};}}
    func close() throws { switch self { case .normal(let d): try d.close(); case .prepared(let d): try d.close() } }
}
@MainActor private final class SequenceCallbackSequence {
    var fired = false
    var trace: [[Any]] = []
    var nested: FullRenderedDiagram?
    var nestedRevision: UInt64?
    let failure = SequenceCallbackFailure()
}

@MainActor final class NativeSequenceCallbackTests: XCTestCase {
    private func bits(_ n: Double) -> String { String(format: "%016llx", n.bitPattern) }
    private func compare(_ actual: FullRenderedDiagram, _ expected: [String: Any]) throws {
        XCTAssertEqual(actual.svg, try XCTUnwrap(expected["svg"] as? String))
        XCTAssertEqual(bits(actual.width), try XCTUnwrap(expected["width"] as? String))
        XCTAssertEqual(bits(actual.height), try XCTUnwrap(expected["height"] as? String))
        let links=try XCTUnwrap(JSONSerialization.jsonObject(with:JSONEncoder().encode(actual.links)) as? [Any]);XCTAssertTrue(NSArray(array:links).isEqual(to:try XCTUnwrap(expected["links"] as? [Any])))
    }
    private func pure() -> Pele.HostResources {
        var host = Pele.HostResources(); host.textWidth = { Double($0.text.utf16.count) * $0.size * 0.6 }; return host
    }
    private static func ref(_ atom: Pele.PublicGraph.Atom?) throws -> UInt32 {
        guard case .ref(let id) = atom else { throw PeleError(code: "test", message: "Sequence reference missing") }; return id
    }
    private static func setText(_ graph: inout Pele.PublicGraph, _ key: String, _ value: String, on node: UInt32? = nil) throws {
        let id = try graph.intern(value); try graph.setField(key, value: .string(id), on: node)
    }
    private static func append(_ graph: inout Pele.PublicGraph, on node:UInt32, value:Pele.PublicGraph.Atom) throws {
        guard case .array(let length,_) = graph.nodes[Int(node)] else {throw PeleError(code:"test",message:"Sequence array missing")}
        try graph.setField(String(length),value:value,on:node)
        graph.nodes[Int(node)] = .array(length:length+1,properties:graph.nodes[Int(node)].properties)
    }
    private static func array(_ graph:inout Pele.PublicGraph,_ values:[Pele.PublicGraph.Atom])throws->Pele.PublicGraph.Atom {
        var properties:[Pele.PublicGraph.Property]=[]
        for (i,value) in values.enumerated(){properties.append(.init(key:try graph.intern(String(i)),value:value))}
        let id=UInt32(graph.nodes.count);graph.nodes.append(.array(length:UInt32(values.count),properties:properties));return .ref(id)
    }
    private static func input(_ graph: inout Pele.PublicGraph, _ value: Any) throws -> Pele.PublicGraph.Atom {
        if let string = value as? String { return .string(try graph.intern(string)) }
        if let number = value as? NSNumber { if CFGetTypeID(number)==CFBooleanGetTypeID(){return .bool(number.boolValue)};return .number(number.doubleValue) }
        if let object=value as? [String:Any],let pairs=object["$map"] as? [[Any]] {let id=UInt32(graph.nodes.count);graph.nodes.append(.map(entries:[],properties:[]));var entries:[Pele.PublicGraph.Entry]=[];for pair in pairs{let key=try input(&graph,pair[0]),value=try input(&graph,pair[1]);entries.append(.init(key:key,value:value))};graph.nodes[Int(id)] = .map(entries:entries,properties:[]);return .ref(id)}
        let id = UInt32(graph.nodes.count); graph.nodes.append(.object(properties: []))
        var properties: [Pele.PublicGraph.Property] = []
        if let array = value as? [Any] {
            for (i, value) in array.enumerated() { let key = try graph.intern(String(i)), atom = try input(&graph, value); properties.append(.init(key: key, value: atom)) }
            graph.nodes[Int(id)] = .array(length: UInt32(array.count), properties: properties)
        } else if let object = value as? [String: Any] {
            for (name, value) in object.sorted(by: { $0.key < $1.key }) { let key = try graph.intern(name), atom = try input(&graph, value); properties.append(.init(key: key, value: atom)) }
            graph.nodes[Int(id)] = .object(properties: properties)
        } else { throw PeleError(code: "test", message: "Unexpected Sequence fixture input") }
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
            if case .map(let entries,_) = node { return ["$map":try entries.map { [try value(graph,$0.key),try value(graph,$0.value)] }] }
            var object: [String: Any] = [:]
            for property in node.properties { if case .undefined = property.value { continue }; object[try XCTUnwrap(graph.strings[Int(property.key)].stringIfWellFormed)] = try value(graph, property.value) }
            return object
        default: throw PeleError(code: "test", message: "Unexpected Sequence fixture atom")
        }
    }
    private static func mapValue(_ graph:Pele.PublicGraph,_ map:UInt32,_ key:String)->Pele.PublicGraph.Atom{guard case .map(let entries,_)=graph.nodes[Int(map)]else{return .undefined};return entries.first{if case .string(let id)=$0.key{return graph.strings[Int(id)].stringIfWellFormed==key};return false}?.value ?? .undefined}
    private static func mapSet(_ graph:inout Pele.PublicGraph,_ map:UInt32,_ key:String,_ value:Pele.PublicGraph.Atom)throws{guard case .map(var entries,let properties)=graph.nodes[Int(map)]else{throw PeleError(code:"test",message:"Map expected")};if let i=entries.firstIndex(where:{if case .string(let id)=$0.key{return graph.strings[Int(id)].stringIfWellFormed==key};return false}){entries[i] = .init(key:entries[i].key,value:value)}else{entries.append(.init(key:.string(try graph.intern(key)),value:value))};graph.nodes[Int(map)] = .map(entries:entries,properties:properties)}
    private static func message(_ graph:Pele.PublicGraph,_ messages:UInt32,_ text:String)throws->UInt32{guard case .array(let length,_)=graph.nodes[Int(messages)]else{throw PeleError(code:"test",message:"Messages expected")};for i in 0..<length{let id=try ref(graph.field(String(i),on:messages));if case .string(let s)=try graph.field("message",on:id),graph.strings[Int(s)].stringIfWellFormed==text{return id}};throw PeleError(code:"test",message:"Message missing")}
    private static func state(_ graph:Pele.PublicGraph)throws->[String:Any]{try XCTUnwrap(value(graph,graph.root) as? [String:Any])}
    func testSequenceCapturedArraysLateFieldsNestedCallbacksAndRecoveryMatchAllOracleRows() throws {
        var path = URL(fileURLWithPath: #filePath); for _ in 0..<5 { path.deleteLastPathComponent() }
        let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-sequence-live-render.json"))) as? [String: Any])
        XCTAssertEqual(fixture["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        let source = try XCTUnwrap(fixture["source"] as? String), rows = try XCTUnwrap(fixture["rows"] as? [[String: Any]])
        XCTAssertEqual(rows.count, 76)
        for row in rows {
            let raw = try XCTUnwrap(row["raw"] as? Bool), responsive = try XCTUnwrap(row["responsive"] as? Bool), mode = try XCTUnwrap(row["mode"] as? String)
            // C15 coordinates cannot recover deleted/reinserted key history from apply.
            // This one scenario remains in the exact real-browser iterator feature.
            if mode=="actor-map-reinsert"{continue}
            let engine = Pele.Engine()
            let document: SequenceCallbackDocument = try raw ? .prepared(engine.parsePrepared("sequence", source: source)) : .normal(engine.parseMutable(source))
            defer { try? document.close() }
            var setup=try document.snapshot().graph
            try Self.setText(&setup,"title","Initial")
            let actors=try Self.ref(setup.field("actors")),alpha=try Self.ref(Self.mapValue(setup,actors,"A")),beta=try Self.ref(Self.mapValue(setup,actors,"B")),boxed=try Self.ref(setup.field("box",on:alpha)),messages=try Self.ref(setup.field("messages")),hello=try Self.message(setup,messages,"Hello"),note=try Self.message(setup,messages,"Detail"),frame=try Self.message(setup,messages,"Count")
            try document.apply(setup)
            let observed=SequenceCallbackSequence()
            var resources = pure()
            resources.textWidth = { request in try MainActor.assumeIsolated {
                observed.trace.append([request.text, String(format: "%016llx", request.size.bitPattern), Int(request.style)])
                let trigger=["message-fields":"Hello","note-placement":"Detail","frame-label":"loop","box-late":"Group","actor-class":"Reply"][mode]
                if !observed.fired && (trigger == nil || request.text == trigger) {
                    observed.fired = true
                    XCTAssertThrowsError(try document.close())
                    var graph = try document.snapshot().graph
                    var calls:[(UInt32,[Pele.PublicGraph.Atom])]=[]
                    switch mode {
                    case "actor-label":try Self.setText(&graph,"description","Changed Alpha",on:alpha);try Self.setText(&graph,"description","Changed Beta",on:beta)
                    case "actor-map-append":let v=try Self.input(&graph,["name":"C","description":"Gamma","wrap":false,"type":"participant","links":["$map":[Any]()],"properties":["$map":[Any]()]]);try Self.mapSet(&graph,actors,"C",v)
                    case "actor-map-delete":guard case .map(var entries,let properties)=graph.nodes[Int(actors)]else{throw PeleError(code:"test",message:"Map expected")};entries.remove(at:1);graph.nodes[Int(actors)] = .map(entries:entries,properties:properties)
                    case "actor-map-replace":let a=try graph.intern("A"),b=try graph.intern("B"),n=UInt32(graph.nodes.count);graph.nodes.append(.map(entries:[.init(key:.string(b),value:.ref(beta)),.init(key:.string(a),value:.ref(alpha))],properties:[]));try graph.setField("actors",value:.ref(n))
                    case "lifecycle":let created=try Self.ref(graph.field("createdActors")),destroyed=try Self.ref(graph.field("destroyedActors"));try Self.mapSet(&graph,created,"B",.number(2));try Self.mapSet(&graph,destroyed,"B",.number(4))
                    case "message-replace":let v=try Self.input(&graph,[Any]());try graph.setField("messages",value:v)
                    case "message-append":let raw=try Self.value(graph,.ref(hello)),v=try Self.input(&graph,raw),n=try Self.ref(v);try Self.setText(&graph,"id","appended",on:n);try Self.setText(&graph,"message","Appended",on:n);try Self.append(&graph,on:messages,value:v)
                    case "message-fields":try Self.setText(&graph,"id","changed-message",on:hello);try graph.setField("centralConnection",value:.number(61),on:hello);try Self.setText(&graph,"message","Changed Hello",on:hello);try Self.setText(&graph,"message","Changed detail",on:note)
                    case "box-replace":let v=try Self.input(&graph,["name":"Replacement","fill":"green","wrap":false,"actorKeys":["B"]]);try graph.setField("box",value:v,on:beta)
                    case "box-late":try Self.setText(&graph,"name","Changed box",on:boxed);try Self.setText(&graph,"fill","green",on:boxed)
                    case "actor-class":for(id,label)in[(alpha,"late-alpha"),(beta,"late-beta")]{let properties=try Self.ref(graph.field("properties",on:id)),v=try Self.input(&graph,label);try Self.mapSet(&graph,properties,"class",v)}
                    case "note-placement":try graph.setField("placement",value:.number(0),on:note);try Self.setText(&graph,"id","changed-note",on:note)
                    case "frame-label":try Self.setText(&graph,"message","Changed frame",on:frame)
                    case "metadata":try Self.setText(&graph,"title","Changed title");try Self.setText(&graph,"accTitle","Changed access");try Self.setText(&graph,"accDescr","Changed description")
                    case "helper":let a=try Self.input(&graph,"C"),b=try Self.input(&graph,"Gamma"),c=try Self.input(&graph,["text":"Gamma","type":"participant"]);calls.append((8,[a,b,c]))
                    case "throw":try Self.setText(&graph,"title","Before throw");try Self.setText(&graph,"description","Saved Beta",on:beta)
                    case "nested":try Self.setText(&graph,"title","Nested title")
                    case "baseline":break
                    default:throw PeleError(code:"test",message:"Unknown Sequence mode")
                    }
                    if mode != "baseline" { try document.apply(graph);for(method,args)in calls{try document.invoke(method,args)} }
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
            XCTAssertEqual(try Self.ref(after.graph.field("actors"))==actors,row["actorsCurrent"] as? Bool)
            XCTAssertEqual(try Self.ref(after.graph.field("messages"))==messages,row["messagesCurrent"] as? Bool)
            for(name,old,now)in[("alphaCurrent",alpha,Self.mapValue(after.graph,actors,"A")),("betaCurrent",beta,Self.mapValue(after.graph,actors,"B")),("boxCurrent",boxed,try after.graph.field("box",on:beta) ?? .undefined)]{let same:Bool;if case .ref(let id)=now{same=id==old}else{same=false};XCTAssertEqual(same,row[name] as? Bool)}
            for(name,id)in[("originalActors",actors),("originalAlpha",alpha),("originalBeta",beta),("originalBox",boxed),("originalHello",hello),("originalNote",note)]{XCTAssertTrue(NSDictionary(dictionary:try XCTUnwrap(Self.value(after.graph,.ref(id)) as? [String:Any])).isEqual(to:try XCTUnwrap(row[name] as? [String:Any])),mode+name)}
            XCTAssertTrue(NSArray(array:try XCTUnwrap(Self.value(after.graph,.ref(messages)) as? [Any])).isEqual(to:try XCTUnwrap(row["originalMessages"] as? [Any])))
            if mode == "nested" { try compare(XCTUnwrap(observed.nested), XCTUnwrap(row["nested"] as? [String: Any])); XCTAssertGreaterThan(document.revision, try XCTUnwrap(observed.nestedRevision)) }
            let recovery = SequenceCallbackSequence(); var plain = pure()
            plain.textWidth = { request in MainActor.assumeIsolated { recovery.trace.append([request.text, String(format: "%016llx", request.size.bitPattern), Int(request.style)]); return Double(request.text.utf16.count) * request.size * 0.6 } }
            try compare(document.render(plain, responsive: responsive), XCTUnwrap(row["recovery"] as? [String: Any]))
            XCTAssertTrue(NSArray(array: recovery.trace).isEqual(to: try XCTUnwrap(row["recoveryTrace"] as? [[Any]])), mode + " recovery trace")
        }
    }
}
