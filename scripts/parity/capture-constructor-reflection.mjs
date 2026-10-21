// Actual pinned class reflection, with separate monolithic and ESM artifacts.
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
const hash=p=>createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const load=p=>import(pathToFileURL(path.resolve(p)));
const corpus=JSON.parse(fs.readFileSync('fixtures/pele/diagram-entries.json'));
if(hash('node_modules/pele/dist/pele.js')!==corpus.oracleSha256)throw Error('Oracle pin mismatch');
const mono=await load('node_modules/pele/dist/pele.js'),core=await load('node_modules/pele/dist/esm/core.js');
const core2=await import(pathToFileURL(path.resolve('node_modules/pele/dist/esm/core.js')).href+'?reflectionSecondRegistry');
const atom=x=>typeof x==='function'?{kind:'function',name:x.name,length:x.length,keys:Reflect.ownKeys(x).map(String)}:x===undefined?{kind:'undefined'}:x===null?null:typeof x==='object'?{kind:'object'}:x;
const descriptor=(o,k)=>{const d=Object.getOwnPropertyDescriptor(o,k);return {key:String(k),enumerable:d.enumerable,configurable:d.configurable,...('value'in d?{writable:d.writable,value:atom(d.value)}:{get:atom(d.get),set:atom(d.set)})};};
const withoutNew=C=>{try{Reflect.apply(C,undefined,[]);return {threw:false};}catch(error){return {threw:true,name:error.name,message:error.message};}};
const inferred=C=>({[C.name]:class{}})[C.name];
const describe=C=>({name:C.name,length:C.length,constructor:Reflect.ownKeys(C).map(k=>descriptor(C,k)),prototype:Reflect.ownKeys(C.prototype).map(k=>descriptor(C.prototype,k)),parentObject:Object.getPrototypeOf(C.prototype)===Object.prototype});
const rows=[],identities=[],classes=new Map();
function row(family,prototype,models){
 const [m,c,d,r]=models.map(x=>Object.getPrototypeOf(x).constructor);
 if(m===Object)return;
 if(c!==d||c!==r||m===r)throw Error('Changed pinned artifact identity');
 const identity=prototype===2?'class-member':family==='swimlane'?'flowchart':family;
 rows.push({family,prototype,identity,monolithName:m.name,moduleName:r.name,monolith:describe(m),module:describe(r),withoutNew:{monolith:withoutNew(m),module:withoutNew(r),inferredMonolith:withoutNew(inferred(m)),inferredModule:withoutNew(inferred(r))}});
 classes.set(family+'/'+prototype,[m,r]);
 identities.push({family,prototype,esmCoreSecondAndRawSame:true,monolithDistinct:true});
}
for(const meta of corpus.entries){
 const c=corpus.cases.find(c=>c.family===meta.type),ref=(await load('node_modules/pele/dist/esm/diagrams/'+meta.type+'.js')).default;core.register(ref);core2.register(ref);
 row(meta.type,1,[mono.parse(c.source),core.parse(c.source),core2.parse(c.source),ref.parse(c.source,c.config,c.title??undefined,c.lineOffset,{maxEdges:undefined})]);
 if(meta.type==='class'){
  const source='classDiagram\nclass A {\n+method() String\n}';
  row('class',2,[mono.parse(source),core.parse(source),core2.parse(source),ref.parse(source,{},undefined,0)].map(m=>m.classes.get('A').methods[0]));
 }
}
const flow=classes.get('flowchart/1'),swim=classes.get('swimlane/1');
if(!flow.every((C,i)=>C===swim[i]))throw Error('Changed Flow/Swim shared class identity');
if(rows.length!==23)throw Error('Changed pinned constructor count');
const output=process.argv[2]??'fixtures/public-constructor-reflection.json';
fs.writeFileSync(output,JSON.stringify({oracleSHA256:corpus.oracleSha256,captureScriptSHA256:hash(new URL(import.meta.url)),rows,identities,flowSwimSharedInBothArtifacts:true},null,2)+'\n');
console.log(JSON.stringify({classes:rows.length,outputSHA256:hash(output)}));
