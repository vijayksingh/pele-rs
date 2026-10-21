import {test,expect} from '@playwright/test';
import {writeFile} from 'node:fs/promises';

test('ER resource callbacks preserve live state, captured references and nested public operations',async({page})=>{
  await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js');
    const oracle=await import('/dist/evidence/pele-oracle.js'),engine=await loadRenderer();
    const source='erDiagram\nA { int aid PK\n string first }\nB { int bid PK\n string second }\nC { int cid PK\n string future }\nA ||--o{ B : owns\nB ||--o{ C : uses';
    const variants=['none','cycle','future-alias','future-label','delete-entity','delete-attribute','attribute-name','same-row-name','replace-attributes','detached-attributes','styles','replace-styles','detached-styles','direction','title-accessibility','throw-cycle','throw-undefined','throw-null','throw-false','throw-object','throw-symbol','nested-existing','nested-new','nested-render','nested-render-edited-attribute','nested-render-throw','source-render-control','cross-document-control','borrowed-helper','late-icons-metadata'];
    const number=value=>{const buffer=new ArrayBuffer(8),view=new DataView(buffer);view.setFloat64(0,value);return view.getBigUint64(0).toString(16).padStart(16,'0');};
    const signature=value=>{
      const ids=new Map(),pending=[],nodes=[];
      const atom=v=>{
        if(v===undefined)return ['undefined'];if(v===null)return null;
        if(typeof v==='number')return ['number',number(v)];
        if(typeof v==='function')return ['function',v.name,v.length];
        if(typeof v==='symbol')return ['symbol',String(v)];
        if(typeof v!=='object')return v;
        if(!ids.has(v)){ids.set(v,pending.length);pending.push(v);}return ['ref',ids.get(v)];
      };
      const root=atom(value);
      for(let i=0;i<pending.length;i++){
        const v=pending[i],kind=Array.isArray(v)?'array':v instanceof Map?'map':v instanceof Set?'set':v instanceof Date?'date':'object';
        nodes.push({kind,properties:Object.getOwnPropertyNames(v).map(k=>{const d=Object.getOwnPropertyDescriptor(v,k);return [k,d.enumerable,!!d.writable,d.configurable,'value'in d?atom(d.value):['accessor']];}),...(kind==='map'?{entries:[...v].map(([k,x])=>[atom(k),atom(x)])}:{}),...(kind==='set'?{values:[...v].map(atom)}:{}),...(kind==='date'?{time:number(v.getTime())}:{})});
      }
      return JSON.stringify({root,nodes});
    };
    const basicWidth=(text,size,style)=>text.length*size*((style&1)?.71:.6);
    const rows=[];
    for(const variant of variants){
      const row={variant};
      for(const [name,renderer]of [['js',oracle],['wasm',engine]]){
        const text=variant==='direction'?source.replace('erDiagram\n','erDiagram\ndirection LR\n'):source;
        const model=renderer.parse(text),other=renderer.parse(source),saved=model.entities.get('C'),oldAttributes=saved.attributes,oldStyles=saved.cssStyles,firstRow=model.entities.get('A').attributes[0];
        const prototype=Object.getPrototypeOf(model),extension={value:'callback-owned',self:null};extension.self=extension;
        const sentinel=variant==='throw-undefined'?undefined:variant==='throw-null'?null:variant==='throw-false'?false:variant==='throw-object'?{message:'opaque callback'}:variant==='throw-symbol'?Symbol('callback'):new Error('callback identity');
        let fired=false,nested=null,nestedIdentity=null,iconCalls=0;const widths=[],icons=[];
        if(variant==='late-icons-metadata')model.entities.get('A').alias='Icon fa:fa-user';
        const outcome=call=>{try{const value=call();return {ok:true,type:value.type,svg:value.svg,width:number(value.width),height:number(value.height)};}catch(error){return {ok:false,kind:typeof error,name:error?.name,message:error?.message,code:error?.code,original:Object.is(error,sentinel)};}};
        const render=(input,measurer,extra={})=>renderer.render(input,{measurer,...extra});
        const mutate=()=>{
          switch(variant){
            case 'cycle':case 'throw-cycle':model.callbackExtension=extension;break;
            case 'future-alias':saved.alias='changed future alias';break;
            case 'future-label':saved.label='changed future label';break;
            case 'delete-entity':model.entities.delete('C');break;
            case 'delete-attribute':oldAttributes.splice(0,1);break;
            case 'attribute-name':oldAttributes[1].name='changed future attribute';break;
            case 'same-row-name':firstRow.name='changed after type width';break;
            case 'replace-attributes':saved.attributes=[{type:'string',name:'replacement',keys:[],comment:''}];break;
            case 'detached-attributes':saved.attributes=[];oldAttributes[1].name='changed detached row';break;
            case 'styles':oldStyles.push('fill:red');break;
            case 'replace-styles':saved.cssStyles=['fill:blue'];break;
            case 'detached-styles':saved.cssStyles=['fill:blue'];oldStyles.push('fill:red');break;
            case 'direction':model.direction='TB';break;
            case 'title-accessibility':model.title='late title';model.accTitle='late accessible title';model.accDescr='late accessible description';break;
            case 'nested-existing':nestedIdentity=model.addEntity('C')===saved;break;
            case 'nested-new':nestedIdentity=model.addEntity('D')===model.entities.get('D');break;
            case 'nested-render':nested=outcome(()=>render(model,{width:basicWidth}));break;
            case 'nested-render-edited-attribute':oldAttributes[1].name='nested changed attribute';nested=outcome(()=>render(model,{width:basicWidth}));break;
            case 'nested-render-throw':nested=outcome(()=>render(model,{width(){throw sentinel;}}));break;
            case 'source-render-control':nested=outcome(()=>render(source,{width:basicWidth}));break;
            case 'cross-document-control':nested=outcome(()=>render(other,{width:basicWidth}));break;
            case 'borrowed-helper':{const receiver={entities:new Map()},value=model.addEntity.call(receiver,'X');nestedIdentity=value===receiver.entities.get('X');break;}
          }
          if(variant==='throw-cycle'||['throw-undefined','throw-null','throw-false','throw-object','throw-symbol'].includes(variant))throw sentinel;
        };
        try{
          const first=outcome(()=>render(model,{width(...args){
            widths.push(args);
            if(!fired&&(variant!=='same-row-name'||args[0]==='int')){fired=true;mutate();}
            return basicWidth(...args);
          }},{...(variant==='direction'?{maxWidth:64}:{}),...(variant==='late-icons-metadata'?{icons(name){icons.push(name);iconCalls++;model.title='icon title '+iconCalls;model.accTitle='icon accessible title '+iconCalls;model.accDescr='icon description '+iconCalls;return '<path d="M0 0h8v8H0z"/>';}}:{})}));
          const after=signature({model,saved,oldAttributes,oldStyles,firstRow});
          const identities={prototype:Object.getPrototypeOf(model)===prototype,entity:model.entities.get('C')===saved,attributes:saved.attributes===oldAttributes,styles:saved.cssStyles===oldStyles,extension:model.callbackExtension===extension,nested:nestedIdentity};
          const next=outcome(()=>render(model,{width:basicWidth}));
          row[name]={fired,first,nested,after,identities,widths,icons,iconCalls,next,afterNext:signature({model,saved,oldAttributes,oldStyles,firstRow})};
        }finally{if(name==='wasm'){renderer.dispose(model);renderer.dispose(other);}}
      }
      row.mismatches=Object.keys(row.js).filter(k=>JSON.stringify(row.js[k])!==JSON.stringify(row.wasm[k]));rows.push(row);
    }
    return {rows};
  });
  expect(report.rows).toHaveLength(30);
  for(const row of report.rows){
    expect(row.js.fired,row.variant+' oracle measurement trigger').toBe(true);
    expect(row.wasm.fired,row.variant+' native measurement trigger').toBe(true);
    if(row.variant==='late-icons-metadata')expect(row.js.iconCalls).toBeGreaterThan(0);
  }
  expect(report.rows.filter(row=>row.mismatches.length).map(({variant,mismatches})=>({variant,mismatches}))).toEqual([]);
});

