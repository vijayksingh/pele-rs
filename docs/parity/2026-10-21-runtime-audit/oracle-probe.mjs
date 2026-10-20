// Read-only public oracle reproduction. Run from pele-rs root.
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {readFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
const root=resolve(process.argv[2]??'.');
const file=resolve(root,'node_modules/pele/dist/pele.js');
const hash=createHash('sha256').update(await readFile(file)).digest('hex');
if(hash!=='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d')throw new Error('Wrong pinned Pele bundle');
const pele=await import(pathToFileURL(file));
const source='radar-beta\naxis a["Alpha"], b["Beta"], c["Gamma"]\ncurve one["One"] {1,2,3}\ncurve two["Two"] {3,2,1}';
const options={measurer:{width:(text,size)=>text.length*size*.6}};
const cases=[];
for(const name of ['info-surrogate','radar-hole','radar-empty-circle','radar-empty-polygon']){
 const model=pele.parse(name==='info-surrogate'?'info':source);
 if(name==='info-surrogate')model.title='\ud800';
 else if(name==='radar-hole')delete model.axes[1];
 else {model.axes=[];model.curves=[model.curves[0]];model.curves[0].entries=[];if(name.endsWith('polygon'))model.options.graticule='polygon';}
 try{const r=pele.render(model,options);cases.push({name,ok:true,width:r.width,height:r.height,svg:r.svg,svgCodeUnits:Array.from({length:r.svg.length},(_,i)=>r.svg.charCodeAt(i))});}
 catch(e){cases.push({name,ok:false,error:{name:e?.name,message:e?.message}});}
}
const fixture=JSON.parse(await readFile(resolve(root,'fixtures/pele/diagram-model-construction.json'),'utf8'));
const sources=new Map();for(const c of fixture.cases)if(!sources.has(c.family))sources.set(c.family,c.source);
const constructors=[];
for(const [family,source] of sources){
 const model=pele.parse(source),prototype=Object.getPrototypeOf(model);
 if(prototype===Object.prototype){constructors.push({family,plainObject:true});continue;}
 try{const value=new prototype.constructor();constructors.push({family,ok:true,samePrototype:Object.getPrototypeOf(value)===prototype,ownKeys:Object.getOwnPropertyNames(value)});}
 catch(e){constructors.push({family,ok:false,error:{name:e.name,message:e.message}});}
}
console.log(JSON.stringify({oracleSHA256:hash,cases,constructors},null,2));
