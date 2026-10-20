// Generic immutable source layouts plus sparse accepted/observed records.
// Diagram construction, state transitions and validation remain native.
import {PeleError} from './errors.js';
const kinds=['object','instance','array','map','set','date','error'];
const errors=['error','pele','type','range','syntax','reference','eval','uri'];
const STRIDE=2**24,DOMAIN=2**44,BUDGET=64*1024*1024;
const invalid=()=>{throw new PeleError('Invalid public binary graph transport','semantic');};
const limit=()=>{throw new PeleError('Public binary graph exceeds transport budget','limit');};
const u32=value=>Number.isInteger(value)&&value>=0&&value<=0xffffffff;
const bits=new DataView(new ArrayBuffer(8));
const hex=(lo,hi)=>hi.toString(16).padStart(8,'0')+lo.toString(16).padStart(8,'0');
function number(lo,hi){bits.setUint32(0,lo,true);bits.setUint32(4,hi,true);return bits.getFloat64(0,true);}
function setAtom(out,at,tag,payload){
  out[at]=tag;
  if(tag===4){bits.setFloat64(0,payload,true);out[at+1]=bits.getUint32(0,true);out[at+2]=bits.getUint32(4,true);}
  else {out[at+1]=tag>=5?payload:0;out[at+2]=0;}
}
function metadata(words){const[tag,lo,hi]=words;switch(tag){case 0:return{kind:'undefined'};case 1:return{kind:'null'};case 2:case 3:return{kind:'bool',value:tag===3};case 4:return{kind:'number',value:hex(lo,hi)};case 5:return{kind:'string',value:lo};case 6:return{kind:'ref',value:lo};case 7:return{kind:'builtinFunction',value:lo};case 8:return{kind:'hostReference',value:lo};default:invalid();}}
class Reader{
  constructor(bytes){if(!(bytes instanceof Uint8Array))invalid();this.bytes=bytes;this.view=new DataView(bytes.buffer,bytes.byteOffset,bytes.byteLength);this.at=0;}
  need(n){if(!Number.isSafeInteger(n)||n<0||n>this.bytes.length-this.at)invalid();}
  u8(){this.need(1);return this.view.getUint8(this.at++);}
  u32(){this.need(4);const v=this.view.getUint32(this.at,true);this.at+=4;return v;}
  u64(){const lo=this.u32(),hi=this.u32();return ((BigInt(hi)<<32n)|BigInt(lo)).toString();}
  count(minimum,maximum=0xffffffff){const n=this.u32();if(n>maximum)limit();this.need(n*minimum);return n;}
  atom(out,at,fixed){const tag=this.u8();if(tag>8)invalid();out[at]=tag;if(fixed){out[at+1]=this.u32();out[at+2]=this.u32();}else{out[at+1]=tag>=4?this.u32():0;out[at+2]=tag===4?this.u32():0;}}
  expression(){const op=this.u8(),tag=this.u8(),a=this.u32(),b=this.u32();if(op>5||tag>8)invalid();return{op,tag,a,b};}
  node(){
    const kind=this.u8();if(kind>6)invalid();let lo=0,hi=0,items=[];
    if(kind===1||kind===2)lo=this.u32();
    else if(kind===3||kind===4){const n=this.count(kind===3?2:1,kind===3?4000000:8000000);items=new Uint32Array(n*(kind===3?6:3));for(let at=0;at<items.length;at+=3)this.atom(items,at,false);}
    else if(kind===5){lo=this.u32();hi=this.u32();}
    else if(kind===6){lo=this.u8();if(lo>=errors.length)invalid();}
    const n=this.count(6,8000000),properties=new Uint32Array(n*5);
    for(let at=0;at<properties.length;at+=5){properties[at]=this.u32();properties[at+1]=this.u8();if(properties[at+1]&~7)invalid();this.atom(properties,at+2,false);}
    return{kind,lo,hi,properties,items};
  }
}
// One expression interpreter serves canonical-word output and semantic cursors.
// The latter keeps numeric affine expressions as numbers until raw bits are needed.
function evaluate(expr,local,item,columns,out,at){
  let tag=expr.tag,lo=expr.a,hi=expr.b,numeric;
  switch(expr.op){
    case 0:break;
    case 1:case 2:lo=expr.a+(expr.op===1?local:item)*expr.b;if(!u32(lo))invalid();hi=0;break;
    case 3:case 4:{const column=columns[expr.a],index=expr.b+(expr.op===3?local:item);if(!column||!u32(index)||index>=column.length/3)invalid();tag=column[index*3];lo=column[index*3+1];hi=column[index*3+2];break;}
    case 5:numeric=expr.a+local*expr.b;if(!u32(numeric))invalid();tag=4;lo=hi=0;break;
    default:invalid();
  }
  if(at===undefined){out.tag=tag;out.payload=tag===4?(numeric===undefined?number(lo,hi):numeric):lo;out.lo=lo;out.hi=hi;out.raw=numeric===undefined;}
  else if(numeric!==undefined)setAtom(out,at,4,numeric);
  else{out[at]=tag;out[at+1]=lo;out[at+2]=hi;}
}
function selectProperty(group,index,target){
  target.itemRow=0;target.template=group.properties[index];
  if(index>=group.properties.length){let at=index-group.properties.length;for(const p of group.dynamic){if(at<p.count){target.template=p;target.itemRow=at;break;}at-=p.count;}}
  if(!target.template)invalid();
}
function selectItem(group,index,target){
  const pair=group.kind===3?2:1;let at=Math.floor(index/pair);
  for(const collection of group.collections){if(at<collection.count){target.template=pair===2&&index%2===0?collection.key:collection.value;target.itemRow=at;return;}at-=collection.count;}
  invalid();
}
// One cursor belongs to one invocation. It retains only references into the
// immutable graph plus scalar registers, never a materialized record array.
class LayoutNodeCursor{
  constructor(graph){this.graph=graph;this.id=-1;this.record=null;this.override=false;this.local=0;this.tag=0;this.payload=0;this.lo=0;this.hi=0;this.raw=true;this.template=null;this.itemRow=0;}
  select(id){
    if(id===this.id)return;
    const override=this.graph.overrides.get(id),record=override??this.graph.group(id);
    this.id=id;this.record=record;this.override=override!==undefined;this.local=this.override?0:id-record.first;
    this.kind=kinds[record.kind];this.propertyCount=this.override?record.properties.length/5:record.propertyCount;
    this.itemAtoms=this.override?record.items.length/3:record.itemAtoms;this.itemCount=this.itemAtoms/(record.kind===3?2:1);
    this.aux=record.kind===5?hex(record.lo,record.hi):record.kind===6?(record.hi?undefined:errors[record.lo]):record.kind===2&&record.hi?undefined:record.lo;
  }
  *properties(){for(let at=0;at<this.propertyCount;at++)yield at;}
  words(words,at){this.tag=words[at];this.lo=words[at+1];this.hi=words[at+2];this.raw=true;this.payload=this.tag===4?number(this.lo,this.hi):this.lo;}
  key(index){
    if(this.override){if(index>=this.propertyCount)invalid();return this.record.properties[index*5];}
    selectProperty(this.record,index,this);evaluate(this.template.key,this.local,this.itemRow,this.graph.base.columns,this);
    if(this.tag!==5)invalid();return this.payload;
  }
  property(index){
    this.keyId=this.key(index);
    if(this.override){const at=index*5;this.flags=this.record.properties[at+1];this.words(this.record.properties,at+2);}
    else{this.flags=this.template.flags;evaluate(this.template.value,this.local,this.itemRow,this.graph.base.columns,this);}
  }
  item(index){
    if(this.override){if(index>=this.itemAtoms)invalid();this.words(this.record.items,index*3);}
    else{selectItem(this.record,index,this);evaluate(this.template,this.local,this.itemRow,this.graph.base.columns,this);}
  }
  writeAtom(out,at){if(this.tag===4&&!this.raw)setAtom(out,at,4,this.payload);else{out[at]=this.tag;out[at+1]=this.lo;out[at+2]=this.hi;}}
  propertyWords(index,out){this.property(index);out[0]=this.keyId;out[1]=this.flags;this.writeAtom(out,2);}
  itemWords(index,out){this.item(index);this.writeAtom(out,0);}
}
function missing(){const node=undefined;return node.properties;}
export class LayoutGraph{
  constructor(base,strings,overrides,nodeCount,root){this.base=base;this.strings=strings;this.overrides=overrides;this.nodeCount=nodeCount;this.rootWords=root;this.root=metadata(root);this.version=1;this.scratch=new Uint32Array(5);}
  group(id){if(!u32(id)||id>=this.nodeCount)missing();const groups=this.base.groups;let low=0,high=groups.length;while(low<high){const mid=(low+high)>>>1;if(groups[mid].first<=id)low=mid+1;else high=mid;}const g=groups[low-1];if(!g||id-g.first>=g.count)missing();return g;}
  hasNode(id){if(!u32(id)||id>=this.nodeCount)return false;if(this.overrides.has(id))return true;return this.base.groups.some(g=>id>=g.first&&id-g.first<g.count);}
  kind(id){return this.overrides.get(id)?.kind??this.group(id).kind;}
  aux(id){return this.overrides.get(id)??this.group(id);}
  propertyCount(id){const o=this.overrides.get(id);return o?o.properties.length/5:this.group(id).propertyCount;}
  itemAtoms(id){const o=this.overrides.get(id);return o?o.items.length/3:this.group(id).itemAtoms;}
  property(id,index,out){
    const override=this.overrides.get(id);
    if(override){if(index>=override.properties.length/5)invalid();for(let k=0;k<5;k++)out[k]=override.properties[index*5+k];return;}
    const g=this.group(id),local=id-g.first;
    selectProperty(g,index,this);const p=this.template,item=this.itemRow;
    evaluate(p.key,local,item,this.base.columns,out,2);if(out[2]!==5)invalid();out[0]=out[3];out[1]=p.flags;evaluate(p.value,local,item,this.base.columns,out,2);
  }
  item(id,index,out){
    const override=this.overrides.get(id);
    if(override){if(index>=override.items.length/3)invalid();for(let k=0;k<3;k++)out[k]=override.items[index*3+k];return;}
    const g=this.group(id);selectItem(g,index,this);evaluate(this.template,id-g.first,this.itemRow,this.base.columns,out,0);
  }
  atom(slot,out){
    if(slot===0){for(let k=0;k<3;k++)out[k]=this.rootWords[k];return;}
    const domain=Math.floor(slot/DOMAIN),value=slot-domain*DOMAIN;
    if(domain===1){this.property(Math.floor(value/STRIDE),value%STRIDE,this.scratch);for(let k=0;k<3;k++)out[k]=this.scratch[k+2];return;}
    if(domain===2){this.item(Math.floor(value/STRIDE),value%STRIDE,out);return;}
    invalid();
  }
}
function readText(r){
  const n=r.count(2,16000000);let text='';
  for(let left=n;left;){const size=Math.min(left,8192),chunk=new Array(size);for(let i=0;i<size;i++){chunk[i]=r.view.getUint16(r.at,true);r.at+=2;}text+=String.fromCharCode(...chunk);left-=size;}
  return text;
}
function readCatalog(r){
  const count=r.count(4,1000000),strings=[],unitPrefix=new Uint32Array(count+1);let units=0;
  for(let id=0;id<count;id++){const text=readText(r);units+=text.length;if(units>16000000)limit();strings.push(text);unitPrefix[id+1]=units;}
  return{strings,unitPrefix};
}
function readLayoutBase(r){
  const columns=[];for(let n=r.count(4);n;n--){const size=r.count(9),column=new Uint32Array(size*3);for(let i=0;i<size;i++)r.atom(column,i*3,true);columns.push(column);}
  const groups=[];let previousEnd=0;
  for(let n=r.count(25);n;n--){
    const first=r.u32(),count=r.u32(),kind=r.u8(),lo=r.u32(),hi=r.u32();if(kind>6||first<previousEnd||!u32(first+count))invalid();if(count)previousEnd=first+count;
    const g={first,count,kind,lo,hi,properties:[],dynamic:[],collections:[],propertyCount:0,itemAtoms:0};
    const prop=dynamic=>{const count=dynamic?r.u32():1,key=r.expression(),flags=r.u8(),value=r.expression();if(flags&~7)invalid();g.propertyCount+=count;if(g.propertyCount>8000000)limit();return{count,key,flags,value};};
    for(let x=r.count(21);x;x--)g.properties.push(prop(false));
    for(let x=r.count(25);x;x--)g.dynamic.push(prop(true));
    const collections=r.count(kind===3?24:14);if(collections&&kind!==3&&kind!==4)invalid();
    for(let x=0;x<collections;x++){const count=r.u32(),key=kind===3?r.expression():null,value=r.expression();g.itemAtoms+=count*(kind===3?2:1);if(g.itemAtoms>8000000)limit();g.collections.push({count,key,value});}if(count)groups.push(g);
  }
  return{groups,columns};
}
function readStringPatches(r,base,count){
  const original=base.originalStrings,n=r.count(8,1000000);
  if(n===0&&count===original.length)return original;
  const strings=original.slice(0,count);strings.length=count;
  let units=base.unitPrefix[Math.min(count,original.length)],appended=0;
  const seen=new Set();
  for(let i=0;i<n;i++){
    const id=r.u32();if(id>=count||seen.has(id))invalid();seen.add(id);
    const text=readText(r);units+=text.length-(id<original.length?original[id].length:0);strings[id]=text;if(id>=original.length)appended++;
  }
  if(appended!==Math.max(0,count-original.length))invalid();
  if(units>16000000)limit();
  return strings;
}
function readOverrides(r,nodeCount){
  const overrides=new Map();for(let n=r.count(9,1000000);n;n--){const id=r.u32();if(id>=nodeCount||overrides.has(id))invalid();overrides.set(id,r.node());}return overrides;
}
export function decodeSourceLayout(bytes,baseline){
  if(!(bytes instanceof Uint8Array))invalid();if(bytes.length>BUDGET)limit();
  if(bytes[0]===80&&bytes[1]===69)return decodeCanonicalLayout(bytes,baseline);
  const r=new Reader(bytes);let magic='';for(let i=0;i<8;i++)magic+=String.fromCharCode(r.u8());
  const legacy=magic==='PLCSRC01',full=magic==='PLCSRC02',update=magic==='PLCSRU01';if(!legacy&&!full&&!update)invalid();
  const layoutId=legacy?undefined:r.u64();if(!legacy&&layoutId==='0')invalid();
  const revision=r.u64();if(r.u32()!==1)invalid();const root=new Uint32Array(3);r.atom(root,0,true);
  const stringCount=r.u32(),nodeCount=r.u32();if(stringCount>1000000||nodeCount>1000000)limit();
  let base,strings;
  if(update){
    if(!(baseline instanceof LayoutGraph)||baseline.base.layoutId!==layoutId||!baseline.base.originalStrings)invalid();
    base=baseline.base;strings=readStringPatches(r,base,stringCount);
  }else{
    const catalog=readCatalog(r);if(legacy&&catalog.strings.length!==stringCount)invalid();
    base=readLayoutBase(r);
    if(full){base.layoutId=layoutId;base.originalStrings=catalog.strings;base.unitPrefix=catalog.unitPrefix;strings=readStringPatches(r,base,stringCount);}
    else strings=catalog.strings;
  }
  const overrides=readOverrides(r,nodeCount);if(r.at!==bytes.length)invalid();
  const graph=new LayoutGraph(base,strings,overrides,nodeCount,root);
  // Only effective rows must exist. Unused original source rows may reference
  // IDs outside a truncated current extent; complete overrides take precedence.
  for(let id=0;id<nodeCount;id++)if(!overrides.has(id))graph.group(id);
  return{revision,graph};
}