test('ER live scalar conversion preserves use order, original throws and recovery',async({page},testInfo)=>{
  await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js');
    const oracle=await import('/dist/evidence/pele-oracle.js'),engine=await loadRenderer();
    const source='erDiagram\nA { int aid PK }\nB { string bid FK }\nA ||--o{ B : owns';
    const variants=['label-null','label-number','label-symbol','alias-null','alias-number','alias-symbol','attribute-type-null','attribute-type-number','attribute-type-symbol','attribute-name-null','attribute-name-number','attribute-name-symbol','attribute-comment-null','attribute-comment-number','attribute-comment-symbol','keys-null','keys-number','keys-symbol','keys-nested-cycle','keys-coercion','keys-coercion-symbol','keys-coercion-throw','accTitle-null','accTitle-number','accTitle-symbol','accDescr-number','title-number','title-symbol','direction-number'];
    const rows=[];
    for(const variant of variants){
      const row={variant};
      for(const [name,renderer]of [['js',oracle],['wasm',engine]]){
        const model=renderer.parse(source),saved=model.entities.get('B'),original=saved.attributes[0],sentinel={marker:'original-conversion-throw'};
        let fired=false;const trace=[];
        const width=(text,size,style)=>{trace.push(['width',text,size,style]);return text.length*size*.6;};
        const call=fn=>{try{const value=fn();return {ok:true,svg:value.svg,width:String(value.width),height:String(value.height)};}catch(error){return {ok:false,kind:typeof error,name:error?.name,message:error?.message,code:error?.code,original:error===sentinel};}};
        const mutate=()=>{
          const [field,kind]=variant.split('-');
          const value=kind==='null'?null:kind==='number'?42:Symbol('live-value');
          if(field==='label'||field==='alias')saved[field]=value;
          else if(field==='accTitle'||field==='accDescr'||field==='title'||field==='direction')model[field]=value;
          else if(field==='attribute')original[kind]=variant.endsWith('null')?null:variant.endsWith('number')?42:Symbol('live-value');
          else if(field==='keys'){
            if(kind==='null'||kind==='number'||kind==='symbol')original.keys=[value];
            else if(kind==='nested'){const a=['a'];a.push(a);original.keys=[a];}
            else original.keys=[{[Symbol.toPrimitive](hint){trace.push(['coerce',hint]);if(variant.endsWith('throw'))throw sentinel;return variant.endsWith('symbol')?Symbol('converted'):'converted-key';}}];
          }
        };
        try{
          const first=call(()=>renderer.render(model,{measurer:{width(...args){if(!fired){fired=true;mutate();}return width(...args);}}}));
          const firstTrace=trace.slice();
          // Repair the same objects after either success or failure. No reparse.
          saved.label='B';saved.alias='';original.type='string';original.name='bid';original.comment='';original.keys=['FK'];model.direction='TB';delete model.accTitle;delete model.accDescr;delete model.title;
          const next=call(()=>renderer.render(model,{measurer:{width}}));
          row[name]={fired,first,firstTrace,next,identity:model.entities.get('B')===saved&&saved.attributes[0]===original};
        }finally{if(name==='wasm')renderer.dispose(model);}
      }
      row.mismatches=Object.keys(row.js).filter(k=>JSON.stringify(row.js[k])!==JSON.stringify(row.wasm[k]));rows.push(row);
    }
    return rows;
  });
  const reportPath=testInfo.outputPath('live-scalar-report.json');
  await writeFile(reportPath,JSON.stringify(report,null,2));
  await testInfo.attach('live-scalar-report',{path:reportPath,contentType:'application/json'});
  for(const row of report){expect(row.js.fired,row.variant).toBe(true);expect(row.wasm.fired,row.variant).toBe(true);}
  expect(report.filter(row=>row.mismatches.length).map(({variant,mismatches})=>({variant,mismatches}))).toEqual([]);
});

