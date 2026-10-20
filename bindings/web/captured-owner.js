// Private opt-in owner. Diagram algorithms, projection and admission stay native.
// No public loader enables this until the complete captured family is proved.
import {FlatGraph,FlatStorage,bindCapturedStringTable} from './captured-graph.js';
import {createCapturedContext,capturedEpoch,closeCapturedContext} from './captured-store.js';
import {getGraphTraversal,numberValue} from './graph-storage.js';
import {PeleError} from './errors.js';

const traversal=getGraphTraversal(FlatStorage),models=new WeakMap();
let testingEnabled=false;
export function enableCapturedSankeyForTesting(value=true){testingEnabled=value===true;}
export function capturedSankeyBackend(model){const owner=models.get(model);return owner?(owner.disposed?'disposed':owner.lane):undefined;}
export function capturedSankeyAvailable(registry,native){
  const methods=['apply_captured','invoke_captured','render_captured','render_raw_captured','revision','schema','live_provider_view'];
  return testingEnabled&&['parse_with_host_storage','parse_prepared_with_captured_sankey','select_prepared_captured_sankey','bind_prepared_captured_sankey'].every(key=>typeof registry?.[key]==='function')&&methods.every(key=>typeof native.CapturedSankeyDocument?.prototype[key]==='function');
}
export function rememberCapturedModel(model,owner,lane){owner.lane=lane;models.set(model,owner);}
const invalidPacket=()=>{throw new TypeError('Invalid captured model outcome');};
const invalidBinary=()=>{throw new PeleError('Invalid public binary graph transport','semantic');};
const frameLimit=()=>{throw new PeleError('Public binary graph exceeds transport budget','limit');};
const incomplete=()=>{throw new TypeError('Public model construction is incomplete');};
const disposed=()=>{throw new TypeError('Public model document is disposed');};
const BUDGET=64*1024*1024;
function missingProperties(){const node=undefined;return node.properties;}

