// Private host storage primitives. ER grammar, methods and rendering remain Rust.
import {PeleError} from './errors.js';
const constructors={error:Error,pele:PeleError,type:TypeError,range:RangeError,syntax:SyntaxError,reference:ReferenceError,eval:EvalError,uri:URIError};
const diagnostics=new Set(['stack','fileName','lineNumber','columnNumber','sourceURL']);
const bits=new DataView(new ArrayBuffer(8));
const numberBits=n=>{bits.setFloat64(0,n);return bits.getBigUint64(0).toString(16).padStart(16,'0');};
const numberValue=n=>{bits.setBigUint64(0,BigInt('0x'+n));return bits.getFloat64(0);};
const errorKind=value=>{const proto=Object.getPrototypeOf(value);return Object.keys(constructors).find(key=>proto===constructors[key].prototype);};
const graphBudget=()=>{throw new RangeError('Public model graph exceeds host exchange budget');};
const frameBudget=()=>{throw new PeleError('Public binary graph exceeds transport budget','limit');};
const accessorError=()=>{throw new TypeError('Accessor properties require an explicit public model codec');};
const decodeUnits=units=>{let value='';for(let n=0;n<units.length;n+=8192)value+=String.fromCharCode(...units.slice(n,n+8192));return value;};
const TEMP=0x80000000;
const atomBytes=atom=>atom[0]===4?9:atom[0]>=5?5:1;

// Abstract string conversion used by storage primitives. It deliberately lacks
// String(symbol)'s constructor-only exception: template/join ToString throws.
function primitiveText(value){return `${value}`;}
// Named parameters retain the host engine's diagnostic receiver spelling even
// after repeated failing calls optimize the surrounding dispatcher.
function cellStringFailure(e){return e.includes("~");}
function labelStringFailure(e){return e.includes("&");}
function accessibilityStringFailure(e){const vx="¶ß";return e.includes(vx);}
// Preserve the host engine's intrinsic cyclic-join policy without expanding
// caller data before checking its budget. Some engines throw instead of eliding.
const cyclicJoinError=(()=>{const values=[];values.push(values);try{Array.prototype.join.call(values);return undefined;}catch(error){return error;}})();
function scalar(value){return value===null||(typeof value!=='object'&&typeof value!=='function');}
function conversionState(){return {active:new Set(),work:0,depth:0};}
function conversionLimit(){throw new PeleError('ER native graph operation exceeds work budget','limit');}
function toText(value,state){
  if(scalar(value))return primitiveText(value);
  const exotic=value[Symbol.toPrimitive];
  if(exotic!==undefined&&exotic!==null){
    if(typeof exotic!=='function')throw new TypeError('Cannot convert object to primitive value');
    const result=Reflect.apply(exotic,value,['string']);
    if(!scalar(result))throw new TypeError('Cannot convert object to primitive value');
    return primitiveText(result);
  }
  for(const key of ['toString','valueOf']){
    const method=value[key];if(typeof method!=='function')continue;
    let result;
    if(method===Array.prototype.toString){
      const join=value.join;
      result=typeof join!=='function'?Object.prototype.toString.call(value):join===Array.prototype.join?boundedJoin(value,',',state):Reflect.apply(join,value,[]);
    }else if(method===Array.prototype.join)result=boundedJoin(value,',',state);
    else result=Reflect.apply(method,value,[]);
    if(scalar(result))return primitiveText(result);
  }
  throw new TypeError('Cannot convert object to primitive value');
}
function boundedJoin(value,separator,state=conversionState()){
  const object=Object(value),rawLength=+object.length;
  const length=Number.isNaN(rawLength)||rawLength<=0?0:Math.min(Math.floor(rawLength),Number.MAX_SAFE_INTEGER);
  if(length>1_000_000||state.depth>=128)conversionLimit();
  if(state.active.has(object)){if(cyclicJoinError!==undefined)throw cyclicJoinError;return '';}
  const parts=[];let units=0;state.active.add(object);state.depth++;
  try{
    for(let i=0;i<length;i++){
      if(++state.work>1_000_000)conversionLimit();
      if(i>0){units+=separator.length;if(units>16_000_000)conversionLimit();}
      const item=object[i],text=item===null||item===undefined?'':toText(item,state);
      units+=text.length;if(units>16_000_000)conversionLimit();parts.push(text);
    }
    return Array.prototype.join.call(parts,separator);
  }finally{state.active.delete(object);state.depth--;}
}


