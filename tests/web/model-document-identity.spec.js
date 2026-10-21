import {test,expect} from '@playwright/test';
import {writeFile} from 'node:fs/promises';

test('shared public objects retain original helper ownership and document-local aliases',async({page},info)=>{
 await page.goto('/examples/web/');
 const report=await page.evaluate(async packageRoot=>{
  const {loadRenderer}=await import(packageRoot+'/full.js'),oracle=await import('/dist/evidence/pele-oracle.js');
  const engine=await loadRenderer(),sources=['sankey\nA,B,5\nB,C,3','sankey\nX,Y,2\nY,Z,1'];
  const actual=sources.map(source=>engine.parse(source)),expected=sources.map(source=>oracle.parse(source));
  const rows=[],checks=[],options={measurer:{width(text,size){return text.length*size*.6;}}};
  const state=models=>models.map(model=>({nodes:model.nodes.map(node=>({id:node.id,index:node.index,keys:Reflect.ownKeys(node)})),lookup:[...model.lookup].map(([key,node])=>[key,model.nodes.indexOf(node)]),shared:model.shared===models[0].nodes[0],peer:model.peer===models[1-models.indexOf(model)]}));
  const compare=stage=>{rows.push({stage,actual:engine.render(actual[1],options),expected:oracle.render(expected[1],options),actualState:state(actual),expectedState:state(expected)});};
  try{
   for(const models of [actual,expected]){
    models[1].shared=models[0].nodes[0];
    models[1].peer=models[0];models[0].peer=models[1];
   }
   const held=actual[1].shared,helper=actual[1].findOrCreateNode;
   compare('capture foreign root, leaf and cycle');
   for(let iteration=0;iteration<3;iteration++){
    const label='Owned '+iteration,value=helper.call(actual[0],label),reference=expected[1].findOrCreateNode.call(expected[0],label);
    checks.push(value===actual[0].nodes.at(-1)&&value===actual[0].lookup.get(label)&&!actual[1].lookup.has(label)&&reference===expected[0].nodes.at(-1));
    compare('original owner '+iteration);
   }
   engine.dispose(actual[0]);
   const value=helper.call(actual[0],'After disposal'),reference=expected[1].findOrCreateNode.call(expected[0],'After disposal');
   checks.push(value===actual[0].nodes.at(-1)&&value===actual[0].lookup.get('After disposal')&&!actual[1].lookup.has('After disposal')&&reference===expected[0].nodes.at(-1));
   checks.push(actual[1].shared===held&&actual[1].peer===actual[0]&&actual[0].peer===actual[1]);
   compare('disposed receiver uses live document without changing aliases');
   engine.dispose(actual[1]);
   let error;try{helper.call(actual[0],'No owner');}catch(value){error={name:value.name,message:value.message};}
   return{rows,checks,error};
  }finally{for(const model of actual)engine.dispose(model);engine.close();}
 },process.env.PELE_TEST_PACKAGE_ROOT??'/dist/package-web/package');
 const path=info.outputPath('model-document-identity.json');await writeFile(path,JSON.stringify(report,null,2));await info.attach('model-document-identity.json',{path,contentType:'application/json'});
 expect(report.checks).toEqual(Array(5).fill(true));
 expect(report.error).toEqual({name:'TypeError',message:'Public method has no live document'});
 expect(report.rows).toHaveLength(5);
 for(const row of report.rows){expect(row.actual,row.stage).toEqual(row.expected);expect(row.actualState,row.stage).toEqual(row.expectedState);}
});
