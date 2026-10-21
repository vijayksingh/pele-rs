// Pinned public railroad output at rounding and inline-storage boundaries.
import assert from 'node:assert/strict';
import {readFile,writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
const bytes=await readFile('node_modules/pele/dist/pele.js');
const sha=createHash('sha256').update(bytes).digest('hex');
assert.equal(sha,'d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d');
const {render}=await import('../../node_modules/pele/dist/pele.js');
const bits=n=>{const b=Buffer.alloc(8);b.writeDoubleBE(n);return b.toString('hex')};
const adjacent=(n,offset)=>{const b=Buffer.alloc(8);b.writeDoubleBE(n);b.writeBigUInt64BE(b.readBigUInt64BE()+BigInt(offset));return b.readDoubleBE()};
const source='railroad-beta\nrule = terminal("x");';
const values=[0,-0,...[.005,-.005,1.005,-1.005,999999999.995,-999999999.995,1e9,-1e9].flatMap(n=>[adjacent(n,-1),n,adjacent(n,1)]),1e21,-1e21,NaN,Infinity,-Infinity];
const cases=values.map((padding,index)=>{const options={fontSize:13,maxWidth:220,padding,now:1704067200000,measurer:{width:(text,size,style)=>text.length*size*((style&1)?.71:.6)}};const result=render(source,options);assert(result.svg.length<64*1024);const x=result.svg.match(/class="pele-rule-name" x="([^"]*)"/)?.[1];assert.notEqual(x,undefined);return{id:`padding/${index}`,paddingBits:bits(padding),ruleNameX:x,result,widthBits:bits(result.width),heightBits:bits(result.height)}});
await writeFile('fixtures/pele/svg-number-boundaries.json',JSON.stringify({oracleRevision:'458322a5b8abb1ae57ddf127c32017ab9fb14a92',oracleSHA256:sha,source,cases},null,2)+'\n');
console.log(`${cases.length} pinned public boundary cases`);