// Same pre-config stage as encodeGraphBinary: count the full canonical frame,
// then reject invalid Array/Error scalar encoding. IDs/history are not rewritten.
export function capturedObservedFrame(graph,metadataBytes=0){
  for(let id=0;id<graph.nodeCount;id++)if(graph.kindCode(id)===0xffffffff)missingProperties();
  const bytes=graph.encodedGraphBytes(metadataBytes);
  if(!Number.isSafeInteger(bytes)||bytes>BUDGET)frameLimit();
  if(graph.firstInvalidScalarNode()!==undefined)invalidBinary();
  return bytes;
}
function sameAtom(a,as,b,bs){
  return a.atomWord(as,0)===b.atomWord(bs,0)&&a.atomWord(as,1)===b.atomWord(bs,1)&&a.atomWord(as,2)===b.atomWord(bs,2);
}
// Exact canonical equality, not a hash or a live re-read. Native apply-noop is
// allowed only under the facade's existing confirmed/unchanged conditions.
function sameGraph(a,b){
  if(a.nodeCount!==b.nodeCount||a.stringCount!==b.stringCount||!sameAtom(a,0,b,0))return false;
  // Observation inherits immutable string history; equal counts imply no suffix.
  for(let id=0;id<a.nodeCount;id++){
    if(a.kindCode(id)!==b.kindCode(id))return false;
    if(a.kindCode(id)===0xffffffff)continue;
    if(a.headerWord(id,1)!==b.headerWord(id,1)||a.headerWord(id,2)!==b.headerWord(id,2)||a.headerWord(id,4)!==b.headerWord(id,4)||a.headerWord(id,6)!==b.headerWord(id,6))return false;
    for(let i=0,count=a.headerWord(id,4);i<count;i++){
      const aa=a.propertyAddress(id,i),bb=b.propertyAddress(id,i);
      for(let c=0;c<5;c++)if(a.propertyWord(aa,c)!==b.propertyWord(bb,c))return false;
    }
    const count=a.headerWord(id,6)*(a.kindCode(id)===3?2:1);
    for(let i=0;i<count;i++)if(!sameAtom(a,FlatStorage.itemAtomSlot(a,id,i),b,FlatStorage.itemAtomSlot(b,id,i)))return false;
  }
  return true;
}
function atomMetadata(graph,slot){
  const tag=graph.atomWord(slot,0),value=graph.atomWord(slot,1);
  switch(tag){
    case 0:return {kind:'undefined'};case 1:return {kind:'null'};
    case 2:return {kind:'bool',value:false};case 3:return {kind:'bool',value:true};
    case 4:return {kind:'number',value:FlatStorage.atomNumberBits(graph,slot)};
    case 5:return {kind:'string',value};case 6:return {kind:'ref',value};
    case 7:return {kind:'builtinFunction',value};case 8:return {kind:'hostReference',value};
    default:invalidPacket();
  }
}
function renderPacket(packet,metadataBytes,raw){
  if(!Array.isArray(packet)||packet.length!==3||packet[0]!==1||packet[1]!==metadataBytes)invalidPacket();
  const value=packet[2],keys=raw?['svg','width','height','links']:['type','svg','width','height','links'],actual=value&&Object.keys(value);
  if(!value||Object.getPrototypeOf(value)!==Object.prototype||actual.length!==keys.length||!keys.every((key,i)=>actual[i]===key)||typeof value.svg!=='string'||typeof value.width!=='number'||typeof value.height!=='number'||!Array.isArray(value.links)||(!raw&&value.type!=='sankey'))invalidPacket();
  for(const link of value.links){const fields=link&&Object.keys(link);if(!link||Object.getPrototypeOf(link)!==Object.prototype||fields.length!==3||fields[0]!=='id'||fields[1]!=='href'||fields[2]!=='internal'||typeof link.id!=='string'||typeof link.href!=='string'||typeof link.internal!=='boolean')invalidPacket();}
  return {Ok:value};
}
function outcomePacket(response,expected,graph,kind){
  if(!Array.isArray(response)||response.length!==5||response[0]!==1||typeof response[1]!=='string'||response[1]!==String(BigInt(expected)+1n)||!Number.isSafeInteger(response[2])||response[2]<0||response[2]>BUDGET||(response[3]!==0&&response[3]!==1))invalidPacket();
  capturedObservedFrame(graph,response[2]);
  const outcome={revision:response[1],graph};
  if(response[3]===1){outcome.factoryError=response[4];return outcome;}
  if(typeof response[4]==='string'){
    if(new TextEncoder().encode(response[4]).length!==response[2])invalidPacket();
    outcome.result=JSON.parse(response[4]);
    if(!outcome.result||Object.hasOwn(outcome.result,'Ok')===Object.hasOwn(outcome.result,'Err'))invalidPacket();
  }else{
    if(kind==='method')invalidPacket();
    outcome.result=renderPacket(response[4],response[2],kind==='raw');
  }
  return outcome;
}

