import {test,expect} from '@playwright/test';
import {writeFile} from 'node:fs/promises';

test('ER error data retains brands, diagnostics, aliases and original callback throws',async({page},testInfo)=>{
  await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const rows=[];
    for(const [mode,path]of [
      ['control','/dist/evidence/pele-oracle.js'],
      ['candidate','/dist/package-web/package/full.js'],
    ]){
      const api=await import(path),engine=mode==='control'?api:await api.loadRenderer();
      const entry=mode==='control'?(await import('/dist/evidence/pele-esm/diagrams/er.js')).default:null;
      const source='erDiagram\nA { int aid PK }\nB { string bid FK }\nA ||--o{ B : owns';
      const constructors=[Error,api.PeleError,TypeError,RangeError,SyntaxError,ReferenceError,EvalError,URIError];
      try{
        for(const Ctor of constructors){
          const model=engine.parse(source),error=new Ctor('retained error'),prototype=Object.getPrototypeOf(error);
          let diagnostics=0;
          Object.defineProperty(error,'sourceURL',{get(){diagnostics++;throw new Error('diagnostics getter must stay unread');},configurable:true});
          Object.defineProperty(error,'hidden',{value:model.entities.get('B'),writable:false,enumerable:false,configurable:true});
          error.self=error;error.map=new Map([[error,model]]);error.set=new Set([error]);model.extension=error;
          const before=Object.getOwnPropertyDescriptors(error);
          const options={measurer:{width:(text,size)=>text.length*size*.6}};
          const first=engine.render(model,options);
          model.addEntity('C');
          const second=engine.render(model,options);
          // The same error is also a caller-owned root. Public model fields are
          // copied as descriptors; all nested objects keep their identities.
          const raw=new Ctor('caller root');
          for(const name of Object.getOwnPropertyNames(model))Object.defineProperty(raw,name,Object.getOwnPropertyDescriptor(model,name));
          const rawResult=entry?entry.render(raw,{},options):engine.renderRaw('er',raw,{},options);
          let caught;
          try{engine.render(model,{measurer:{width(){throw error;}}});}catch(value){caught=value;}
          const repaired=engine.render(model,options);
          const after=Object.getOwnPropertyDescriptors(error);
          const sameDescriptors=Reflect.ownKeys(before).length===Reflect.ownKeys(after).length&&Reflect.ownKeys(before).every(key=>{
            const a=before[key],b=after[key];return b&&a.value===b.value&&a.get===b.get&&a.set===b.set&&a.writable===b.writable&&a.enumerable===b.enumerable&&a.configurable===b.configurable;
          });
          rows.push({mode,brand:error.name,first,second,rawResult,repaired,diagnostics,sameDescriptors,
            samePrototype:Object.getPrototypeOf(error)===prototype,original: caught===error,
            caught:{name:caught?.name,message:caught?.message,code:caught?.code},
            aliases:error.self===error&&error.map.get(error)===model&&error.set.has(error)&&error.hidden===model.entities.get('B')});
          engine.dispose?.(raw);engine.dispose?.(model);
        }
      }finally{engine.close?.();}
    }
    return rows;
  });
  const path=testInfo.outputPath('er-error-brands.json');await writeFile(path,JSON.stringify(report,null,2));
  await testInfo.attach('er-error-brands.json',{path,contentType:'application/json'});
  for(const row of report){
    expect(row.diagnostics,row.brand).toBe(0);
    for(const key of ['sameDescriptors','samePrototype','aliases'])expect(row[key],row.brand+'/'+key).toBe(true);
    // Both public wrappers translate resource RangeError into a Pele limit
    // error. Other thrown errors preserve the caller's original identity.
    expect(row.original,row.brand).toBe(row.brand!=='RangeError');
    expect(row.repaired).toEqual(row.second);
  }
  expect(report.filter(row=>row.mode==='candidate').map(({mode,...row})=>row))
    .toEqual(report.filter(row=>row.mode==='control').map(({mode,...row})=>row));
});
