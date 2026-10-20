import {PeleError} from './errors.js';
// Host values only: diagram state transitions and rendering remain in Rust.
const errorConstructors={error:Error,pele:PeleError,type:TypeError,range:RangeError,syntax:SyntaxError,reference:ReferenceError,eval:EvalError,uri:URIError};
const errorDiagnostics=new Set(['stack','fileName','lineNumber','columnNumber','sourceURL']);
const errorKind=value=>{const proto=Object.getPrototypeOf(value);return Object.keys(errorConstructors).find(key=>proto===errorConstructors[key].prototype);};
const bits = new DataView(new ArrayBuffer(8));
function numberBits(value) {
  bits.setFloat64(0, value);
  return bits.getBigUint64(0).toString(16).padStart(16, '0');
}
export function numberValue(hex) {
  bits.setBigUint64(0, BigInt('0x' + hex));
  return bits.getFloat64(0);
}
function units(value) {
  const output = new Array(value.length);
  for (let i = 0; i < value.length; i++) output[i] = value.charCodeAt(i);
  return output;
}
export function stringValue(value) {
  let output = '';
  for (let i = 0; i < value.length; i += 8192) output += String.fromCharCode(...value.slice(i, i + 8192));
  return output;
}
// Flat comparisons avoid serializing every unchanged record during an opted-in
// live callback. The existing full snapshot remains the ordinary operation lane.
const sameAtom=(a,b)=>a?.kind===b?.kind&&a?.value===b?.value;
const sameList=(a,b,equal)=>a.length===b.length&&a.every((v,i)=>equal(v,b[i]));
const sameNode=(a,b)=>a&&a.kind===b.kind&&a.prototype===b.prototype&&a.errorType===b.errorType&&a.length===b.length&&a.milliseconds===b.milliseconds&&
  sameList(a.properties,b.properties,(x,y)=>x.key===y.key&&x.flags===y.flags&&sameAtom(x.value,y.value))&&
  (a.kind!=='map'||sameList(a.entries,b.entries,(x,y)=>sameAtom(x[0],y[0])&&sameAtom(x[1],y[1])))&&
  (a.kind!=='set'||sameList(a.values,b.values,sameAtom));

// Identity sentinel. Legacy traversal keeps direct nested accesses on this path.
// Other backends implement the scalar slot/capture interface documented privately.
export const NestedStorage = Object.freeze({});

function flatValue(bridge, storage, graph, slot) {
  const tag = storage.atomTag(graph, slot);
  const value = storage.atomPayload(graph, slot);
  switch (tag) {
    case 0: return undefined;
    case 1: return null;
    case 2: return false;
    case 3: return true;
    case 4: return value;
    case 5: return bridge.strings[value];
    case 6: return bridge.objects[value];
    case 7: return bridge.registration.functions.get(value);
    case 8: return bridge.hostValues[value];
    default: throw new TypeError('Unknown public atom kind');
  }
}

