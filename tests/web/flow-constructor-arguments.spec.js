import {test,expect} from '@playwright/test';
import {readFile,writeFile} from 'node:fs/promises';
const driver=await readFile(new URL('./support/flow-constructor-feature.js',import.meta.url),'utf8');
const ganttDriver=await readFile(new URL('./support/gantt-constructor-feature.js',import.meta.url),'utf8');
const packageRoot=process.env.PELE_TEST_PACKAGE_ROOT??'/dist/package-web/package';
async function record(info,name,value){const path=info.outputPath(name);await writeFile(path,JSON.stringify(value,null,2));await info.attach(name,{path,contentType:'application/json'});}

// Exact diagnostic tests below remain unchanged and expose known browser wording gaps.
// Semantic companions allow ONLY these recorded TypeError message pairs. Every
// other value, error kind/identity, state, descriptor, trace and SVG stays exact.
const recordedWordingPairs = [
  [
    "can't access property \"warn\", bridge.value(...) is null",
    "can't access property \"warn\", this.options is null"
  ],
  [
    "can't access property \"warn\", bridge.value(...) is undefined",
    "can't access property \"warn\", this.options is undefined"
  ],
  [
    "e is not a function",
    "e is not a function. (In 'e()', 'e' is 3)"
  ],
  [
    "e is not a function",
    "e is not a function. (In 'e()', 'e' is null)"
  ],
  [
    "null is not an object (evaluating 'bridge.value(operation.receiver)[operation.key]')",
    "null is not an object (evaluating 'this.options.warn')"
  ],
  [
    "this.options.warn is not a function",
    "this.options.warn is not a function. (In 'this.options.warn(e())', 'this.options.warn' is an instance of Object)"
  ],
  [
    "this.options.warn is not a function",
    "this.options.warn is not a function. (In 'this.options.warn(e())', 'this.options.warn' is false)"
  ],
  [
    "this.options.warn is not a function",
    "this.options.warn is not a function. (In 'this.options.warn?.(`Style applied to unknown node \"${s}\". This may indicate a typo. The node will be created automatically.`)', 'this.options.warn' is an instance of Object)"
  ],
  [
    "this.options.warn is not a function",
    "this.options.warn is not a function. (In 'this.options.warn?.(`Style applied to unknown node \"${s}\". This may indicate a typo. The node will be created automatically.`)', 'this.options.warn' is false)"
  ],
  [
    "undefined is not an object (evaluating 'bridge.value(operation.receiver)[operation.key]')",
    "undefined is not an object (evaluating 'this.options.warn')"
  ]
];
function semanticEqual(actual,expected,label){
  const copied=structuredClone(actual);
  function visit(a,b){
    if(!a||!b||typeof a!=='object'||typeof b!=='object')return;
    if(a.name==='TypeError'&&b.name==='TypeError'&&a.message!==b.message&&
       recordedWordingPairs.some(([left,right])=>a.message===left&&b.message===right))a.message=b.message;
    for(const key of Object.keys(a))visit(a[key],b[key]);
  }
  visit(copied,expected);expect(copied,label).toEqual(expected);
}
for(const semantic of [false,true]){
const title=name=>semantic?'semantic construction parity: '+name:name;
const equal=(actual,expected,label)=>semantic?semanticEqual(actual,expected,label):expect(actual,label).toEqual(expected);
// Functional construction/host effects, not known name or realm identity gaps.
test(title('Flow constructors keep options opaque through ordered host reads, reentry and original throws'),async({page},info)=>{
  test.setTimeout(120000);await page.goto('/examples/web/');
  const report=await page.evaluate(async({source,packageRoot})=>{
    const url=URL.createObjectURL(new Blob([source],{type:'text/javascript'}));
    try{
      const {runFlowCases,fixtureCaseIds}=await import(url),corpus=await(await fetch('/fixtures/pele/diagram-entries.json')).json(),fixture=await(await fetch('/fixtures/public-flow-arguments.json')).json();
      const {loadDiagramEntries,disposeDiagramModel}=await import(location.origin+packageRoot+'/diagram-entries.js');await loadDiagramEntries();
      const rows=[];
      for(const family of ['flowchart','swimlane']){
        const input=corpus.cases.find(x=>x.family===family),candidate=(await import(location.origin+packageRoot+'/diagrams/'+family+'.js')).default,oracle=(await import(location.origin+'/dist/evidence/pele-esm/diagrams/'+family+'.js')).default;
        rows.push({family,candidate:runFlowCases(candidate,family,input,disposeDiagramModel),oracle:runFlowCases(oracle,family,input),fixture:fixture.cases.filter(x=>x.family===family),fixtureCaseIds});
      }return rows;
    }finally{URL.revokeObjectURL(url);}
  },{source:driver,packageRoot});
  await record(info,'flow-constructor-arguments.json',report);
  for(const row of report){
    equal(row.candidate,row.oracle,row.family);
    for(const c of row.candidate){expect(c.constructorReads,c.id).toBe(0);expect(c.sameOptions,c.id).toBe(true);expect(c.returnReads,c.id).toBe(0);expect(c.descriptorPreserved,c.id).toBe(true);}
    for(const fixture of row.fixture){const actual=row.candidate.find(x=>x.id===fixture.id);const {returnReads,descriptorPreserved,render,...base}=actual;expect(base,row.family+'/'+fixture.id).toEqual(fixture);}
  }
});

test(title('Flow options history, borrowed receivers and captured constructors survive registry close'),async({page},info)=>{
  test.setTimeout(120000);await page.goto('/examples/web/');
  const rows=await page.evaluate(async({source,packageRoot})=>{
    const url=URL.createObjectURL(new Blob([source],{type:'text/javascript'}));
    try{
      const {runFlowLifetimes}=await import(url),corpus=await(await fetch('/fixtures/pele/diagram-entries.json')).json();
      const {loadRegistry}=await import(location.origin+packageRoot+'/registry.js'),{loadDiagramEntries,disposeDiagramModel}=await import(location.origin+packageRoot+'/diagram-entries.js');await loadDiagramEntries();const rows=[];
      for(const family of ['flowchart','swimlane']){
        const input=corpus.cases.find(x=>x.family===family),candidate=(await import(location.origin+packageRoot+'/diagrams/'+family+'.js')).default,oracle=(await import(location.origin+'/dist/evidence/pele-esm/diagrams/'+family+'.js')).default;
        const registry=await loadRegistry();registry.register(candidate);const parsed=registry.parse(input.source),C=parsed.constructor;
        registry.dispose(parsed);registry.register({type:family,parse(){throw Error('Replacement');},render(){throw Error('Replacement');}});registry.close();
        const original=oracle.parse(input.source,input.config,undefined,input.lineOffset,{maxEdges:undefined}),O=original.constructor;
        rows.push({family,candidate:runFlowLifetimes(candidate,family,input,C,disposeDiagramModel),oracle:runFlowLifetimes(oracle,family,input,O)});
      }return rows;
    }finally{URL.revokeObjectURL(url);}
  },{source:driver,packageRoot});
  await record(info,'flow-constructor-lifetimes.json',rows);
  for(const row of rows){equal(row.candidate,row.oracle,row.family);expect(row.candidate.independent).toBe(true);expect(row.candidate.subclass).toEqual({prototype:true,marker:'child',node:true});}
});


test(title('Gantt warning reads current options twice, retains selected callee and preserves lazy producer failures'),async({page},info)=>{
  test.setTimeout(120000);await page.goto('/examples/web/');
  const report=await page.evaluate(async({source,packageRoot})=>{
    const url=URL.createObjectURL(new Blob([source],{type:'text/javascript'}));
    try{
      const {runGanttCases,runGanttRetainedCases}=await import(url),{loadDiagramEntries,disposeDiagramModel}=await import(location.origin+packageRoot+'/diagram-entries.js');await loadDiagramEntries();
      const candidate=(await import(location.origin+packageRoot+'/diagrams/gantt.js')).default,oracle=(await import(location.origin+'/dist/evidence/pele-esm/diagrams/gantt.js')).default;
      return{candidate:runGanttCases(candidate,disposeDiagramModel),oracle:runGanttCases(oracle),retainedCandidate:runGanttRetainedCases(candidate,disposeDiagramModel),retainedOracle:runGanttRetainedCases(oracle),fixture:await(await fetch('/fixtures/public-gantt-warning-arguments.json')).json()};
    }finally{URL.revokeObjectURL(url);}
  },{source:ganttDriver,packageRoot});
  await record(info,'gantt-warning-arguments.json',report);
  equal(report.candidate,report.oracle);
  equal(report.retainedCandidate,report.retainedOracle);
  for(const row of report.retainedCandidate)expect(row.reads,row.name).toBe(0);
  expect(report.candidate).toEqual({initial:report.fixture.initial,drawing:report.fixture.drawing,cases:report.fixture.cases});
});

}