// Private fixed-signature imports used by the operation-local Rust read store.
// Scalar words stay below 2^39; no JS Array or public graph copy is constructed.
const READ_BASE=0x100000000,READ_FLAGS=0x1000000000;
function readAtom(context,value){
  const owner=context.owner;
  if(value===undefined)return 0;if(value===null)return READ_BASE;
  switch(typeof value){
    case 'boolean':return (value?3:2)*READ_BASE;
    case 'number':context.numberValue=value;context.numberPending=true;return 4*READ_BASE;
    case 'string':return 5*READ_BASE+owner.observedString(value);
    case 'function':{const id=owner.registration.functionIds.get(value);if(id!==undefined)return 7*READ_BASE+id;break;}
    case 'object':{
      const prototype=Object.getPrototypeOf(value);
      if(!(Array.isArray(value)||value instanceof Map||value instanceof Set||value instanceof Date||errorKind(value)||prototype===Object.prototype||owner.registration.prototypeIds.has(prototype)))break;
      return 6*READ_BASE+(owner.temporary?.objectsIds.get(value)??owner.ids.get(value)??owner.temporaryId('objects',value));
    }
  }
  return 8*READ_BASE+(owner.temporary?.hostsIds.get(value)??owner.hostIds.get(value)??owner.temporaryId('hosts',value));
}
function readOwn(context,id,key){
  const owner=context.owner;owner.assertLive();
  const descriptor=Object.getOwnPropertyDescriptor(owner.object(id),key);
  if(!descriptor)return -1;
  if(!Object.hasOwn(descriptor,'value'))accessorError();
  const flags=(descriptor.enumerable?1:0)|(descriptor.writable?2:0)|(descriptor.configurable?4:0);
  // A nested operation owns another context. Publish numeric slot only once
  // descriptor/classification work has returned; Rust consumes it immediately.
  return readAtom(context,descriptor.value)+flags*READ_FLAGS;
}
export function hostErReadOwn(context,id,key){return readOwn(context,id,key);}
export function hostErReadOwnCount(context,id){
  const owner=context.owner;owner.assertLive();
  // Retain enumeration and empty-key classification, including their errors
  // and ID/string side effects. Only the count crosses the admission ABI.
  return owner.ownInfo(id,'')[0];
}
export function hostErReadIndex(context,id,index){return readOwn(context,id,index);}
export function hostErReadNumberBits(context){
  context.owner.assertLive();
  if(!context.numberPending)throw new TypeError('Missing host numeric read result');
  bits.setFloat64(0,context.numberValue);const value=bits.getBigUint64(0);
  context.numberPending=false;context.numberValue=undefined;return value;
}
export function hostErReadKind(context,id){
  const owner=context.owner;owner.assertLive();
  const value=owner.object(id),prototype=Object.getPrototypeOf(value);
  if(Array.isArray(value))return READ_BASE+value.length;
  if(value instanceof Map)return 2*READ_BASE+Object.getOwnPropertyDescriptor(Map.prototype,'size').get.call(value);
  if(value instanceof Set)return 3*READ_BASE+Object.getOwnPropertyDescriptor(Set.prototype,'size').get.call(value);
  if(value instanceof Date){numberBits(Date.prototype.getTime.call(value));return 4*READ_BASE;}
  const prototypeId=owner.registration.prototypeIds.get(prototype);
  return prototypeId===undefined?0:5*READ_BASE+prototypeId;
}
export function hostErReadUnits(context,id){
  const owner=context.owner;owner.assertLive();
  const text=owner.string(id),units=new Uint16Array(text.length);
  for(let n=0;n<text.length;n++)units[n]=text.charCodeAt(n);
  return units;
}