// One closure set per backend. The hot operations do not receive or select a
// backend; the nested/flat choice is an immutable factory binding. Whether an
// engine folds inner branches is an empirical optimization, not a guarantee.
export function createGraphTraversal(storage = NestedStorage) {
  const nested = storage === NestedStorage;

function reconcileGraph(bridge, graph, changed) {
  bridge.graph = graph;
  if (nested) {
    if(changed) {
      for(let i=bridge.strings.length;i<graph.strings.length;i++){const value=stringValue(graph.strings[i]);bridge.strings.push(value);bridge.stringIds.set(value,i);}
    } else {
      bridge.strings = graph.strings.map(stringValue);
      bridge.stringIds = new Map(bridge.strings.map((value, index) => [value, index]));
    }
  } else {
    if (!changed) { bridge.strings = []; bridge.stringIds = new Map(); }
    for (let i=bridge.strings.length;i<storage.stringCount(graph);i++) {
      const value=storage.stringValueAt(graph,i);
      bridge.strings.push(value);bridge.stringIds.set(value,i);
    }
  }
  const freshStart = bridge.objects.length;
  const nodeCount = nested ? graph.nodes.length : storage.nodeCount(graph);
  while (bridge.objects.length < nodeCount) {
    const id = bridge.objects.length, node = nested ? graph.nodes[id] : id;
    const kind = nested ? node.kind : storage.nodeKind(graph,id);
    let value;
    switch (kind) {
      case 'error': {
        const Constructor=errorConstructors[nested?node.errorType:storage.nodeAux(graph,id)];
        if(!Constructor)throw new TypeError('Unknown native Error brand');
        let message;
        if(nested)message=node.properties.some(p=>bridge.strings[p.key]==='message');
        else {
          message=false;
          for(let i=0;i<storage.propertyCount(graph,id);i++) {
            if(bridge.strings[storage.propertyKey(graph,storage.propertySlot(graph,id,i))]==='message'){message=true;break;}
          }
        }
        value=new Constructor(message?'':undefined);break;
      }
      case 'instance': value = Object.create(bridge.registration.prototypes.get(nested?node.prototype:storage.nodeAux(graph,id))); break;
      case 'object': value = {}; break;
      case 'array': value = new Array(nested?node.length:storage.nodeAux(graph,id)); break;
      case 'map': value = new Map(); break;
      case 'set': value = new Set(); break;
      case 'date': value = new Date(numberValue(nested?node.milliseconds:storage.nodeAux(graph,id))); break;
      default: throw new TypeError('Unsupported public object kind');
    }
    bridge.objects.push(value); bridge.ids.set(value, id); bridge.registry.owners.set(value, bridge);
  }
  for (const id of changed ?? (nested?graph.nodes.keys():storage.nodeIds(graph))) {
    const node = nested ? graph.nodes[id] : id;
    const kind = nested ? node.kind : storage.nodeKind(graph,id);
    const object = bridge.objects[id], fresh = id >= freshStart;
    if (!fresh || kind === 'error') {
      let keys;
      if(nested)keys=new Set(node.properties.map(property=>bridge.strings[property.key]));
      else {
        keys=new Set();
        for(let i=0;i<storage.propertyCount(graph,id);i++)keys.add(bridge.strings[storage.propertyKey(graph,storage.propertySlot(graph,id,i))]);
      }
      for (const key of Object.getOwnPropertyNames(object)) {
        if (kind === 'array' && key === 'length') continue;
        if (kind === 'error' && errorDiagnostics.has(key) && !keys.has(key)) continue;
        if (!keys.has(key) && !Reflect.deleteProperty(object, key)) throw new TypeError('Native graph removed a non-configurable property');
      }
    }
    if (kind === 'map') {
      if (!fresh) Map.prototype.clear.call(object);
      if(nested)for(const [key,value] of node.entries)Map.prototype.set.call(object,bridge.value(key),bridge.value(value));
      else for(let i=0;i<storage.itemCount(graph,id);i++)Map.prototype.set.call(object,flatValue(bridge,storage,graph,storage.itemAtomSlot(graph,id,2*i)),flatValue(bridge,storage,graph,storage.itemAtomSlot(graph,id,2*i+1)));
    }
    if (kind === 'set') {
      if (!fresh) Set.prototype.clear.call(object);
      if(nested)for(const value of node.values)Set.prototype.add.call(object,bridge.value(value));
      else for(let i=0;i<storage.itemCount(graph,id);i++)Set.prototype.add.call(object,flatValue(bridge,storage,graph,storage.itemAtomSlot(graph,id,i)));
    }
    if (kind === 'date' && !fresh) Date.prototype.setTime.call(object, numberValue(nested?node.milliseconds:storage.nodeAux(graph,id)));
    if (kind === 'array' && object.length !== (nested?node.length:storage.nodeAux(graph,id))) object.length=nested?node.length:storage.nodeAux(graph,id);
    let assignFresh = fresh && (kind === 'object' || kind === 'array');
    for(const property of nested?node.properties:storage.propertySlots(graph,id)) {
      const flags=nested?property.flags:storage.propertyFlags(graph,property);
      if(flags!==7)assignFresh=false;
      const key=bridge.strings[nested?property.key:storage.propertyKey(graph,property)];
      const atom=nested?property.value:storage.propertyAtomSlot(graph,property);
      const value=nested?bridge.value(atom):flatValue(bridge,storage,graph,atom);
      if(assignFresh&&key!=='__proto__'&&!(kind==='array'&&key==='length'))object[key]=value;
      else {
        const current=!fresh&&Object.getOwnPropertyDescriptor(object,key);
        const sameValue=current&&Object.hasOwn(current,'value')&&(Object.is(current.value,value)&&
          (typeof value!=='number'||!Number.isNaN(value)||numberBits(current.value)===(nested?property.value.value:(storage.atomTag(graph,atom)===4?storage.atomNumberBits(graph,atom):undefined))));
        if(!sameValue||current.enumerable!==!!((nested?property.flags:flags)&1)||current.writable!==!!((nested?property.flags:flags)&2)||current.configurable!==!!((nested?property.flags:flags)&4))Object.defineProperty(object,key,{
          value,enumerable:!!((nested?property.flags:flags)&1),writable:!!((nested?property.flags:flags)&2),configurable:!!((nested?property.flags:flags)&4)
        });
      }
    }
  }
  bridge.root=nested?bridge.value(graph.root):flatValue(bridge,storage,graph,storage.rootSlot(graph));
  return bridge.root;
}

function matchesGraph(bridge) {
  const graph=bridge.graph;
  if(bridge.objects.length!==(nested?graph.nodes.length:storage.nodeCount(graph))||bridge.strings.length!==(nested?graph.strings.length:storage.stringCount(graph)))return false;
  for (const baseline of bridge.registration.prototypeBaseline) {
    const descriptors=Object.getOwnPropertyDescriptors(baseline.value),original=baseline.descriptors;
    if(Object.getPrototypeOf(baseline.value)!==baseline.parent||Reflect.ownKeys(descriptors).length!==Reflect.ownKeys(original).length||Reflect.ownKeys(original).some(key=>{const a=original[key],b=descriptors[key];return !b||a.value!==b.value||a.get!==b.get||a.set!==b.set||a.enumerable!==b.enumerable||a.writable!==b.writable||a.configurable!==b.configurable;}))throw new TypeError('Prototype edits require an explicit native prototype mutation codec');
  }
  const matches=nested?(atom,value)=>{
    switch(atom.kind){
      case 'undefined':return value===undefined;
      case 'null':return value===null;
      case 'bool':return value===atom.value;
      case 'number':return typeof value==='number'&&numberBits(value)===atom.value;
      case 'string':return typeof value==='string'&&value===bridge.strings[atom.value]&&bridge.stringIds.get(value)===atom.value;
      case 'ref':return value===bridge.objects[atom.value]&&bridge.ids.get(value)===atom.value;
      case 'builtinFunction':return value===bridge.registration.functions.get(atom.value)&&bridge.registration.functionIds.get(value)===atom.value;
      default:return false;
    }
  }:(slot,value)=>{
    const tag=storage.atomTag(graph,slot),payload=storage.atomPayload(graph,slot);
    switch(tag){
      case 0:return value===undefined;
      case 1:return value===null;
      case 2:return value===false;
      case 3:return value===true;
      case 4:return typeof value==='number'&&numberBits(value)===storage.atomNumberBits(graph,slot);
      case 5:return typeof value==='string'&&value===bridge.strings[payload]&&bridge.stringIds.get(value)===payload;
      case 6:return value===bridge.objects[payload]&&bridge.ids.get(value)===payload;
      case 7:return value===bridge.registration.functions.get(payload)&&bridge.registration.functionIds.get(value)===payload;
      default:return false;
    }
  };
  if(!matches(nested?graph.root:storage.rootSlot(graph),bridge.root))return false;
  for(let id=0;id<(nested?graph.nodes.length:storage.nodeCount(graph));id++){
    const node=nested?graph.nodes[id]:id,object=bridge.objects[id],prototype=Object.getPrototypeOf(object);
    const kind=nested?node.kind:storage.nodeKind(graph,id);
    switch(kind){
      case 'object':if(prototype!==Object.prototype)return false;break;
      case 'instance':if(prototype!==bridge.registration.prototypes.get(nested?node.prototype:storage.nodeAux(graph,id)))return false;break;
      case 'array':if(prototype!==Array.prototype||object.length!==(nested?node.length:storage.nodeAux(graph,id)))return false;break;
      case 'map':if(prototype!==Map.prototype)return false;break;
      case 'set':if(prototype!==Set.prototype)return false;break;
      case 'date':if(prototype!==Date.prototype||numberBits(Date.prototype.getTime.call(object))!==(nested?node.milliseconds:storage.nodeAux(graph,id)))return false;break;
      case 'error':if(prototype!==errorConstructors[nested?node.errorType:storage.nodeAux(graph,id)].prototype)return false;break;
      default:return false;
    }
    let count=0;
    for(const key of Object.getOwnPropertyNames(object)){
      const descriptor=Object.getOwnPropertyDescriptor(object,key);
      if(kind==='array'&&key==='length'&&descriptor.writable)continue;
      if(!Object.hasOwn(descriptor,'value')){if(kind==='error'&&errorDiagnostics.has(key))continue;return false;}
      if(nested){
        const property=node.properties[count++];
        if(!property||key!==bridge.strings[property.key]||bridge.stringIds.get(key)!==property.key||descriptor.enumerable!==!!(property.flags&1)||descriptor.writable!==!!(property.flags&2)||descriptor.configurable!==!!(property.flags&4)||!matches(property.value,descriptor.value))return false;
      }else{
        if(count>=storage.propertyCount(graph,id))return false;
        const property=storage.propertySlot(graph,id,count++),keyId=storage.propertyKey(graph,property),flags=storage.propertyFlags(graph,property);
        if(key!==bridge.strings[keyId]||bridge.stringIds.get(key)!==keyId||descriptor.enumerable!==!!(flags&1)||descriptor.writable!==!!(flags&2)||descriptor.configurable!==!!(flags&4)||!matches(storage.propertyAtomSlot(graph,property),descriptor.value))return false;
      }
    }
    if(count!==(nested?node.properties.length:storage.propertyCount(graph,id)))return false;
    if(kind==='map'){
      if(Object.getOwnPropertyDescriptor(Map.prototype,'size').get.call(object)!==(nested?node.entries.length:storage.itemCount(graph,id)))return false;
      let i=0;
      for(const [key,value] of Map.prototype.entries.call(object)){
        if(nested){const pair=node.entries[i++];if(!pair||!matches(pair[0],key)||!matches(pair[1],value))return false;}
        else{const at=i++;if(!matches(storage.itemAtomSlot(graph,id,at*2),key)||!matches(storage.itemAtomSlot(graph,id,at*2+1),value))return false;}
      }
    }
    if(kind==='set'){
      if(Object.getOwnPropertyDescriptor(Set.prototype,'size').get.call(object)!==(nested?node.values.length:storage.itemCount(graph,id)))return false;
      let i=0;for(const value of Set.prototype.values.call(object))if(!matches(nested?node.values[i++]:storage.itemAtomSlot(graph,id,i++),value))return false;
    }
  }
  return true;
}

function snapshotGraph(bridge, values = [], delta = false) {
  for (const baseline of bridge.registration.prototypeBaseline) {
    const descriptors = Object.getOwnPropertyDescriptors(baseline.value), original = baseline.descriptors;
    if (Object.getPrototypeOf(baseline.value) !== baseline.parent || Reflect.ownKeys(descriptors).length !== Reflect.ownKeys(original).length || Reflect.ownKeys(original).some(key => {
      const a=original[key],b=descriptors[key];
      return !b || a.value!==b.value || a.get!==b.get || a.set!==b.set || a.enumerable!==b.enumerable || a.writable!==b.writable || a.configurable!==b.configurable;
    })) throw new TypeError('Prototype edits require an explicit native prototype mutation codec');
  }
  const baseline=bridge.graph, changes=nested?[]:null;
  const pending=[],seen=new Set(),nodes=nested?baseline.nodes.slice():null;
  const capture=nested?null:storage.beginCapture(baseline,delta);
  try {
  const store=nested?(id,node)=>{if(!delta||!sameNode(nodes[id],node)){nodes[id]=node;changes.push([id,node]);}}:null;
  let codeUnits=bridge.strings.reduce((n,s)=>n+s.length,0),propertiesCount=0;
  const budget=()=>{throw new RangeError('Public model graph exceeds host exchange budget');};
  const string=value=>{
    if(bridge.stringIds.has(value))return bridge.stringIds.get(value);
    const id=bridge.strings.length;
    codeUnits+=value.length;
    if(id>=1_000_000||codeUnits>16_000_000)budget();
    bridge.strings.push(value);bridge.stringIds.set(value,id);
    return id;
  };
  let atomTag=0;
  const host=nested?value=>bridge.hostReference(value):value=>{atomTag=8;return bridge.hostReferenceId(value);};
  const atom=value=>{
    if(value===undefined){if(nested)return{kind:'undefined'};atomTag=0;return 0;}
    if(value===null){if(nested)return{kind:'null'};atomTag=1;return 0;}
    switch(typeof value){
      case 'boolean':if(nested)return{kind:'bool',value};atomTag=value?3:2;return 0;
      case 'number':if(nested)return{kind:'number',value:numberBits(value)};atomTag=4;return value;
      case 'string':{const id=string(value);if(nested)return{kind:'string',value:id};atomTag=5;return id;}
      case 'function':{
        const id=bridge.registration.functionIds.get(value);
        if(id===undefined)return host(value);
        if(nested)return{kind:'builtinFunction',value:id};atomTag=7;return id;
      }
      case 'object':{
        const prototype=Object.getPrototypeOf(value);
        if(!(Array.isArray(value)||value instanceof Map||value instanceof Set||value instanceof Date||errorKind(value)||prototype===Object.prototype||bridge.registration.prototypeIds.has(prototype)))return host(value);
        let id=bridge.ids.get(value);
        if(id===undefined){id=bridge.objects.length;if(id>=1_000_000)budget();bridge.objects.push(value);bridge.ids.set(value,id);}
        if(!seen.has(id)){seen.add(id);pending.push([id,value]);}
        if(nested)return{kind:'ref',value:id};atomTag=6;return id;
      }
      default:return host(value);
    }
  };
  const root=atom(bridge.root);
  let argumentsValue;
  if(nested)argumentsValue=values.map(atom);
  else {
    capture.root(atomTag,root);
    const length=values.length;
    capture.beginArguments(length);
    for(let i=0;i<length;i++)if(i in values){const payload=atom(values[i]);capture.argument(i,atomTag,payload);}
  }
  for(const object of bridge.objects)atom(object);
  while(pending.length){
    const [id,value]=pending.pop(),properties=nested?[]:null;
    if(!nested)capture.beginNode(id);
    for(const key of Object.getOwnPropertyNames(value)){
      const descriptor=Object.getOwnPropertyDescriptor(value,key);
      if(Array.isArray(value)&&key==='length'){
        if(descriptor.writable)continue;
      }
      if(!Object.hasOwn(descriptor,'value')){
        if(errorKind(value)&&errorDiagnostics.has(key))continue;
        throw new TypeError('Accessor properties require an explicit public model codec');
      }
      if(++propertiesCount>8_000_000)budget();
      const keyId=string(key),payload=atom(descriptor.value);
      const flags=(descriptor.enumerable?1:0)|(descriptor.writable?2:0)|(descriptor.configurable?4:0);
      if(nested)properties.push({key:keyId,value:payload,flags});
      else capture.property(keyId,flags,atomTag,payload);
    }
    if(value instanceof Map){
      const entries=nested?[]:null;
      if(!nested)capture.beginItems('map');
      for(const [key,item] of Map.prototype.entries.call(value)){
        propertiesCount+=2;if(propertiesCount>8_000_000)budget();
        if(nested)entries.push([atom(key),atom(item)]);
        else {const keyPayload=atom(key),keyTag=atomTag;const valuePayload=atom(item);capture.mapEntry(keyTag,keyPayload,atomTag,valuePayload);}
      }
      if(nested)store(id,{kind:'map',entries,properties});
      else capture.finishNode(id,'map',undefined);
    }else if(value instanceof Set){
      const items=nested?[]:null;
      if(!nested)capture.beginItems('set');
      for(const item of Set.prototype.values.call(value)){
        if(++propertiesCount>8_000_000)budget();
        if(nested)items.push(atom(item));
        else {const payload=atom(item);capture.setItem(atomTag,payload);}
      }
      if(nested)store(id,{kind:'set',values:items,properties});
      else capture.finishNode(id,'set',undefined);
    }else if(value instanceof Date){
      const milliseconds=numberBits(Date.prototype.getTime.call(value));
      if(nested)store(id,{kind:'date',milliseconds,properties});
      else capture.finishNode(id,'date',milliseconds);
    }else if(errorKind(value)){
      const errorType=errorKind(value);
      if(nested)store(id,{kind:'error',errorType,properties});
      else capture.finishNode(id,'error',errorType);
    }else if(Array.isArray(value)){
      const length=value.length;
      if(nested)store(id,{kind:'array',length,properties});
      else capture.finishNode(id,'array',length);
    }else{
      const prototype=bridge.registration.prototypeIds.get(Object.getPrototypeOf(value));
      if(nested)store(id,prototype===undefined?{kind:'object',properties}:{kind:'instance',prototype,properties});
      else capture.finishNode(id,prototype===undefined?'object':'instance',prototype);
    }
  }
  if(nested){
    const appended=bridge.strings.slice(baseline.strings.length).map(units);
    bridge.graph={version:1,root,strings:delta?baseline.strings.concat(appended):bridge.strings.map(units),nodes};
    return{graph:bridge.graph,values:argumentsValue,...(delta?{delta:{stringsStart:baseline.strings.length,strings:appended,nodesStart:baseline.nodes.length,nodes:changes.sort((a,b)=>a[0]-b[0])}}:{})};
  }
  const result=capture.finish(bridge.strings);
  bridge.graph=result.graph;
  return result;
  } catch(error) {
    try { capture?.abort?.(bridge.strings); } catch {}
    throw error;
  }
}

  return Object.freeze({reconcileGraph, matchesGraph, snapshotGraph});
}

const traversalCache = new WeakMap();
export function getGraphTraversal(storage = NestedStorage) {
  let traversal = traversalCache.get(storage);
  if (traversal === undefined) {
    traversal = createGraphTraversal(storage);
    traversalCache.set(storage, traversal);
  }
  return traversal;
}

// ModelBridge imports these direct closures. No cache lookup or backend dispatch
// occurs in its ordinary reconcile/snapshot/match operation path.
export const {
  reconcileGraph: reconcileNestedGraph,
  matchesGraph: matchesNestedGraph,
  snapshotGraph: snapshotNestedGraph,
} = getGraphTraversal(NestedStorage);

// Compatibility entry points for storage consumers. New owners should retain
// getGraphTraversal(storage) once at creation and call its closures directly.
export function reconcileGraph(bridge, graph, changed, storage = NestedStorage) {
  return getGraphTraversal(storage).reconcileGraph(bridge, graph, changed);
}
export function matchesGraph(bridge, storage = NestedStorage) {
  return getGraphTraversal(storage).matchesGraph(bridge);
}
export function snapshotGraph(bridge, values = [], delta = false, storage = NestedStorage) {
  return getGraphTraversal(storage).snapshotGraph(bridge, values, delta);
}
