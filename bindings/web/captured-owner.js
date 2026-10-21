// Private opt-in owner. Diagram algorithms, projection and admission stay native.
// No public loader enables this until the complete captured family is proved.
import {NativeGraphView,NativeStorage} from './native-storage.js';
import {getGraphTraversal,numberValue} from './graph-storage.js';
import {PeleError} from './errors.js';

const traversal=getGraphTraversal(NativeStorage),models=new WeakMap();
let testingEnabled=false;
export function enableCapturedSankeyForTesting(value=true){testingEnabled=value===true;}
export function capturedSankeyBackend(model){const owner=models.get(model);return owner?(owner.disposed?'disposed':owner.lane):undefined;}
export function capturedSankeyStorage(model){const owner=models.get(model);return owner?.graph instanceof NativeGraphView?'native':undefined;}
export function capturedSankeyAvailable(registry,native){
  const methods=['initial_epoch','begin_observation','publication_started','apply_native','invoke_native','render_native','render_raw_native','revision','schema','live_provider_view'];
  return testingEnabled&&typeof native.native_canonical_memory==='function'&&['parse_with_native_storage','parse_prepared_with_native_sankey','select_prepared_native_sankey','create_prepared_native_sankey','bind_prepared_native_sankey'].every(key=>typeof registry?.[key]==='function')&&methods.every(key=>typeof native.NativeCanonicalSankeyDocument?.prototype[key]==='function');
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
  const result=graph.preflight(metadataBytes);
  if(!(result instanceof Uint32Array)||result.length!==2)invalidPacket();
  switch(result[0]){case 0:return result[1];case 1:return missingProperties();case 2:return invalidBinary();case 3:return frameLimit();default:invalidPacket();}
}
function atomMetadata(graph,slot){
  const tag=graph.atomWord(slot,0),value=graph.atomWord(slot,1);
  switch(tag){
    case 0:return {kind:'undefined'};case 1:return {kind:'null'};
    case 2:return {kind:'bool',value:false};case 3:return {kind:'bool',value:true};
    case 4:return {kind:'number',value:NativeStorage.atomNumberBits(graph,slot)};
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
  if(!Array.isArray(response)||response.length!==6||response[0]!==1||typeof response[1]!=='string'||response[1]!==String(BigInt(expected)+1n)||!Number.isSafeInteger(response[2])||response[2]<0||response[2]>BUDGET||(response[3]!==0&&response[3]!==1))invalidPacket();
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
  constructor(registration,registry,callNative,decodeEnvelope,decodeError,memory,transport){
    this.registry=registry;this.registration=registration;
    if(registration.documents.length>256)registration.documents=registration.documents.filter(reference=>{const value=reference.deref();return value&&!value.disposed;});
    registration.documents.push(new WeakRef(this));
    this.disposed=false;this.invoke=incomplete;
    this.ids=new WeakMap();this.hostIds=new Map();this.hostValues=[];this.objects=[];
    this.strings=[];this.stringIds=new Map();this.root=undefined;
    this._graph=null;this.memory=memory;this.transport=transport;
    this.callNative=callNative;this.decodeEnvelope=decodeEnvelope;this.decodeError=decodeError;
  }
  get graph(){return this._graph;}
  set graph(value){if(value===this._graph)return;value?.retain();const previous=this._graph;this._graph=value;previous?.release();}
  view(epoch){return new NativeGraphView(epoch,this.memory,this);}
  seed(epoch){const graph=this.view(epoch);try{this.graph=graph;}finally{graph.release();}}
  reconcile(graph){
    graph.retain();
    try{
      const prefix=this.callNative(()=>this.transport.publication_started(graph.epoch));
      graph.useStrings(this.strings,Math.min(prefix,this.strings.length));
      return traversal.reconcileGraph(this,graph);
    }finally{
      if(this.graph===graph&&this.strings?.length>=graph.stringCount)graph.useStrings(this.strings,graph.stringCount);
      graph.release();
    }
  }
  snapshot(values=[]){
    const baseline=this.graph.retain();
    try{const result=traversal.snapshotGraph(this,values,false);result.graph.release();return result;}
    finally{baseline.release();}
  }
  matchesPublishedGraph(){const graph=this.graph.retain();try{return traversal.matchesGraph(this);}finally{graph.release();}}
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
  attach(entry){
    this.transport=entry.handle;
    this.invoke=(method,receiver,args)=>this.invokeMethod(entry,method,receiver,args);
  }
  busy(entry,run){
    if(entry.nativeBusy)throw new TypeError('Public model document is already in use');
    entry.nativeBusy=true;try{return run();}finally{entry.nativeBusy=false;}
  }
  prepare(entry,values=[]){
    const confirmed=entry.confirmedGraph;entry.confirmedGraph=false;
    const baseline=this.graph.retain();
    try{
      if(confirmed&&entry.unchangedApplyIsNoop&&values.length===0&&this.matchesPublishedGraph())return {graph:baseline,values:[]};
      const input=this.snapshot(values);
      try{
        const unchanged=confirmed&&entry.unchangedApplyIsNoop&&baseline.exactEquals(input.graph);
        if(!unchanged){
          capturedObservedFrame(input.graph);
          entry.revision=this.decodeEnvelope(this.callNative(()=>entry.handle.apply_native(input.graph.epoch,entry.revision)));
        }
        return input;
      }catch(error){input.graph.releaseArguments();throw error;}
    }finally{baseline.release();}
  }
  publish(entry,outcome){
    entry.revision=outcome.revision;
    this.reconcile(outcome.graph);
    entry.confirmedGraph=true;
    if(Object.hasOwn(outcome,'factoryError'))throw outcome.factoryError;
  }
  packet(entry,kind,call){
    const response=this.callNative(call);
    let graph;
    try{
      if(!Array.isArray(response)||response.length!==6)invalidPacket();
      graph=this.view(response[5]);
      const outcome=outcomePacket(response,entry.revision,graph,kind);
      this.publish(entry,outcome);return outcome.result;
    }finally{graph?.release();}
  }
  invokeMethod(entry,method,receiver,args){
    return this.busy(entry,()=>{
      if(entry.disposed)disposed();
      const input=this.prepare(entry,[receiver,...args]);
      let callJSON;
      try{callJSON=JSON.stringify({method,receiver:atomMetadata(input.graph,input.values[0]),arguments:input.values.slice(1).map(slot=>atomMetadata(input.graph,slot))});}
      finally{input.graph.releaseArguments();}
      const result=this.packet(entry,'method',()=>entry.handle.invoke_native(entry.revision,callJSON));
      if(Object.hasOwn(result,'Err'))throw this.decodeError(result.Err,this);
      return this.value(result.Ok);
    });
  }
  render(entry,numericArguments){
    return this.busy(entry,()=>{
      const input=this.prepare(entry);input.graph.releaseArguments();
      return this.packet(entry,'render',()=>entry.handle.render_native(entry.revision,...numericArguments));
    });
  }
  release(){
    this.disposed=true;this.graph=null;this.transport=null;this.strings=null;this.stringIds=null;
    this.objects=null;this.root=null;this.hostIds=null;this.hostValues=null;
    this.invoke=disposed;this.activeInvoke=undefined;
  }
}