// Map keys keep intrinsic SameValueZero/identity semantics; no string coercion.
function readKey(owner,tag,payload,numeric){
  switch(tag){
    case 0:return undefined;case 1:return null;case 2:return false;case 3:return true;
    case 4:return numeric;case 5:return owner.string(payload);case 6:return owner.object(payload);
    case 7:return owner.registration.functions.get(payload);
    case 8:return payload>=TEMP?owner.temporary.hosts[payload-TEMP]:owner.hostValues[payload];
    default:throw new TypeError('Unknown public atom kind');
  }
}
export function hostErReadMapText(context,id,key){
  const owner=context.owner;owner.assertLive();
  return readAtom(context,Map.prototype.get.call(owner.object(id),key));
}
export function hostErReadMapValue(context,id,tag,payload,numeric){
  const owner=context.owner;owner.assertLive();
  const key=readKey(owner,tag,payload,numeric);
  return readAtom(context,Map.prototype.get.call(owner.object(id),key));
}
export function hostErReadMapIterator(context,id){
  const owner=context.owner;owner.assertLive();
  return Map.prototype.entries.call(owner.object(id));
}
function writeMapAtom(context,view,offset,value){
  if(typeof value==='number'){
    view.setUint8(offset,4);view.setFloat64(offset+1,value,true);
  }else{
    const word=readAtom(context,value),tag=Math.floor(word/READ_BASE);
    view.setUint8(offset,tag);view.setUint32(offset+1,word-tag*READ_BASE,true);
  }
}
export function hostErReadMapChunk(context,cursor,maximum){
  context.owner.assertLive();
  if(!Number.isInteger(maximum)||maximum<1||maximum>128)throw new TypeError('Invalid host map chunk bound');
  // Header: u32 count, u8 done. Each pair: two tag/u64 atoms (18 bytes).
  // Buffer never exceeds 2309 bytes and dies after Rust copies this chunk.
  const bytes=new Uint8Array(5+18*maximum),view=new DataView(bytes.buffer);
  let count=0,done=false,offset=5;
  while(count<maximum){
    const item=cursor.next();
    if(item.done){done=true;break;}
    // Preserve existing nextMap order: classify key, then value, then next.
    writeMapAtom(context,view,offset,item.value[0]);offset+=9;
    writeMapAtom(context,view,offset,item.value[1]);offset+=9;
    count++;
  }
  view.setUint32(0,count,true);view.setUint8(4,done?1:0);
  return bytes.subarray(0,offset);
}