// Callback deltas keep the established JSON contract, but allocate records only
// for changed nodes. The immutable source baseline is never expanded here.
function stringUnits(text){const units=new Array(text.length);for(let i=0;i<text.length;i++)units[i]=text.charCodeAt(i);return units;}
function unitString(units){let text='';for(let i=0;i<units.length;i+=8192)text+=String.fromCharCode(...units.slice(i,i+8192));return text;}
function canonicalRecord(record){
  const node={kind:kinds[record.kind]},words=record.properties,items=record.items;
  if(record.kind===1)node.prototype=record.lo;
  else if(record.kind===2)node.length=record.hi?record.invalidAux:record.lo;
  else if(record.kind===5)node.milliseconds=hex(record.lo,record.hi);
  else if(record.kind===6)node.errorType=record.hi?record.invalidAux:errors[record.lo];
  else if(record.kind===3){node.entries=[];for(let i=0;i<items.length;i+=6)node.entries.push([metadata(items.slice(i,i+3)),metadata(items.slice(i+3,i+6))]);}
  else if(record.kind===4){node.values=[];for(let i=0;i<items.length;i+=3)node.values.push(metadata(items.slice(i,i+3)));}
  node.properties=[];for(let i=0;i<words.length;i+=5)node.properties.push({key:words[i],flags:words[i+1],value:metadata(words.slice(i+2,i+5))});
  return node;
}
function canonicalAtom(atom,out,at){
  switch(atom.kind){
    case'undefined':setAtom(out,at,0,0);return;case'null':setAtom(out,at,1,0);return;case'bool':setAtom(out,at,atom.value?3:2,0);return;
    case'number':out[at]=4;out[at+1]=parseInt(atom.value.slice(8),16);out[at+2]=parseInt(atom.value.slice(0,8),16);return;
    case'string':setAtom(out,at,5,atom.value);return;case'ref':setAtom(out,at,6,atom.value);return;
    case'builtinFunction':setAtom(out,at,7,atom.value);return;case'hostReference':setAtom(out,at,8,atom.value);return;default:invalid();
  }
}
function packedRecord(node){
  const kind=kinds.indexOf(node.kind);if(kind<0)invalid();let lo=0,hi=0,invalidAux;
  if(kind===1)lo=node.prototype;
  else if(kind===2){if(u32(node.length))lo=node.length;else{hi=1;invalidAux=node.length;}}
  else if(kind===5){lo=parseInt(node.milliseconds.slice(8),16);hi=parseInt(node.milliseconds.slice(0,8),16);}
  else if(kind===6){lo=errors.indexOf(node.errorType);if(lo<0){lo=0;hi=1;invalidAux=node.errorType;}}
  const properties=new Uint32Array(node.properties.length*5);
  for(let i=0;i<node.properties.length;i++){const p=node.properties[i],at=i*5;properties[at]=p.key;properties[at+1]=p.flags;canonicalAtom(p.value,properties,at+2);}
  const items=new Uint32Array(kind===3?node.entries.length*6:kind===4?node.values.length*3:0);
  if(kind===3)for(let i=0;i<node.entries.length;i++){canonicalAtom(node.entries[i][0],items,i*6);canonicalAtom(node.entries[i][1],items,i*6+3);}
  if(kind===4)for(let i=0;i<node.values.length;i++)canonicalAtom(node.values[i],items,i*3);
  return{kind,lo,hi,invalidAux,properties,items};
}
export function applyLayoutDelta(graph,delta){
  if(delta.stringsStart!==graph.strings.length||delta.nodesStart!==graph.nodeCount)throw new TypeError('Stale native model delta');
  const overrides=new Map(graph.overrides);let nodeCount=graph.nodeCount;
  for(const[id,node]of delta.nodes){overrides.set(id,packedRecord(node));nodeCount=Math.max(nodeCount,id+1);}
  return new LayoutGraph(graph.base,graph.strings.concat(delta.strings.map(unitString)),overrides,nodeCount,graph.rootWords);
}

