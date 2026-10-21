// Whole public SVG output; no private SVG helper is called by these fixtures.
import assert from 'node:assert/strict';
import {readFile,writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
const oracleSHA256=createHash('sha256').update(await readFile('node_modules/pele/dist/pele.js')).digest('hex');
assert.equal(oracleSHA256,'d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d');
const {render}=await import('../../node_modules/pele/dist/pele.js');
const radius=' rx="var(--pele-radius,4px)"';
const source='flowchart TB\nA@{ icon: "fixture", form: "square", label: "Root output" }';
const markups=[
 ['plain','<path d="M0,0"/>'],
 ['rounded',`<rect x="1"${radius}/>`],
 ['paired-tag',`<rect${radius}></rect>`],
 ['existing-style',`<rect style="fill:red"${radius}/>`],
 ['later-style',`<rect${radius} style="fill:red"/>`],
 ['authored-radius-first',`<rect rx="8"${radius}/>`],
 ['authored-radius-last',`<rect${radius} rx="8"/>`],
 ['duplicate-default',`<rect${radius}${radius}/>`],
 ['multiple',`<rect${radius}/><rect/><rect style=""${radius}>x</rect>`],
 ['nonrect',`<rectangle${radius}/><rect_${radius}/><RECT${radius}/>`],
 ['ascii-boundary',`<rect-a${radius}/><rect1${radius}/>`],
 ['unicode-boundary',`<recté${radius}/><rect漢${radius}/>`],
 ['unicode-content',`<g data-label="漢😀"><rect${radius}/></g>`],
 ['truncated',`<rect${radius}`],
 ['weight-aliases','<path data-a="var(--_r) var(--_tw) var(--_hw) var(--_w)"/>'],
 ['alias-removed-with-radius',`<rect${radius}/>`],
 ['unrelated-default',`<path${radius}/><rect data-a="x"/>`],
 ['style-substring',`<rect data-a=' style="x'${radius}/>`],
 ['formed-style',`<rect st${radius}yle="fill:red"/>`],
 ['formed-title-alias',`<rect data-a="var(--_t${radius}w)"/>`],
 ['formed-heading-alias',`<rect data-a="var(--_h${radius}w)"/>`],
 ['formed-bold-alias',`<rect data-a="var(--_${radius}w)"/>`],
 ['right-angle-in-attribute',`<rect data-a=">"${radius}/><rect${radius}/>`],
];
const variants=[
 ['defaults',{},source],
 ['fixed-size',{responsive:false},source],
 ['accessible',{idPrefix:'root<&"😀'},'flowchart TB\naccTitle: Root &amp; title\naccDescr: Description &lt; detail\nA@{ icon: "fixture", form: "square", label: "Root output" }'],
 ['fonts',{fontFamily:'"A Font", serif',fontFamilyMono:'"Code Font", monospace'},source],
 ['invalid-fonts',{fontFamily:'bad; font',fontFamilyMono:'bad()'},source],
];
const cases=[];
for(const [id,markup] of markups)for(const [variant,options,text] of variants){
 const result=render(text,{...options,now:1704067200000,measurer:{width:(t,z,s)=>t.length*z*(s&1?.71:.6)},icons:()=>markup});
 assert(result.svg.includes(markup)||result.svg.includes('data-icon="fixture"'),id);
 cases.push({id:`${id}/${variant}`,source:text,options,markup,result});
}
await writeFile('fixtures/pele/svg-root.json',JSON.stringify({oracleRevision:'458322a5b8abb1ae57ddf127c32017ab9fb14a92',oracleSHA256,cases},null,2)+'\n');
console.log(`${cases.length} pinned public SVG root cases`);
