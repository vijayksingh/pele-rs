import Foundation
import Pele
import XCTest
private final class FlowSubgraphFailure: Error {}
@MainActor private enum FlowSubgraphDocument {
 case normal(Pele.MutableDocument), prepared(Pele.PreparedDocument)
 var revision:UInt64{switch self{case .normal(let d):d.revision;case .prepared(let d):d.revision}}
 func snapshot()throws->Pele.PublicSnapshot{switch self{case .normal(let d):try d.snapshot();case .prepared(let d):try d.snapshot()}}
 func apply(_ graph:Pele.PublicGraph)throws{switch self{case .normal(let d):_ = try d.apply(graph,expectedRevision:d.revision);case .prepared(let d):_ = try d.apply(graph,expectedRevision:d.revision)}}
 func bind(_ f:Pele.HostFunction)throws->Pele.PublicGraph.Atom{switch self{case .normal(let d):try d.bindHostFunction(f);case .prepared(let d):try d.bindHostFunction(f)}}
 func render(_ resources:Pele.HostResources,_ responsive:Bool)throws->FullRenderedDiagram{var options=Pele.Engine.Options();options.maxWidth=responsive ? 200:nil;switch self{case .normal(let d):return try d.render(options:options,resources:resources);case .prepared(let d):return try d.render(options:options,resources:resources)}}
 func close()throws{switch self{case .normal(let d):try d.close();case .prepared(let d):try d.close()}}
}
@MainActor private final class FlowSubgraphState {
 var fired=false;var redrawn=false;var trace:[[Any]]=[];var nested:FullRenderedDiagram?;let failure=FlowSubgraphFailure();var functions:[Pele.HostFunction]=[]
}
@MainActor final class NativeFlowSubgraphCallbackTests:XCTestCase {
 private static func bits(_ value:Double)->String{String(format:"%016llx",value.bitPattern)}
 private static func ref(_ value:Pele.PublicGraph.Atom?)throws->UInt32{guard case .ref(let id)=value else{throw FlowSubgraphFailure()};return id}
 private static func same(_ a:Pele.PublicGraph.Atom?,_ b:Pele.PublicGraph.Atom?)->Bool{if case .ref(let x)=a,case .ref(let y)=b{return x==y};return false}
 private static func json(_ value:Any)throws->ConfigValue{try JSONDecoder().decode(ConfigValue.self,from:JSONSerialization.data(withJSONObject:value,options:[.fragmentsAllowed]))}
 private static func map(_ graph:Pele.PublicGraph,_ key:String,_ name:String)throws->UInt32{let id=try ref(graph.field(key));guard case .map(let entries,_)=graph.nodes[Int(id)]else{throw FlowSubgraphFailure()};for e in entries{if case .string(let k)=e.key,graph.strings[Int(k)].stringIfWellFormed==name{return try ref(e.value)}};throw FlowSubgraphFailure()}
 private static func text(_ graph:inout Pele.PublicGraph,_ key:String,_ text:String,on:UInt32?=nil)throws{let id=try graph.intern(text);try graph.setField(key,value:.string(id),on:on)}
 private static func record(_ graph:inout Pele.PublicGraph,_ fields:[(String,Pele.PublicGraph.Atom)])throws->UInt32{let id=UInt32(graph.nodes.count);graph.nodes.append(.object(properties:[]));for(k,v)in fields{try graph.setField(k,value:v,on:id)};return id}
 private static func emptyArray(_ graph:inout Pele.PublicGraph)->UInt32{let id=UInt32(graph.nodes.count);graph.nodes.append(.array(length:0,properties:[]));return id}
 private static func value(_ graph:Pele.PublicGraph,_ atom:Pele.PublicGraph.Atom)throws->Any{switch atom{
 case .undefined,.null:return NSNull();case .bool(let v):return v;case .number(let v):return v;case .string(let id):return try XCTUnwrap(graph.strings[Int(id)].stringIfWellFormed)
 case .ref(let id):if case .array(let count,_)=graph.nodes[Int(id)]{return try (0..<count).map{try value(graph,graph.field(String($0),on:id) ?? .undefined)}};var out:[String:Any]=[:];for p in graph.nodes[Int(id)].properties{if case .undefined=p.value{continue};out[try XCTUnwrap(graph.strings[Int(p.key)].stringIfWellFormed)]=try value(graph,p.value)};return out
 default:throw FlowSubgraphFailure()}}
 private static func state(_ graph:Pele.PublicGraph)throws->ConfigValue{var out:[String:Any]=[:];for key in["title","accTitle","accDescr","direction","nodes","edges","subgraphs","subgraphLookup"]{let atom=try graph.field(key) ?? .undefined;if case .undefined=atom{continue};if key=="nodes"||key=="subgraphLookup"{let id=try ref(atom);guard case .map(let entries,_)=graph.nodes[Int(id)]else{throw FlowSubgraphFailure()};out[key=="subgraphLookup" ? "lookup":key]=try entries.map{try [value(graph,$0.key),value(graph,$0.value)]}}else{out[key]=try value(graph,atom)}};return try json(out)}
 private static func compare(_ actual:FullRenderedDiagram,_ expected:[String:Any],_ id:String)throws{XCTAssertEqual(actual.svg,expected["svg"] as? String,id);XCTAssertEqual(bits(actual.width),expected["width"] as? String,id);XCTAssertEqual(bits(actual.height),expected["height"] as? String,id);XCTAssertEqual(try JSONDecoder().decode(ConfigValue.self,from:JSONEncoder().encode(actual.links)),try json(XCTUnwrap(expected["links"])),id)}
 private static func resources(_ state:FlowSubgraphState?=nil,document:FlowSubgraphDocument?=nil,mode:String="",replacement:Pele.PublicGraph.Atom?=nil,later:UInt32?=nil)->Pele.HostResources{var resources=Pele.HostResources();resources.textWidth={request in try MainActor.assumeIsolated{
  if let state{state.trace.append(["width",request.text,bits(request.size),request.style]);if mode=="redraw-replace" && !state.redrawn{state.redrawn=true;let document=try XCTUnwrap(document);var graph=try document.snapshot().graph;try graph.setField("subgraph",value:XCTUnwrap(replacement));try text(&graph,"text","After width",on:later);try document.apply(graph)}}
  return Double(request.text.utf16.count)*request.size*0.6
 }};return resources}
 func testLiveFlowSubgraphOverridesMatch80ActualOracleCases()throws{
  var path=URL(fileURLWithPath:#filePath);for _ in 0..<5{path.deleteLastPathComponent()};let fixture=try XCTUnwrap(JSONSerialization.jsonObject(with:Data(contentsOf:path.appendingPathComponent("fixtures/public-flow-subgraph-live.json"))) as? [String:Any]);let rows=try XCTUnwrap(fixture["rows"] as? [[String:Any]]);XCTAssertEqual(rows.count,80)
  for row in rows{
   let family=try XCTUnwrap(row["family"] as? String),mode=try XCTUnwrap(row["mode"] as? String),raw=try XCTUnwrap(row["raw"] as? Bool),responsive=try XCTUnwrap(row["responsive"] as? Bool),id="\(family)/\(mode)/\(raw)/\(responsive)";let state=FlowSubgraphState()
   let registry=Pele.Engine();let source="\(family=="flowchart" ? "flowchart":"swimlane-beta") LR\nsubgraph Outer[Outer group]\nsubgraph Inner[Inner group]\nA[Alpha] --> B[Beta]\nend\nC[Gamma]\nend\nB --> C"
   let document:FlowSubgraphDocument
   if raw{document = .prepared(try registry.parsePrepared(family,source:source))}else{document = .normal(try registry.parseMutable(source))};defer{try? document.close()}
   var graph=try document.snapshot().graph;let groups=try Self.ref(graph.field("subgraphs")),later=try Self.map(graph,"nodes","B"),inner=try Self.map(graph,"subgraphLookup","Inner"),outer=try Self.map(graph,"subgraphLookup","Outer");let view=try graph.intern("collapsed"),metadata=try Self.record(&graph,[("view",.string(view))]);try graph.setField("metadata",value:.ref(metadata),on:inner)
   let replacement=Pele.HostFunction{view,receiver,args in let key=try view.string(args[0]);state.trace.append(["replacement",key.stringReplacingInvalidCodeUnits,Self.same(receiver,try view.root())]);return try view.invokeBuiltin(method:19,receiver:receiver,arguments:args)}
   let replacementValue=try document.bind(replacement);state.functions.append(replacement)
   let callback=Pele.HostFunction{view,receiver,args in
    XCTAssertThrowsError(try document.close());let key=try view.string(args[0]);let name=key.stringReplacingInvalidCodeUnits;state.trace.append(["subgraph",name,Self.same(receiver,try view.root())])
    if !state.fired{state.fired=true;var current=try document.snapshot().graph;switch mode{
     case "label":try Self.text(&current,"title","Changed title");try Self.text(&current,"text","Changed Beta",on:later);try document.apply(current)
     case "captured-groups":let empty=Self.emptyArray(&current);try current.setField("subgraphs",value:.ref(empty));try Self.text(&current,"title","Captured outer",on:outer);try document.apply(current)
     case "append-member":let nodes=try Self.ref(current.field("nodes",on:outer));guard case .array(let count,_)=current.nodes[Int(nodes)]else{throw FlowSubgraphFailure()};let late=try current.intern("Late");try current.setArrayElement(count,value:.string(late),on:nodes);try document.apply(current)
     case "throw-object","throw-undefined":try Self.text(&current,"title","Saved");try document.apply(current);throw state.failure
     case "nested":state.nested=try document.render(Self.resources(),false);current=try document.snapshot().graph;try Self.text(&current,"title","After nested");try document.apply(current)
     default:break}}
    if mode=="false-parent"{return .undefined}
    if mode=="collapsed-return" && name=="Outer"{let object=try view.object(),metadata=try view.object();try view.set("view",on:metadata,to:view.intern("collapsed"));try view.set("metadata",on:object,to:metadata);return object}
    return try view.invokeBuiltin(method:19,receiver:receiver,arguments:args)
   };state.functions.append(callback);let callbackValue=try document.bind(callback);try graph.setField("subgraph",value:callbackValue);try document.apply(graph)
   let first=try XCTUnwrap(row["first"] as? [String:Any]);do{let result=try document.render(Self.resources(state,document:document,mode:mode,replacement:replacementValue,later:later),responsive);XCTAssertEqual(first["ok"] as? Bool,true,id);try Self.compare(result,first,id)}catch{XCTAssertEqual(first["ok"] as? Bool,false,id);XCTAssertTrue((error as AnyObject)===state.failure,id)}
   XCTAssertTrue(state.fired,id);XCTAssertEqual(state.redrawn,row["redrawn"] as? Bool,id);XCTAssertEqual(try Self.json(state.trace),try Self.json(XCTUnwrap(row["trace"])),id);graph=try document.snapshot().graph;XCTAssertEqual(try Self.state(graph),try Self.json(XCTUnwrap(row["after"])),id);XCTAssertEqual(Self.same(.ref(groups),try graph.field("subgraphs")),row["groupsCurrent"] as? Bool,id);XCTAssertEqual(try Self.map(graph,"nodes","B"),later,id)
   if let nested=state.nested{try Self.compare(nested,XCTUnwrap(row["nested"] as? [String:Any]),id)}
   try graph.setField("subgraph",value:.builtinFunction(19));try document.apply(graph);try Self.compare(document.render(Self.resources(),responsive),XCTUnwrap(row["recovery"] as? [String:Any]),id);XCTAssertEqual(try Self.state(document.snapshot().graph),try Self.json(XCTUnwrap(row["afterRecovery"])),id)
  }
 }
}