class LayoutCapture{
  constructor(baseline,delta){this.baseline=baseline;this.cursor=new LayoutNodeCursor(baseline);this.delta=delta;this.overrides=new Map(baseline.overrides);this.rootWords=new Uint32Array(3);this.values=[];this.changes=[];this.nodeCount=baseline.nodeCount;this.node=null;this.words=new Uint32Array(5);this.atomWords=new Uint32Array(3);}
  root(tag,value){setAtom(this.rootWords,0,tag,value);}
  beginArguments(length){this.values=new Array(length);}
  argument(index,tag,value){setAtom(this.atomWords,0,tag,value);this.values[index]=metadata(this.atomWords);}
  beginNode(id){const existing=this.baseline.hasNode(id);if(existing)this.cursor.select(id);this.node={id,existing,properties:null,items:null,propertyAt:0,itemAt:0};}
  copyProperties(count){const words=[];for(let i=0;i<count;i++){this.cursor.propertyWords(i,this.words);for(let k=0;k<5;k++)words.push(this.words[k]);}return words;}
  copyItems(count){const words=[];for(let i=0;i<count;i++){this.cursor.itemWords(i,this.atomWords);for(let k=0;k<3;k++)words.push(this.atomWords[k]);}return words;}
  property(key,flags,tag,value){
    const n=this.node,index=n.propertyAt++;
    let same=false;if(!n.properties&&n.existing&&index<this.cursor.propertyCount){
      this.cursor.propertyWords(index,this.words);setAtom(this.atomWords,0,tag,value);same=this.words[0]===key&&this.words[1]===flags&&this.words[2]===this.atomWords[0]&&this.words[3]===this.atomWords[1]&&this.words[4]===this.atomWords[2];
    }
    if(same)return;
    if(!n.properties)n.properties=this.copyProperties(index);
    n.properties.push(key,flags);const at=n.properties.length;n.properties.length+=3;setAtom(n.properties,at,tag,value);
  }
  beginItems(){}
  appendItem(tag,value){const n=this.node,index=n.itemAt++;let same=false;if(!n.items&&n.existing&&index<this.cursor.itemAtoms){this.cursor.itemWords(index,this.words);setAtom(this.atomWords,0,tag,value);same=this.words[0]===this.atomWords[0]&&this.words[1]===this.atomWords[1]&&this.words[2]===this.atomWords[2];}if(same)return;if(!n.items)n.items=this.copyItems(index);const at=n.items.length;n.items.length+=3;setAtom(n.items,at,tag,value);}
  mapEntry(kt,k,vt,v){this.appendItem(kt,k);this.appendItem(vt,v);}
  setItem(tag,value){this.appendItem(tag,value);}
  finishNode(id,kind,aux){
    const n=this.node;if(n.id!==id)invalid();const code=kinds.indexOf(kind);let lo=0,hi=0;
    if(code===5){lo=parseInt(aux.slice(8),16);hi=parseInt(aux.slice(0,8),16);}else if(code===6){lo=errors.indexOf(aux);if(lo<0){lo=0;hi=1;}}else if(code===2){if(u32(aux))lo=aux;else hi=1;}else if(code===1)lo=aux;
    let same=false;if(n.existing){const old=this.cursor.record;same=code===old.kind&&lo===old.lo&&hi===old.hi&&n.propertyAt===this.cursor.propertyCount&&n.itemAt===this.cursor.itemAtoms&&!n.properties&&!n.items;}
    if(!same){const record={kind:code,lo,hi,...(hi&&(code===2||code===6)?{invalidAux:aux}:{}),properties:n.properties??this.copyProperties(n.propertyAt),items:n.items??this.copyItems(n.itemAt)};this.overrides.set(id,record);this.changes.push([id,record]);}
    this.nodeCount=Math.max(this.nodeCount,id+1);this.node=null;
  }
  finish(strings){if(this.node)invalid();const graph=new LayoutGraph(this.baseline.base,strings.slice(),this.overrides,this.nodeCount,this.rootWords);const result={graph,values:this.values};if(this.delta)result.delta={stringsStart:this.baseline.strings.length,strings:graph.strings.slice(this.baseline.strings.length).map(stringUnits),nodesStart:this.baseline.nodeCount,nodes:this.changes.sort((a,b)=>a[0]-b[0]).map(([id,record])=>[id,canonicalRecord(record)])};return result;}
  abort(){this.overrides=null;this.node=null;this.values=null;}
}
const atomScratch=new Uint32Array(3);
export const LayoutStorage=Object.freeze({
  createNodeCursor:graph=>new LayoutNodeCursor(graph),
  stringCount:g=>g.strings.length,stringValueAt:(g,id)=>g.strings[id],nodeCount:g=>g.nodeCount,
  *nodeIds(g){for(let id=0;id<g.nodeCount;id++)yield id;},rootSlot:()=>0,
  nodeKind:(g,id)=>kinds[g.kind(id)],nodeAux(g,id){const a=g.aux(id),kind=g.kind(id);return kind===5?hex(a.lo,a.hi):kind===6?(a.hi?undefined:errors[a.lo]):kind===2&&a.hi?undefined:a.lo;},
  propertyCount:(g,id)=>g.propertyCount(id),propertySlot:(_g,id,at)=>id*STRIDE+at,
  *propertySlots(g,id){for(let at=0;at<g.propertyCount(id);at++)yield id*STRIDE+at;},
  readProperty(g,slot,out){g.property(Math.floor(slot/STRIDE),slot%STRIDE,out);},
  propertyKey(g,slot){g.property(Math.floor(slot/STRIDE),slot%STRIDE,g.scratch);return g.scratch[0];},
  propertyFlags(g,slot){g.property(Math.floor(slot/STRIDE),slot%STRIDE,g.scratch);return g.scratch[1];},propertyAtomSlot:(_g,slot)=>DOMAIN+slot,
  itemCount:(g,id)=>g.itemAtoms(id)/(g.kind(id)===3?2:1),itemAtomSlot:(_g,id,at)=>2*DOMAIN+id*STRIDE+at,
  atomTag(g,slot){g.atom(slot,atomScratch);return atomScratch[0];},atomPayload(g,slot){g.atom(slot,atomScratch);return atomScratch[0]===4?number(atomScratch[1],atomScratch[2]):atomScratch[1];},atomNumberBits(g,slot){g.atom(slot,atomScratch);return hex(atomScratch[1],atomScratch[2]);},
  beginCapture:(g,delta=false)=>new LayoutCapture(g,delta),
});

