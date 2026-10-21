// Build-time compilation of Rust-owned RecordSchema + initial source recipes.
// Run against a built package; never parse diagram syntax or infer field names here.
import {readFile,writeFile} from 'node:fs/promises';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
const [packagePath,decoderPath,outputPath]=process.argv.slice(2);
if(!outputPath)throw Error('usage: node generate-source-initializer.mjs PACKAGE LAYOUT_DECODER OUTPUT');
const {initSync,NativeRegistry}=await import(pathToFileURL(resolve(packagePath,'pkg/pele.js')));
const {decodeSourceLayout}=await import(pathToFileURL(resolve(decoderPath)));
initSync({module:await readFile(resolve(packagePath,'pkg/pele_bg.wasm'))});
const registry=new NativeRegistry(true);let document;
let schema,base,strings;
try{
  const envelope=JSON.parse(registry.entry_schema('sankey'));if(!envelope.ok)throw Error('Missing native schema');schema=envelope.result;
  document=registry.parse('sankey\nA,B,1\nB,C,2','{}',0,0,0,0);
  const packet=document.snapshot_source_layout(),decoded=decodeSourceLayout(packet[2]);base=decoded.graph.base;strings=decoded.graph.strings;
}finally{document?.free();registry.close();registry.free();}
const groups=base.groups.map(g=>{
 const names=g.properties.map(p=>{if(p.key.op!==0||p.key.tag!==5||p.key.b!==0||p.flags!==7)throw Error('Unsupported key schema');return strings[p.key.a];});
 if(new Set(names).size!==names.length||names.includes('__proto__'))throw Error('Unsafe fixed keys');
 const record=g.properties.length?schema.records.find(r=>r.fields.length===names.length&&r.fields.every((f,i)=>f.name===names[i]&&f.flags===g.properties[i].flags)&&((g.kind===1&&r.prototype===g.lo)||(g.kind===0&&r.prototype===null))):null;
 if(names.length&&!record)throw Error('Recipe disagrees with native RecordSchema');
 const expression=(e,field)=>{
  if(e.op===0){if(!field)throw Error('Unexpected literal binding');return{op:0,field:field.kind,undefined:field.undefined,nullable:field.nullable};}
  if(![2,3,4,5].includes(e.op))throw Error('Unsupported source operand');
  return{op:e.op,tag:e.tag,...(e.op===2?{b:e.b}:{a:e.a,b:e.b}),...(field?{field:field.kind}: {})};
 };
 return{kind:g.kind,prototype:g.kind===1?g.lo:null,
  properties:g.properties.map((p,i)=>({name:names[i],flags:p.flags,value:expression(p.value,record.fields[i])})),
  dynamic:g.dynamic.map(p=>({flags:p.flags,key:expression(p.key),value:expression(p.value)})),
  collections:g.collections.map(c=>({key:expression(c.key),value:expression(c.value)}))};
});
if(groups.length!==6||groups.some(g=>![0,1,2,3].includes(g.kind)))throw Error('Unsupported initial layout');
const columnKinds=new Map();
for(const g of groups){for(const p of g.properties)if(p.value.op===3){const kind=p.value.field.kind;const tag=kind==='string'?5:kind==='number'?4:kind==='record'?6:0;if(!tag)throw Error('Unsupported source field');if(columnKinds.has(p.value.a)&&columnKinds.get(p.value.a)!==tag)throw Error('Conflicting column');columnKinds.set(p.value.a,tag);}
 for(const p of g.dynamic)columnKinds.set(p.key.a,5);for(const c of g.collections)columnKinds.set(c.key.a,5);}
