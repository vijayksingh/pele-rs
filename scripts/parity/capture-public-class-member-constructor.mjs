// Pinned ClassMember constructor, declared inputs and virtual parser dispatch.
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
const sha=x=>createHash('sha256').update(x).digest('hex');
const pin='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
if(sha(fs.readFileSync('node_modules/pele/dist/pele.js'))!==pin)throw Error('Oracle pin mismatch');
const entry=(await import(pathToFileURL(path.resolve('node_modules/pele/dist/esm/diagrams/class.js')))).default;
const seed=entry.parse('classDiagram\nclass A {\n+method() String\n}',{},undefined,0);
const C=seed.classes.get('A').methods[0].constructor;
const normalize=x=>x===undefined?{undefined:true}:x;
const fields=o=>Object.getOwnPropertyNames(o).map(k=>{const d=Object.getOwnPropertyDescriptor(o,k);return[k,normalize(d.value),(d.enumerable?1:0)|(d.writable?2:0)|(d.configurable?4:0)];});
const cases=[];
for(const [input,kind] of [['+name','attribute'],['+run(x, y) String','method'],['+notAMethod','method'],[' + leading  ','attribute'],['#value$','attribute'],['~méthod(😀) Result*','method'],['','attribute']]){
 const member=new C(input,kind, new Proxy({},{get(){throw Error('Unused third argument read');}}));
 cases.push({input,kind,fields:fields(member),display:member.getDisplayDetails(),classifier:member.parseClassifier()});
}
const virtual=[];
for(const mode of ['method-true','method-false','attribute','method-throw','attribute-throw']){
 const trace=[],sentinel={original:mode};let escaped;
 class Derived extends C {
  parseMethod(input){escaped=this;trace.push(['parseMethod',input,fields(this)]);if(mode==='method-throw')throw sentinel;this.id=' + custom ';return mode==='method-true';}
  parseAttribute(input){escaped=this;trace.push(['parseAttribute',input,fields(this)]);if(mode==='attribute-throw')throw sentinel;this.id=' attribute ';}
 }
 const kind=mode.startsWith('attribute')?'attribute':'method';
 try{const member=new Derived('+input()',kind);virtual.push({mode,kind,trace,ok:true,fields:fields(member),instance:member instanceof Derived});}
 catch(error){virtual.push({mode,kind,trace,ok:false,original:error===sentinel,fields:fields(escaped)});if(error!==sentinel)throw error;}
}
if(virtual.filter(r=>!r.ok).length!==2)throw Error('Missing original-throw proof');
const report={oracleSHA256:pin,scriptSHA256:sha(fs.readFileSync(new URL(import.meta.url))),classChunkSHA256:sha(fs.readFileSync('node_modules/pele/dist/esm/chunks/KV3E653F.js')),cases,virtual};
fs.writeFileSync(process.argv[2]??'fixtures/public-class-member-constructor.json',JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify({declared:cases.length,virtual:virtual.length,originalThrows:virtual.filter(r=>r.original).length}));