// Deliberately no graph, nodes, decoded properties or native model owner.
export class HostErOwner {
  constructor(registration,registry,root,invoke) {
    this.registration=registration;this.registry=registry;this.root=root;
    this.objects=[];this.ids=new WeakMap();this.strings=[];this.firstStrings=new Map();this.lastStrings=new Map();
    this.hostValues=[];this.hostIds=new Map();this.units=0;this.references=0;this.nodeBytes=[];this.nodeRefs=[];
    this.disposed=false;this.phase='constructing';this.epoch=0n;this.resourceDepth=0;this.methodDepth=0;this.depth=0;
    this.invoke=invoke;this.temporary=null;this.constructionNode=null;this.constructionString=null;this.constructionBytes=0;this.constructionNeedsAudit=false;
  }
  assertLive(){if(this.disposed)throw new TypeError('Public model document is disposed');}
  host(value){
    let id=this.hostIds.get(value);
    if(id===undefined){id=this.hostValues.length;if(id>=1_000_000)throw new RangeError('Host reference budget exceeded');this.hostValues.push(value);this.hostIds.set(value,id);}
    return [8,id];
  }
  intern(text,first=false){
    const known=(first?this.firstStrings:this.lastStrings).get(text);if(known!==undefined)return known;
    const id=this.strings.length;if(id>=1_000_000||this.units+text.length>16_000_000)graphBudget();
    this.units+=text.length;this.strings.push(text);this.firstStrings.set(text,id);this.lastStrings.set(text,id);return id;
  }
  temporaryId(kind,value){
    const t=this.temporary;if(!t)throw new TypeError('Host value observed outside operation');
    const map=t[kind+'Ids'],values=t[kind];let id=map.get(value);
    if(id!==undefined)return id;
    if(values.length>=1_000_000)graphBudget();
    if(kind==='strings'){t.units+=value.length;if(t.units>16_000_000)graphBudget();}
    id=TEMP+values.length;values.push(value);map.set(value,id);return id;
  }
  observedString(value){return this.temporary?.stringsIds.get(value)??this.lastStrings.get(value)??this.temporaryId('strings',value);}
  object(id){return id>=TEMP?this.temporary.objects[id-TEMP]:this.objects[id];}
  string(id){return id>=TEMP?this.temporary.strings[id-TEMP]:this.strings[id];}
  atom(value,admit=false){
    if(value===undefined)return [0];if(value===null)return [1];
    switch(typeof value){
      case 'boolean':return [value?3:2];
      case 'number':return [4,numberBits(value)];
      case 'string':return [5,admit?this.intern(value):this.observedString(value)];
      case 'function':{const id=this.registration.functionIds.get(value);if(id!==undefined)return [7,id];break;}
      case 'object':{
        const prototype=Object.getPrototypeOf(value);
        if(!(Array.isArray(value)||value instanceof Map||value instanceof Set||value instanceof Date||errorKind(value)||prototype===Object.prototype||this.registration.prototypeIds.has(prototype)))break;
        let id=this.ids.get(value);
        if(admit){if(id===undefined){id=this.objects.length;if(id>=1_000_000)graphBudget();this.objects.push(value);this.ids.set(value,id);}return [6,id];}
        return [6,this.temporary?.objectsIds.get(value)??id??this.temporaryId('objects',value)];
      }
    }
    if(admit)return this.host(value);
    return [8,this.temporary?.hostsIds.get(value)??this.hostIds.get(value)??this.temporaryId('hosts',value)];
  }
  value(atom){
    switch(atom[0]){
      case 0:return undefined;case 1:return null;case 2:return false;case 3:return true;case 4:return numberValue(atom[1]);
      case 5:return this.string(atom[1]);case 6:return this.object(atom[1]);case 7:return this.registration.functions.get(atom[1]);case 8:return atom[1]>=TEMP?this.temporary.hosts[atom[1]-TEMP]:this.hostValues[atom[1]];
      default:throw new TypeError('Unknown public atom kind');
    }
  }
  prototypeAudit(){
    for(const baseline of this.registration.prototypeBaseline){
      const descriptors=Object.getOwnPropertyDescriptors(baseline.value),original=baseline.descriptors;
      if(Object.getPrototypeOf(baseline.value)!==baseline.parent||Reflect.ownKeys(descriptors).length!==Reflect.ownKeys(original).length||Reflect.ownKeys(original).some(key=>{
        const a=original[key],b=descriptors[key];return !b||a.value!==b.value||a.get!==b.get||a.set!==b.set||a.enumerable!==b.enumerable||a.writable!==b.writable||a.configurable!==b.configurable;
      }))throw new TypeError('Prototype edits require an explicit native prototype mutation codec');
    }
  }
  // Same discovery order as ModelBridge.snapshot; accumulates counts, not values.
  audit(values=[],entry=true){
    this.assertLive();if(entry)this.prototypeAudit();
    const pending=[],seen=new Set(),accounting=[];let references=0,nodeBytes=0;
    const visit=value=>{const atom=this.atom(value,true);if(atom[0]===6&&!seen.has(atom[1])){seen.add(atom[1]);pending.push(atom[1]);}return atom;};
    const root=visit(this.root),args=values.map(visit);
    for(const value of this.objects)visit(value);
    while(pending.length){
      const id=pending.pop(),value=this.objects[id],startReferences=references;let count=0,propertyBytes=0;
      for(const key of Object.getOwnPropertyNames(value)){
        const descriptor=Object.getOwnPropertyDescriptor(value,key);
        if(Array.isArray(value)&&key==='length'&&descriptor.writable)continue;
        if(!Object.hasOwn(descriptor,'value')){if(errorKind(value)&&diagnostics.has(key))continue;accessorError();}
        if(++references>8_000_000)graphBudget();
        this.intern(key);propertyBytes+=5+atomBytes(visit(descriptor.value));count++;
      }
      let bytes=1+4+propertyBytes;
      if(value instanceof Map){bytes+=4;for(const [key,item]of Map.prototype.entries.call(value)){references+=2;if(references>8_000_000)graphBudget();bytes+=atomBytes(visit(key))+atomBytes(visit(item));}}
      else if(value instanceof Set){bytes+=4;for(const item of Set.prototype.values.call(value)){if(++references>8_000_000)graphBudget();bytes+=atomBytes(visit(item));}}
      else if(value instanceof Date)bytes+=8;
      else if(errorKind(value))bytes+=1;
      else if(Array.isArray(value))bytes+=4;
      else if(this.registration.prototypeIds.has(Object.getPrototypeOf(value)))bytes+=4;
      nodeBytes+=bytes;accounting.push(id,bytes,references-startReferences);
    }
    // Unclassifiable retained objects keep their last admitted native-node
    // storage cost. Their current value is a host reference, as in snapshot().
    for(let id=0;id<this.objects.length;id++)if(!seen.has(id)){nodeBytes+=this.nodeBytes[id]??0;references+=this.nodeRefs[id]??0;}
    for(let n=0;n<accounting.length;n+=3){this.nodeBytes[accounting[n]]=accounting[n+1];this.nodeRefs[accounting[n]]=accounting[n+2];}
    this.references=references;
    const graphBytes=8+4+8+4+atomBytes(root)+4+this.strings.length*4+this.units*2+4+nodeBytes;
    return {root,args,graphBytes,nodes:this.objects.length,strings:this.strings.length,units:this.units,references};
  }
  frameAdmission(audit,metadataBytes){if(audit.graphBytes+metadataBytes>64*1024*1024||audit.references>8_000_000)frameBudget();}
  begin(kind,values=[]){
    this.assertLive();
    const nested=this.phase==='running'&&this.resourceDepth>0;
    if(this.phase!=='idle'&&this.phase!=='needsAudit'&&!nested)throw new TypeError('Public model document is already in use');
    if(this.depth>=128)throw new PeleError('Host operation nesting exceeds work budget','limit');
    const previous=this.phase;this.phase='entryAudit';
    try{const audit=this.audit(values);this.frameAdmission(audit,0);if(this.epoch===0xffffffffffffffffn)throw new RangeError('Public model revision exhausted');this.epoch++;if(this.depth===0)this.temporary={objects:[],objectsIds:new WeakMap(),strings:[],stringsIds:new Map(),hosts:[],hostsIds:new Map(),units:0};this.depth++;this.phase='running';if(kind==='method')this.methodDepth++;return{audit,previous,nested,kind,finishedAudit:false};}
    catch(error){this.phase=nested?previous:'needsAudit';throw error;}
  }
  structure(token){this.phase='finalAudit';try{token.audit=this.audit([],false);token.finishedAudit=true;if(token.audit.references>8_000_000)throw new PeleError('Public graph exceeds work budget','limit');return token.audit;}catch(error){token.auditError=error;token.auditFailed=true;throw error;}}
  finish(token,metadataBytes=0){
    this.phase='finalAudit';
    try{if(token.auditFailed)throw token.auditError;const audit=token.finishedAudit?token.audit:this.audit([],false);this.frameAdmission(audit,metadataBytes,true);this.phase=token.nested?token.previous:'idle';return audit;}
    catch(error){this.phase=token.nested?token.previous:'needsAudit';throw error;}
    finally{this.depth--;if(token.kind==='method')this.methodDepth--;if(this.depth===0)this.temporary=null;}
  }
  resource(run){this.resourceDepth++;try{return run();}finally{this.resourceDepth--;}}
  dispose(){
    if(this.phase!=='idle'&&this.phase!=='needsAudit')throw new TypeError('Public model document is already in use');
    this.disposed=true;this.phase='disposed';this.root=undefined;this.objects=null;this.ids=null;this.strings=null;
    this.nodeBytes=null;this.nodeRefs=null;this.firstStrings=null;this.lastStrings=null;this.temporary=null;this.hostValues=null;this.hostIds=null;this.invoke=undefined;
  }
  ownInfo(id,key){
    const value=this.object(id);let count=0;
    for(const name of Object.getOwnPropertyNames(value)){
      const descriptor=Object.getOwnPropertyDescriptor(value,name);
      if(Array.isArray(value)&&name==='length'&&descriptor.writable)continue;
      if(!Object.hasOwn(descriptor,'value')){if(errorKind(value)&&diagnostics.has(name))continue;accessorError();}count++;
    }
    const d=Object.getOwnPropertyDescriptor(value,key);
    if(!d||(Array.isArray(value)&&key==='length'&&d.writable))return[count,false,0,[0]];
    if(!Object.hasOwn(d,'value'))accessorError();
    return[count,true,(d.enumerable?1:0)|(d.writable?2:0)|(d.configurable?4:0),this.atom(d.value)];
  }
  allocate(kind){
    if(this.objects.length>=1_000_000)graphBudget();
    let value;if(kind===1||kind==='array')value=[];else if(kind===2||kind==='map')value=new Map();else if(kind===3||kind==='set')value=new Set();else if(kind===4||kind==='er')value=Object.create(this.registration.prototypes.get(1));else value={};
    const id=this.objects.length;this.objects.push(value);this.ids.set(value,id);this.registry.owners.set(value,this);return id;
  }
  // Bounded constructor events. State spans batches only while unpublished:
  // one final object under construction, or fragments of one large string.
  publishBatch(bytes){
    const bad=()=>{throw new TypeError('Invalid host constructor batch');};
    if(!(bytes instanceof Uint8Array)||bytes.byteLength<2||bytes.byteLength>65536)bad();
    const view=new DataView(bytes.buffer,bytes.byteOffset,bytes.byteLength);
    const count=view.getUint16(0,true);if(count<1||count>128)bad();
    let offset=2;
    const need=n=>{if(n>bytes.byteLength-offset)bad();};
    const u8=()=>{need(1);return view.getUint8(offset++);};
    const u32=()=>{need(4);const value=view.getUint32(offset,true);offset+=4;return value;};
    const scratch=[];
    const text=units=>{
      need(units*2);const parts=[];
      for(let left=units;left>0;){
        const size=Math.min(left,8192);scratch.length=size;
        for(let i=0;i<size;i++){scratch[i]=view.getUint16(offset,true);offset+=2;}
        parts.push(String.fromCharCode(...scratch));left-=size;
      }
      return parts.join('');
    };
    const stringRoom=(id,units)=>{
      if(id!==this.strings.length)bad();
      if(id>=1_000_000||units>16_000_000-this.units)graphBudget();
    };
    const addString=(id,value)=>{if(this.intern(value,true)!==id)bad();};
    const valueBytes=value=>value===null||value===undefined||typeof value==='boolean'?1:typeof value==='number'?9:5;
    const atom=()=>{
      const tag=u8();
      switch(tag){
        case 0:return undefined;case 1:return null;case 2:return false;case 3:return true;
        case 4:{need(8);bits.setBigUint64(0,view.getBigUint64(offset,true));offset+=8;return bits.getFloat64(0);}
        case 5:return this.string(u32());case 6:return this.object(u32());
        case 7:return this.registration.functions.get(u32());
        case 8:{const id=u32();return id>=TEMP?this.temporary.hosts[id-TEMP]:this.hostValues[id];}
        default:bad();
      }
    };
    // Inline and fragmented nodes share publication and accounting rules.
    const beginNode=(id,kind,length)=>{
      if(this.constructionNode||this.constructionString||kind>4||id!==this.objects.length)bad();
      if(this.allocate(kind)!==id)bad();
      this.constructionNode={id,object:this.object(id),kind,length,properties:false,assignFresh:kind===0||kind===1,bytes:kind===0?5:9,refs:0};
    };
    const mapEntry=()=>{
      const node=this.constructionNode;if(!node||node.kind!==2||node.properties||this.constructionString)bad();
      const key=atom(),value=atom(),had=Map.prototype.has.call(node.object,key);
      if(had)node.bytes+=valueBytes(value)-valueBytes(Map.prototype.get.call(node.object,key));
      else{node.refs+=2;node.bytes+=valueBytes(key)+valueBytes(value);}
      Map.prototype.set.call(node.object,key,value);
    };
    const setValue=()=>{
      const node=this.constructionNode;if(!node||node.kind!==3||node.properties||this.constructionString)bad();
      const value=atom();if(!Set.prototype.has.call(node.object,value)){node.refs++;node.bytes+=valueBytes(value);}
      Set.prototype.add.call(node.object,value);
    };
    const property=()=>{
      const node=this.constructionNode;if(!node||this.constructionString)bad();
      const key=u32(),flags=u8(),name=this.string(key),value=atom();node.properties=true;
      if(flags!==7)node.assignFresh=false;
      const previous=Object.getOwnPropertyDescriptor(node.object,name),length=node.kind===1&&name==='length';
      // Keep raw NaN publication on the descriptor path. Once a nondefault
      // descriptor appears, later duplicate keys must use strict redefinition.
      if(node.assignFresh&&name!=='__proto__'&&name!=='length'&&!(typeof value==='number'&&Number.isNaN(value)))node.object[name]=value;
      else if(!Reflect.defineProperty(node.object,name,{value,enumerable:!!(flags&1),writable:!!(flags&2),configurable:!!(flags&4)}))return false;
      if(previous&&!(length&&previous.writable)){node.refs--;node.bytes-=5+valueBytes(previous.value);}
      if(!(length&&(flags&2))){node.refs++;node.bytes+=5+valueBytes(value);}
      if(length)this.constructionNeedsAudit=true;
      return true;
    };
    const endNode=()=>{
      const node=this.constructionNode;if(!node||this.constructionString)bad();
      if(node.kind===1){
        if(node.object.length>node.length)this.constructionNeedsAudit=true;
        if(!Reflect.set(node.object,'length',node.length))return false;
      }
      this.nodeBytes[node.id]=node.bytes;this.nodeRefs[node.id]=node.refs;
      this.constructionBytes+=node.bytes;this.references+=node.refs;
      this.constructionNode=null;return true;
    };
    for(let event=0;event<count;event++){
      const tag=u8();
      switch(tag){
        case 0:{
          if(this.constructionNode||this.constructionString)bad();
          const id=u32(),units=u32();stringRoom(id,units);
          addString(id,text(units));break;
        }
        case 1:{
          if(this.constructionNode||this.constructionString)bad();
          const id=u32(),units=u32();stringRoom(id,units);
          this.constructionString={id,units,read:0,parts:[]};break;
        }
        case 2:{
          const pending=this.constructionString;if(!pending||this.constructionNode)bad();
          const units=u32();if(units===0||units>pending.units-pending.read)bad();
          pending.parts.push(text(units));pending.read+=units;break;
        }
        case 3:{
          const pending=this.constructionString;
          if(!pending||pending.read!==pending.units||this.constructionNode)bad();
          addString(pending.id,pending.parts.join(''));this.constructionString=null;break;
        }
        case 4:{
          if(this.constructionNode||this.constructionString)bad();
          const id=u32(),kind=u8(),length=u32();beginNode(id,kind,length);break;
        }
        case 5:mapEntry();break;
        case 6:setValue();break;
        case 7:if(!property())return -1;break;
        case 8:if(!endNode())return -2;break;
        case 9:{
          if(this.constructionNode||this.constructionString)bad();
          const id=u32(),kind=u8(),length=u32(),payloadCount=u32(),propertyCount=u32();
          if(kind>4||(kind!==2&&kind!==3&&payloadCount!==0))bad();
          // Reject impossible counts before allocation or iteration. Every map
          // pair uses at least two bytes; every property uses at least six.
          need(payloadCount*(kind===2?2:1)+propertyCount*6);
          beginNode(id,kind,length);
          if(kind===2)for(let i=0;i<payloadCount;i++)mapEntry();
          else if(kind===3)for(let i=0;i<payloadCount;i++)setValue();
          for(let i=0;i<propertyCount;i++)if(!property())return -1;
          if(!endNode())return -2;break;
        }
        default:bad();
      }
    }
    if(offset!==bytes.byteLength)bad();
    return 0;
  }
  disposeConstruction(){this.constructionNode=null;this.constructionString=null;this.constructionBytes=0;this.constructionNeedsAudit=false;this.objects=[];this.ids=new WeakMap();this.strings=[];this.firstStrings=new Map();this.lastStrings=new Map();this.hostValues=[];this.hostIds=new Map();this.root=undefined;this.units=0;this.references=0;this.nodeBytes=[];this.nodeRefs=[];}
  // Generic adapter ABI; no ER method/field decisions occur in this switch.
  primitive(op,args){
    this.assertLive();
    switch(op){
      case 'context':return {owner:this,numberValue:undefined,numberPending:false};
      case 'root':return this.atom(this.root);
      case 'counts':return[this.objects.length,this.strings.length,this.units,this.references];
      case 'own':return this.ownInfo(args[0],args[1]);
      case 'peekOwn':{const d=Object.getOwnPropertyDescriptor(this.object(args[0]),args[1]);if(!d)return[false,0,[0]];if(!Object.hasOwn(d,'value'))accessorError();return[true,(d.enumerable?1:0)|(d.writable?2:0)|(d.configurable?4:0),this.atom(d.value)];}
      case 'kind':{
        const value=this.object(args[0]),proto=Object.getPrototypeOf(value);
        if(Array.isArray(value))return[1,value.length];if(value instanceof Map)return[2,Object.getOwnPropertyDescriptor(Map.prototype,'size').get.call(value)];
        if(value instanceof Set)return[3,Object.getOwnPropertyDescriptor(Set.prototype,'size').get.call(value)];if(value instanceof Date)return[4,numberBits(Date.prototype.getTime.call(value))];
        const id=this.registration.prototypeIds.get(proto);return id===undefined?[0,0]:[5,id];
      }
      case 'string':return this.string(args[0]);
      case 'units':{const text=this.string(args[0]),out=new Uint16Array(text.length);for(let i=0;i<text.length;i++)out[i]=text.charCodeAt(i);return out;}
      case 'intern':return this.intern(decodeUnits(args[0]),true);
      case 'findString':return this.firstStrings.get(decodeUnits(args[0]));
      case 'internText':return this.intern(args[0],true);
      case 'discard':this.disposeConstruction();return undefined;
      case 'setRoot':this.root=this.value(args[0]);return undefined;
      case 'constructed':{
        if(this.constructionNode||this.constructionString)throw new TypeError('Incomplete host constructor batch');
        // Native publisher already owns canonical IDs and all key strings. No
        // caller callback occurred, so aggregates replace the redundant walk.
        // Unusual length rewrites retain the original exact audit path.
        const admission=this.constructionNeedsAudit?this.audit([],false):{
          graphBytes:32+atomBytes(this.atom(this.root,true))+this.strings.length*4+this.units*2+this.constructionBytes,
          references:this.references,
        };
        if(admission.references>8_000_000)graphBudget();
        this.frameAdmission(admission,13);this.constructionBytes=0;return undefined;
      }
      case 'defineKey':return Reflect.defineProperty(this.object(args[0]),this.string(args[1]),{value:this.value(args[2]),enumerable:!!(args[3]&1),writable:!!(args[3]&2),configurable:!!(args[3]&4)});
      case 'allocate':return this.allocate(args[0]);
      case 'publishBatch':return this.publishBatch(args[0]);
      case 'define':return Reflect.defineProperty(this.object(args[0]),args[1],{value:this.value(args[2]),enumerable:!!(args[3]&1),writable:!!(args[3]&2),configurable:!!(args[3]&4)});
      case 'length':return Reflect.set(this.object(args[0]),'length',args[1]);
      case 'typeError':throw new TypeError(args[0]);
      // Ask the host engine to produce its own primitive-operation error. This
      // preserves browser-specific TypeError spelling, including null access.
      case 'stringFailure':{
        const e=this.value(args[0]);
        switch(args[1]){
          case 'includes':return cellStringFailure(e);
          case 'label':return labelStringFailure(e);
          case 'accessibility':return accessibilityStringFailure(e);
          case 'split':return e.split(" ");
          case 'toLowerCase':return e.toLowerCase();
          case 'indexOf':return e.indexOf(":");
          case 'replace':return e.replace("","");
          default:return e.trim();
        }
      }
      case 'joinUnits':{const text=this.resource(()=>boundedJoin(this.value(args[0]),args[1]));if(text.length>16_000_000)graphBudget();const units=new Uint16Array(text.length);for(let i=0;i<text.length;i++)units[i]=text.charCodeAt(i);return units;}
      case 'coerceUnits':{const text=this.resource(()=>toText(this.value(args[0]),conversionState()));if(text.length>16_000_000)graphBudget();const units=new Uint16Array(text.length);for(let i=0;i<text.length;i++)units[i]=text.charCodeAt(i);return units;}
      case 'mapGetText':return this.atom(Map.prototype.get.call(this.object(args[0]),args[1]));
      case 'mapGet':return this.atom(Map.prototype.get.call(this.object(args[0]),this.value(args[1])));
      case 'mapHas':return Map.prototype.has.call(this.object(args[0]),this.value(args[1]));
      case 'mapSet':Map.prototype.set.call(this.object(args[0]),this.value(args[1]),this.value(args[2]));return undefined;
      case 'setHas':return Set.prototype.has.call(this.object(args[0]),this.value(args[1]));
      case 'setAdd':Set.prototype.add.call(this.object(args[0]),this.value(args[1]));return undefined;
      case 'mapIterator':return Map.prototype.entries.call(this.object(args[0]));
      case 'setIterator':return Set.prototype.values.call(this.object(args[0]));
      case 'nextMap':{const item=args[0].next();return item.done?[]:[this.atom(item.value[0]),this.atom(item.value[1])];}
      case 'nextSet':{const item=args[0].next();return item.done?[]:[this.atom(item.value)];}
      default:throw new TypeError('Unknown host storage operation');
    }
  }
}