if(columnKinds.size!==base.columns.length)throw Error('Unbound column');
const value=(e,site,index='i')=>{
 if(e.op===0)return`literal(${site},bridge)`;
 if(e.op===2)return`objects[${site}.a+${index}*${e.b}]`;
 if(e.op===5)return`${e.a}+${index}*${e.b}`;
 const at=`${index}+${e.b}`,col=`c${e.a}`;
 if(e.field?.kind==='number')return`number(${col}.loAt(${at}),${col}.hiAt(${at}))`;
 return`${e.field?.kind==='record'?'objects':'strings'}[${col}.loAt(${at})]`;
};
let body='';
for(let n=0;n<groups.length;n++){
 const g=groups[n],at=`g${n}`;
 body+=`  const ${at}=groups[${n}];\n`;
 if(g.kind===0){body+=`  for(let i=0;i<${at}.count;i++){const object={${g.properties.map((p,j)=>`${JSON.stringify(p.name)}:${value(p.value,`${at}.properties[${j}].value`)}`).join(',')}};publish(bridge,object);}\n`;}
 else if(g.kind===2){if(g.dynamic.length!==1||g.properties.length)throw Error('Unsupported dense array');body+=`  {const object=new Array(${at}.lo);publish(bridge,object);for(let i=0;i<${at}.dynamic[0].count;i++)object[i]=${value(g.dynamic[0].value,`${at}.dynamic[0].value`)};}\n`;}
 else if(g.kind===3){if(g.collections.length!==1||g.properties.length)throw Error('Unsupported map');const c=g.collections[0];body+=`  {const object=new Map();publish(bridge,object);for(let i=0;i<${at}.collections[0].count;i++)Map.prototype.set.call(object,${value(c.key,`${at}.collections[0].key`)},${value(c.value,`${at}.collections[0].value`)});}\n`;}
 else {body+=`  {const object=Object.create(bridge.registration.prototypes.get(${at}.lo));publish(bridge,object);\n`;for(let j=0;j<g.properties.length;j++){const p=g.properties[j];body+=`    Object.defineProperty(object,${JSON.stringify(p.name)},{value:${value(p.value,`${at}.properties[${j}].value`)},enumerable:true,writable:true,configurable:true});\n`;}body+='  }\n';}
}
const helpers=String.raw`
const schemaMatch=new WeakMap(),bits=new DataView(new ArrayBuffer(8));
function number(lo,hi){bits.setUint32(0,lo,true);bits.setUint32(4,hi,true);return bits.getFloat64(0,true);}
function atom(e,limit,graph,field,optional,nullable){
 if(e.op!==0||e.b!==0||!Number.isInteger(e.a)||e.a<0)return false;
 if(e.tag===0)return optional&&e.a===0;
 if(e.tag===1)return nullable&&e.a===0;
 if(field.kind==='string')return e.tag===5&&e.a<graph.strings.length;
 if(['record','array','map'].includes(field.kind))return e.tag===6&&e.a<limit;
 return false;
}
function expression(e,s,count,limit,graph){
 if(e.op!==s.op)return false;
 if(s.op===0)return atom(e,limit,graph,s.field,s.undefined,s.nullable);
 if(e.tag!==s.tag||e.b!==s.b)return false;
 if(s.op===2)return e.tag===6&&Number.isInteger(e.a)&&e.a>=0&&e.a+(count-1)*e.b<limit;
 if(e.a!==s.a)return false;
 if(s.op===5)return Number.isSafeInteger(e.a+Math.max(0,count-1)*e.b)&&e.a+Math.max(0,count-1)*e.b<=0xffffffff;
 const col=graph.base.columns[e.a];if(!col||count+e.b>col.length)return false;
 if(COLUMN_TAGS[e.a]===6)for(let i=0;i<count;i++)if(col.loAt(i+e.b)>=limit)return false;
 return true;
}
function eligible(bridge,graph){
 if(bridge.objects.length||graph.version!==1||!graph.base.originalStrings||graph.strings!==graph.base.originalStrings||graph.overrides.size||graph.rootWords[0]!==6||graph.rootWords[2]!==0||graph.rootWords[1]!==graph.nodeCount-1)return false;
 // Existing generic fresh writes use assignment. Any inherited descriptor on
 // a compiled field keeps that exact setter/readonly behavior on the old path.
 for(const key of OBJECT_KEYS)if(Object.getOwnPropertyDescriptor(Object.prototype,key))return false;
 const reg=bridge.registration;if(reg.schema.id!==SCHEMA.id)return false;
 let same=schemaMatch.get(reg);if(same===undefined){same=JSON.stringify(reg.schema)===SCHEMA_JSON;schemaMatch.set(reg,same);}if(!same)return false;
 const groups=graph.base.groups,columns=graph.base.columns;if(groups.length!==SHAPES.length||columns.length!==COLUMN_TAGS.length)return false;
 // Examine source columns once before creating any object. Unsupported numeric
 // representations conservatively retain the existing generic publication.
 for(let c=0;c<columns.length;c++){const column=columns[c],tag=COLUMN_TAGS[c];for(let i=0;i<column.length;i++){if(column.tagAt(i)!==tag)return false;const lo=column.loAt(i),hi=column.hiAt(i);if(tag===4){const n=number(lo,hi);if(!Number.isFinite(n)||Object.is(n,-0))return false;}else if(hi!==0||(tag===5&&lo>=graph.strings.length)||(tag===6&&lo>=graph.nodeCount))return false;}}
 let end=0;
 for(let n=0;n<groups.length;n++){
  const g=groups[n],s=SHAPES[n];if(g.first!==end||!Number.isInteger(g.count)||g.count<1||g.kind!==s.kind||g.hi!==0||g.properties.length!==s.properties.length||g.dynamic.length!==s.dynamic.length||g.collections.length!==s.collections.length)return false;end+=g.count;if(end>graph.nodeCount)return false;
  if(g.kind===0&&g.lo!==0)return false;
  if(g.kind!==0&&g.count!==1)return false;
  if(g.kind===1&&(g.lo!==s.prototype||!reg.prototypes.has(g.lo)))return false;
  if(g.kind===3&&g.lo!==0)return false;
  let properties=0,items=0;
  for(let j=0;j<s.properties.length;j++){const p=g.properties[j],q=s.properties[j];if(p.count!==1||p.flags!==q.flags||p.key.op!==0||p.key.tag!==5||p.key.b!==0||graph.strings[p.key.a]!==q.name||!expression(p.value,q.value,g.count,g.first,graph))return false;properties++;}
  for(let j=0;j<s.dynamic.length;j++){const p=g.dynamic[j],q=s.dynamic[j];if(p.flags!==7||p.count!==g.lo||!expression(p.key,q.key,p.count,g.first,graph)||!expression(p.value,q.value,p.count,g.first,graph))return false;const col=columns[p.key.a];for(let i=0;i<p.count;i++)if(graph.strings[col.loAt(i+p.key.b)]!==String(i))return false;properties+=p.count;}
  for(let j=0;j<s.collections.length;j++){const c=g.collections[j],q=s.collections[j];if(!expression(c.key,q.key,c.count,g.first,graph)||!expression(c.value,q.value,c.count,g.first,graph))return false;items+=c.count*2;}
  if(properties!==g.propertyCount||items!==g.itemAtoms)return false;
 }
 return end===graph.nodeCount;
}
function literal(e,bridge){if(e.tag===0)return undefined;if(e.tag===1)return null;if(e.tag===5)return bridge.strings[e.a];return bridge.objects[e.a];}
function publish(bridge,object){const id=bridge.objects.length;bridge.objects.push(object);bridge.ids.set(object,id);bridge.registry.owners.set(object,bridge);}
`;
const generated=`// GENERATED by scripts/generate-source-initializer.mjs from native schema/recipe.\n// No runtime code generation. Keep generic reconciliation for every ineligible layout.\nconst SCHEMA=${JSON.stringify(schema)};\nconst SCHEMA_JSON=JSON.stringify(SCHEMA);\nconst SHAPES=${JSON.stringify(groups)};\nconst COLUMN_TAGS=${JSON.stringify([...Array(columnKinds.size)].map((_,i)=>columnKinds.get(i)))};\nconst OBJECT_KEYS=${JSON.stringify([...new Set(groups.filter(g=>g.kind===0).flatMap(g=>g.properties.map(p=>p.name)))])};\n${helpers}\nexport function tryInitializeSourceLayout(bridge,graph){\n  if(!eligible(bridge,graph))return false;\n  bridge.graph=graph;bridge.strings=graph.strings.slice();bridge.stringIds=new Map();\n  for(let id=0;id<bridge.strings.length;id++)bridge.stringIds.set(bridge.strings[id],id);\n  const strings=bridge.strings,objects=bridge.objects,groups=graph.base.groups;\n  ${base.columns.map((_,i)=>`const c${i}=graph.base.columns[${i}];`).join('')}\n${body}  bridge.root=objects[graph.rootWords[1]];return true;\n}\n`;
await writeFile(resolve(outputPath),generated);