function sameRecord(graph,id,record,words){
  if(!graph.hasNode(id)||graph.kind(id)!==record.kind)return false;
  const aux=graph.aux(id);if(aux.lo!==record.lo||aux.hi!==record.hi||graph.propertyCount(id)!==record.properties.length/5||graph.itemAtoms(id)!==record.items.length/3)return false;
  for(let i=0;i<record.properties.length/5;i++){graph.property(id,i,words);for(let k=0;k<5;k++)if(words[k]!==record.properties[i*5+k])return false;}
  for(let i=0;i<record.items.length/3;i++){graph.item(id,i,words);for(let k=0;k<3;k++)if(words[k]!==record.items[i*3+k])return false;}
  return true;
}
// Bounded canonical fallback is consumed one record at a time, retaining the
// same immutable layout base and only replacements. No nested graph upgrade.
function decodeCanonicalLayout(bytes,baseline){
  if(!(baseline instanceof LayoutGraph))invalid();if(bytes.length>BUDGET)limit();
  const r=new Reader(bytes);for(const b of [80,69,76,71,66,73,78,49])if(r.u8()!==b)invalid();if(r.u32()!==1)invalid();const revision=r.u64();if(r.u32()!==0)invalid();
  const root=new Uint32Array(3);r.atom(root,0,false);const strings=[];let units=0;
  for(let n=r.count(4,1000000);n;n--){const length=r.count(2,16000000);units+=length;if(units>16000000)limit();let text='';for(let at=0;at<length;){const size=Math.min(length-at,8192),chunk=new Array(size);for(let i=0;i<size;i++){chunk[i]=r.view.getUint16(r.at,true);r.at+=2;}text+=String.fromCharCode(...chunk);at+=size;}strings.push(text);}
  const count=r.count(5,1000000),graph=new LayoutGraph(baseline.base,strings,new Map(),count,root),words=new Uint32Array(5);let work=0;
  for(let id=0;id<count;id++){const node=r.node();work+=node.properties.length/5+node.items.length/3;if(work>8000000)limit();if(!sameRecord(graph,id,node,words))graph.overrides.set(id,node);}
  if(r.at!==bytes.length)invalid();return{revision,graph};
}
const atomSize=tag=>tag===4?9:tag>=5?5:1;
export function encodeLayoutBinary(revision,graph,metadataBytes=new Uint8Array()){
  let size=32+atomSize(graph.rootWords[0])+metadataBytes.length;
  for(const text of graph.strings)size+=4+2*text.length;
  const words=new Uint32Array(5);
  for(let id=0;id<graph.nodeCount;id++){
    const count=graph.propertyCount(id),kind=graph.kind(id);size+=5;
    for(let at=0;at<count;at++){graph.property(id,at,words);size+=5+atomSize(words[2]);}
    if(kind===1||kind===2)size+=4;else if(kind===5)size+=8;else if(kind===6)size+=1;
    else if(kind===3||kind===4){size+=4;for(let at=0;at<graph.itemAtoms(id);at++){graph.item(id,at,words);size+=atomSize(words[0]);}}
    else if(kind!==0)invalid();
  }
  if(!Number.isSafeInteger(size)||size>BUDGET)limit();
  const bytes=new Uint8Array(size),view=new DataView(bytes.buffer);let at=0;
  const byte=value=>view.setUint8(at++,value),word=value=>{if(!u32(value))invalid();view.setUint32(at,value,true);at+=4;};
  const atom=(tag,lo,hi)=>{if(tag>8)invalid();byte(tag);if(tag>=4)word(lo);if(tag===4)word(hi);};
  for(const b of [80,69,76,71,66,73,78,49])byte(b);word(1);view.setBigUint64(at,BigInt(revision),true);at+=8;word(metadataBytes.length);
  atom(...graph.rootWords);word(graph.strings.length);for(const text of graph.strings){word(text.length);for(let i=0;i<text.length;i++){view.setUint16(at,text.charCodeAt(i),true);at+=2;}}
  word(graph.nodeCount);
  for(let id=0;id<graph.nodeCount;id++){
    const kind=graph.kind(id),aux=graph.aux(id);byte(kind);
    if(kind===1||kind===2){if(kind===2&&aux.hi)invalid();word(aux.lo);}
    else if(kind===6){if(aux.hi||aux.lo>=errors.length)invalid();byte(aux.lo);}
    else if(kind===5){word(aux.lo);word(aux.hi);}
    else if(kind===3||kind===4){word(graph.itemAtoms(id)/(kind===3?2:1));for(let i=0;i<graph.itemAtoms(id);i++){graph.item(id,i,words);atom(words[0],words[1],words[2]);}}
    word(graph.propertyCount(id));for(let i=0;i<graph.propertyCount(id);i++){graph.property(id,i,words);word(words[0]);byte(words[1]);atom(words[2],words[3],words[4]);}
  }
  bytes.set(metadataBytes,at);at+=metadataBytes.length;if(at!==size)invalid();return bytes;
}