test('ER fallback syntax retains live callback edits and nested methods without an arena owner',async({page},testInfo)=>{
  await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js');
    const oracle=await import('/dist/evidence/pele-oracle.js'),engine=await loadRenderer();
    const source='erDiagram\nA { int aid PK }\nB { string bid FK }\nA ||--o{ B : owns';
    const variants=[['classes',source+'\nclassDef hot fill:red\nclass A hot'],['styles',source+'\nstyle A fill:red'],['groups','erDiagram\nsubgraph G\nA { int aid PK }\nend\nB { string bid FK }\nA ||--o{ B : owns'],['redefinition',source+'\nB["Second label"]'],['large-comment',source+'\n%%'+ 'x'.repeat(65536)]];
    const width=(text,size,style)=>text.length*size*((style&1)?.71:.6),rows=[];
    for(const [variant,text]of variants){const row={variant};
      for(const [name,renderer]of [['js',oracle],['wasm',engine]]){
        const model=renderer.parse(text,{limit:100000}),saved=model.entities.get('B'),attribute=saved.attributes[0],extension={marker:'callback'};extension.self=extension;
        let fired=false,nested=null,helperIdentity=false;
        const outcome=fn=>{try{const r=fn();return {ok:true,svg:r.svg,width:String(r.width),height:String(r.height)};}catch(error){return {ok:false,name:error?.name,message:error?.message,code:error?.code};}};
        try{
          const first=outcome(()=>renderer.render(model,{measurer:{width(...args){if(!fired){fired=true;attribute.name='callback changed';model.extension=extension;helperIdentity=model.addEntity('B')===saved;nested=outcome(()=>renderer.render(model,{measurer:{width}}));}return width(...args);}}}));
          const next=outcome(()=>renderer.render(model,{measurer:{width}}));
          row[name]={fired,first,next,nested,helperIdentity,extension:model.extension===extension,entity:model.entities.get('B')===saved,attribute:saved.attributes[0]===attribute,name:attribute.name};
        }finally{if(name==='wasm')renderer.dispose(model);}
      }
      row.mismatches=Object.keys(row.js).filter(key=>JSON.stringify(row.js[key])!==JSON.stringify(row.wasm[key]));rows.push(row);
    }return rows;
  });
  const reportPath=testInfo.outputPath('fallback-callback-report.json');await writeFile(reportPath,JSON.stringify(report,null,2));await testInfo.attach('fallback-callback-report',{path:reportPath,contentType:'application/json'});
  for(const row of report){expect(row.js.fired,row.variant).toBe(true);expect(row.js.helperIdentity,row.variant).toBe(true);}
  expect(report.filter(row=>row.mismatches.length).map(({variant,mismatches})=>({variant,mismatches}))).toEqual([]);
});
