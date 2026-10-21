import {LayoutGraph,encodeLayoutBinary} from './layout-storage.js';
// PELGBIN1 transports identity tables. Diagram state and behavior stay in Rust.
import {PeleError} from './errors.js';
const errorTypes=['error','pele','type','range','syntax','reference','eval','uri'];
const BUDGET=64*1024*1024,magic=[80,69,76,71,66,73,78,49];
const invalid=()=>new PeleError('Invalid public binary graph transport','semantic');
const limit=()=>new PeleError('Public binary graph exceeds transport budget','limit');
const atomSize=a=>a.kind==='number'?9:['string','ref','builtinFunction','hostReference'].includes(a.kind)?5:1;
const propertiesSize=p=>4+p.reduce((n,p)=>n+5+atomSize(p.value),0);
export function encodeGraphBinary(revision,graph,metadata=new Uint8Array()){
  if(graph instanceof LayoutGraph)return encodeLayoutBinary(revision,graph,metadata);
  if(graph.version!==1)throw invalid();
  let size=24+atomSize(graph.root)+4+4+metadata.length;
  for(const s of graph.strings)size+=4+2*s.length;
  for(const node of graph.nodes){
    size+=1+propertiesSize(node.properties);
    switch(node.kind){
      case'error':size+=1;break;
      case'instance':case'array':size+=4;break;
      case'date':size+=8;break;
      case'map':size+=4;for(const[k,v]of node.entries)size+=atomSize(k)+atomSize(v);break;
      case'set':size+=4;for(const v of node.values)size+=atomSize(v);break;
      case'object':break;default:throw invalid();
    }
  }
  if(!Number.isSafeInteger(size)||size>BUDGET)throw limit();
  const bytes=new Uint8Array(size),view=new DataView(bytes.buffer);let offset=0;
  const u8=v=>{view.setUint8(offset++,v);};
  const u32=v=>{if(!Number.isInteger(v)||v<0||v>0xffffffff)throw invalid();view.setUint32(offset,v,true);offset+=4;};
  const bits=hex=>{if(typeof hex!=='string'||!/^[0-9a-fA-F]{16}$/.test(hex))throw invalid();view.setBigUint64(offset,BigInt('0x'+hex),true);offset+=8;};
  const atom=a=>{switch(a.kind){
    case'undefined':u8(0);break;case'null':u8(1);break;case'bool':u8(a.value?3:2);break;
    case'number':u8(4);bits(a.value);break;case'string':u8(5);u32(a.value);break;
    case'ref':u8(6);u32(a.value);break;case'builtinFunction':u8(7);u32(a.value);break;
    case'hostReference':u8(8);u32(a.value);break;default:throw invalid();
  }};
  const properties=p=>{u32(p.length);for(const value of p){u32(value.key);u8(value.flags);atom(value.value);}};
  for(const v of magic)u8(v);u32(graph.version);view.setBigUint64(offset,BigInt(revision),true);offset+=8;u32(metadata.length);
  atom(graph.root);u32(graph.strings.length);
  for(const s of graph.strings){u32(s.length);for(const unit of s){if(!Number.isInteger(unit)||unit<0||unit>65535)throw invalid();view.setUint16(offset,unit,true);offset+=2;}}
  u32(graph.nodes.length);
  for(const node of graph.nodes){switch(node.kind){
    case'error':{const tag=errorTypes.indexOf(node.errorType);if(tag<0)throw invalid();u8(6);u8(tag);break;}
    case'object':u8(0);break;case'instance':u8(1);u32(node.prototype);break;
    case'array':u8(2);u32(node.length);break;
    case'map':u8(3);u32(node.entries.length);for(const[k,v]of node.entries){atom(k);atom(v);}break;
    case'set':u8(4);u32(node.values.length);for(const v of node.values)atom(v);break;
    case'date':u8(5);bits(node.milliseconds);break;
  }properties(node.properties);}
  bytes.set(metadata,offset);offset+=metadata.length;
  if(offset!==size)throw invalid();return bytes;
}
export function decodeGraphBinary(bytes){
  if(!(bytes instanceof Uint8Array)||bytes.length>BUDGET)throw limit();
  const view=new DataView(bytes.buffer,bytes.byteOffset,bytes.byteLength);let offset=0,end=bytes.length,work=0,units=0;
  const need=n=>{if(!Number.isSafeInteger(n)||n<0||n>end-offset)throw invalid();};
  const u8=()=>{need(1);return view.getUint8(offset++);};
  const u32=()=>{need(4);const v=view.getUint32(offset,true);offset+=4;return v;};
  const bits=()=>{need(8);const v=view.getBigUint64(offset,true).toString(16).padStart(16,'0');offset+=8;return v;};
  const count=(maximum,minimum)=>{const n=u32();if(n>maximum)throw limit();need(n*minimum);return n;};
  const addWork=n=>{work+=n;if(work>8_000_000)throw limit();};
  const atom=()=>{switch(u8()){
    case 0:return{kind:'undefined'};case 1:return{kind:'null'};case 2:return{kind:'bool',value:false};case 3:return{kind:'bool',value:true};
    case 4:return{kind:'number',value:bits()};case 5:return{kind:'string',value:u32()};case 6:return{kind:'ref',value:u32()};
    case 7:return{kind:'builtinFunction',value:u32()};case 8:return{kind:'hostReference',value:u32()};default:throw invalid();
  }};
  const properties=()=>{const n=count(8_000_000,6);addWork(n);const out=[];for(let i=0;i<n;i++){const key=u32(),flags=u8(),value=atom();if(flags&~7)throw invalid();out.push({key,flags,value});}return out;};
  for(const v of magic)if(u8()!==v)throw invalid();
  const version=u32();if(version!==1)throw invalid();
  need(8);const revision=view.getBigUint64(offset,true).toString();offset+=8;
  const metadataLength=u32();need(metadataLength);end-=metadataLength;
  const root=atom(),strings=[],nodes=[];let n=count(1_000_000,4);
  for(let i=0;i<n;i++){const length=count(16_000_000,2);units+=length;if(units>16_000_000)throw limit();const s=new Array(length);for(let j=0;j<length;j++){s[j]=view.getUint16(offset,true);offset+=2;}strings.push(s);}
  n=count(1_000_000,5);
  for(let i=0;i<n;i++){
    let node;switch(u8()){
      case 6:{const type=u8();if(type>=errorTypes.length)throw invalid();node={kind:'error',errorType:errorTypes[type]};break;}
      case 0:node={kind:'object'};break;
      case 1:node={kind:'instance',prototype:u32()};break;
      case 2:node={kind:'array',length:u32()};break;
      case 3:{const length=count(4_000_000,2);addWork(length*2);const entries=[];for(let j=0;j<length;j++)entries.push([atom(),atom()]);node={kind:'map',entries};break;}
      case 4:{const length=count(8_000_000,1);addWork(length);const values=[];for(let j=0;j<length;j++)values.push(atom());node={kind:'set',values};break;}
      case 5:node={kind:'date',milliseconds:bits()};break;default:throw invalid();
    }node.properties=properties();nodes.push(node);
  }
  if(offset!==end)throw invalid();
  const check=a=>{if(a.kind==='ref'&&a.value>=nodes.length||a.kind==='string'&&a.value>=strings.length)throw invalid();};check(root);
  for(const node of nodes){for(const p of node.properties){if(p.key>=strings.length)throw invalid();check(p.value);}if(node.kind==='map')for(const[k,v]of node.entries){check(k);check(v);}if(node.kind==='set')for(const value of node.values)check(value);}
  const metadata=metadataLength?JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(bytes.subarray(end))):null;
  return{revision,graph:{version,root,strings,nodes},metadata};
}