export class CapturedOwner {
  constructor(registration,registry,callNative,decodeEnvelope,decodeError){
    this.registry=registry;this.registration=registration;
    if(registration.documents.length>256)registration.documents=registration.documents.filter(reference=>{const value=reference.deref();return value&&!value.disposed;});
    registration.documents.push(new WeakRef(this));
    this.disposed=false;this.invoke=incomplete;
    this.ids=new WeakMap();this.hostIds=new Map();this.hostValues=[];this.objects=[];
    this.strings=[];this.stringIds=new Map();this.root=undefined;
    this.graph=new FlatGraph();bindCapturedStringTable(this.graph,this.strings);
    this.accepted=null;this.acceptedRevision='0';
    this.callNative=callNative;this.decodeEnvelope=decodeEnvelope;this.decodeError=decodeError;
  }
  reconcile(graph){
    try{return traversal.reconcileGraph(this,graph);}
    finally{if(this.strings?.length>=graph.stringCount)bindCapturedStringTable(graph,this.strings);}
  }
  snapshot(values=[]){return traversal.snapshotGraph(this,values,false);}
  matchesPublishedGraph(){return traversal.matchesGraph(this);}
  hostReferenceId(value){
    if(!this.hostIds.has(value)){
      if(this.hostValues.length>=1_000_000)throw new RangeError('Host reference budget exceeded');
      this.hostIds.set(value,this.hostValues.length);this.hostValues.push(value);
    }
    return this.hostIds.get(value);
  }
  value(atom){
    switch(atom.kind){
      case 'undefined':return undefined;case 'null':return null;case 'bool':return atom.value;
      case 'number':return numberValue(atom.value);case 'string':return this.strings[atom.value];
      case 'ref':return this.objects[atom.value];case 'builtinFunction':return this.registration.functions.get(atom.value);
      case 'hostReference':return this.hostValues[atom.value];default:throw new TypeError('Unknown public atom kind');
    }
  }
  attach(entry,graph){
    this.accepted=graph;this.acceptedRevision=entry.revision;
    this.invoke=(method,receiver,args)=>this.invokeMethod(entry,method,receiver,args);
  }
  busy(entry,run){
    if(entry.nativeBusy)throw new TypeError('Public model document is already in use');
    entry.nativeBusy=true;try{return run();}finally{entry.nativeBusy=false;}
  }
  commit(context,revision){
    // Native commit is not facade acknowledgment. Even a later transport error
    // keeps this accepted W and leaves entry.revision stale, matching arena.
    this.accepted=capturedEpoch(context);this.acceptedRevision=revision;
  }
  prepare(entry,values=[]){
    const confirmed=entry.confirmedGraph;entry.confirmedGraph=false;
    const baseline=this.graph;
    if(confirmed&&entry.unchangedApplyIsNoop&&values.length===0&&this.matchesPublishedGraph())return {graph:baseline,values:[]};
    const input=this.snapshot(values);
    try{
      const unchanged=confirmed&&entry.unchangedApplyIsNoop&&sameGraph(baseline,input.graph);
      if(!unchanged){
        capturedObservedFrame(input.graph);
        const context=createCapturedContext(this,input.graph,[],false);
        try{entry.revision=this.decodeEnvelope(this.callNative(()=>entry.handle.apply_captured(context,entry.revision,revision=>this.commit(context,revision))));}
        finally{closeCapturedContext(context);}
      }
      return input;
    }catch(error){input.graph.releaseArguments();throw error;}
  }
  publish(entry,outcome){
    entry.revision=outcome.revision;
    this.reconcile(outcome.graph);
    entry.confirmedGraph=true;
    if(Object.hasOwn(outcome,'factoryError'))throw outcome.factoryError;
  }
  packet(entry,kind,call){
    const context=createCapturedContext(this,this.accepted,[],kind==='method');
    try{
      const response=this.callNative(()=>call(context,revision=>this.commit(context,revision)));
      const graph=capturedEpoch(context),outcome=outcomePacket(response,entry.revision,graph,kind);
      this.publish(entry,outcome);return outcome.result;
    }finally{closeCapturedContext(context);}
  }
  invokeMethod(entry,method,receiver,args){
    return this.busy(entry,()=>{
      if(entry.disposed)disposed();
      const input=this.prepare(entry,[receiver,...args]);
      let callJSON;
      try{callJSON=JSON.stringify({method,receiver:atomMetadata(input.graph,input.values[0]),arguments:input.values.slice(1).map(slot=>atomMetadata(input.graph,slot))});}
      finally{input.graph.releaseArguments();}
      const result=this.packet(entry,'method',(context,commit)=>entry.handle.invoke_captured(context,entry.revision,callJSON,commit));
      if(Object.hasOwn(result,'Err'))throw this.decodeError(result.Err,this);
      return this.value(result.Ok);
    });
  }
  render(entry,numericArguments){
    return this.busy(entry,()=>{
      const input=this.prepare(entry);input.graph.releaseArguments();
      return this.packet(entry,'render',(context,commit)=>entry.handle.render_captured(context,entry.revision,...numericArguments,commit));
    });
  }
  release(){
    this.disposed=true;this.accepted=null;this.graph=null;this.strings=null;this.stringIds=null;
    this.objects=null;this.root=null;this.hostIds=null;this.hostValues=null;
    this.invoke=disposed;this.activeInvoke=undefined;
  }
}
