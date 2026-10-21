import {test, expect} from '@playwright/test';
import {writeFile} from 'node:fs/promises';

test('ER public source grammar preserves direction precedence, cross-line spans, and keyword boundaries', async ({page}, testInfo) => {
  await page.goto('/examples/web/');
  const report = await page.evaluate(async () => {
    const {loadRenderer} = await import('/dist/package-web/package/full.js');
    const oracle = await import('/dist/evidence/pele-oracle.js'), engine = await loadRenderer();
    const cases = [
      ['ordinary', 'erDiagram\nA { int id PK\n string name }\nB\nA ||--o{ B : owns', true],
      ['mixed-direction-case', 'erDiagram\nDiReCtIoN lR\nA\nB\nA ||--o{ B : owns', true],
      ['priority-TB', 'erDiagram\ndirection TB direction LR\nA', true],
      ['priority-BT', 'erDiagram\ndirection BT direction RL direction LR\nA', true],
      ['priority-RL', 'erDiagram\ndirection RL direction LR\nA', true],
      ['rightmost-cross-LF', 'erDiagram\ndirection TB direction\nTB\nA', true],
      ['rightmost-cross-CRLF', 'erDiagram\ndirection TB direction\r\nTB\nA', true],
      ['priority-before-crossline', 'erDiagram\ndirection TB direction\r\nLR\nA', true],
      ['rightmost-invalid', 'erDiagram\ndirection LR direction invalid\nA', true],
      ['direction-no-left-boundary', 'erDiagram\nxdirection LR\nA', true],
      ['direction-no-right-boundary', 'erDiagram\ndirection LRx\nA', true],
      ['direction-missing-space', 'erDiagram\ndirectionLR\nA', true],
      ['direction-CR-tail', 'erDiagram\ndirection LR\rA\nB', true],
      ['direction-line-separator-tail', 'erDiagram\ndirection LR\u2028A\nB', true],
      ['direction-paragraph-separator-tail', 'erDiagram\ndirection LR\u2029A\nB', true],
      ['direction-cross-line-separator', 'erDiagram\ndirection TB direction\u2028TB\nA', true],
      ['direction-cross-paragraph-separator', 'erDiagram\ndirection TB direction\u2029TB\nA', true],
      ['unicode-candidates', 'erDiagram\nKİſ😀direction LR\nA', true],
      ['unicode-folding-identifiers', 'erDiagram\nK\nİ\nſ\nκ\nſtyle\nendÉ\nonlyſone'],
      ['unicode-direction-lookalike', 'erDiagram\ndİrection\nA'],
      ['ascii-name-boundaries', 'erDiagram\nend_\nend2\nstyle_\nclassDefX\noneX\nmanyX\ntoX', true],
      ['unicode-keyword-boundaries', 'erDiagram\nendÉ\nA'],
      ['unicode-cardinality-boundary', 'erDiagram\nA oneÉ--one B : x'],
      ['subgraph-end-space', 'erDiagram\nsubgraph box\nA\nend \t\r\nB', true],
      ['classes-and-styles', 'erDiagram\nA\nclassDef hot fill:red\nclass A hot\nstyle A stroke:blue', true],
      ['accessibility', 'erDiagram\naccTitle: T\naccDescr { D }\nA', true],
      ['unterminated-block', 'erDiagram\nA { int'],
      ['incomplete-direction', 'erDiagram\ndirection'],
    ];
    for (const spelling of ['one or zero','one or more','one or many','one','only one','zero or one','zero or more','zero or many','many(0)','many(1)','many']) {
      cases.push(['word-cardinality/'+spelling, `erDiagram\nA ${spelling} to one B : x`]);
      cases.push(['upper-cardinality/'+spelling, `erDiagram\nA ${spelling.toUpperCase()} OPTIONALLY TO ONE B : x`]);
    }
    for (const left of ['o|','||','}|','}o']) for (const link of ['--','..','.-','-.']) {
      cases.push([`symbol/${left}${link}`, `erDiagram\nA ${left}${link}o{ B : x`]);
    }
    for (const suffix of ['A','_','é','😀']) cases.push(['relation-boundary/'+suffix, `erDiagram\nA ||--|o${suffix} : x`]);
    const bits = n => {const b=new ArrayBuffer(8), v=new DataView(b);v.setFloat64(0,n);return v.getBigUint64(0).toString(16).padStart(16,'0');};
    const canonical = root => {
      const ids=new Map(),pending=[],nodes=[];
      const atom=v=>{
        if(v===undefined)return ['undefined'];if(v===null)return ['null'];
        if(typeof v==='number')return ['number',bits(v)];
        if(typeof v==='function')return ['function',v.length];
        if(typeof v!=='object')return [typeof v,v];
        if(!ids.has(v)){ids.set(v,pending.length);pending.push(v);}return ['ref',ids.get(v)];
      };
      const rootAtom=atom(root);
      for(let i=0;i<pending.length;i++){
        const v=pending[i],p=Object.getPrototypeOf(v),kind=Array.isArray(v)?'array':v instanceof Map?'map':v instanceof Set?'set':v instanceof Date?'date':p===null?'null-object':'object';
        const properties=Reflect.ownKeys(v).map(k=>{const d=Object.getOwnPropertyDescriptor(v,k);return [k,d.enumerable,d.configurable,'value'in d?['data',d.writable,atom(d.value)]:['accessor',atom(d.get),atom(d.set)]];});
        const prototype=p&&![Object.prototype,Array.prototype,Map.prototype,Set.prototype,Date.prototype].includes(p)?Reflect.ownKeys(p).filter(k=>k!=='constructor').map(k=>{const d=Object.getOwnPropertyDescriptor(p,k);return [k,d.enumerable,d.configurable,d.writable,atom(d.value)];}):[];
        nodes.push([kind,properties,prototype,kind==='map'?[...v].map(([k,x])=>[atom(k),atom(x)]):kind==='set'?[...v].map(atom):kind==='date'?bits(+v):null]);
      }
      return JSON.stringify([rootAtom,nodes]);
    };
    const rows=[];
    for(const [id,source,valid=false] of cases){
      const row={id,source,valid};
      for(const [lane,renderer] of [['js',oracle],['wasm',engine]]){
        let model;const traces=[];
        const options={measurer:{width(text,size,style){const width=text.length*size*(style&1?.71:.6);traces.push([text,bits(size),style,bits(width)]);return width;}}};
        try{
          model=renderer.parse(source);const graph=canonical(model),result=renderer.render(model,options);
          row[lane]={ok:true,graph,result:canonical(result),traces,after:canonical(model)};
        }catch(error){row[lane]={ok:false,stage:model?'render':'parse',error:{name:error?.name,message:error?.message,code:error?.code,type:error?.type,line:error?.line,column:error?.column,snippet:error?.snippet},...(model?{graph:canonical(model),traces}:{})};}
        finally{if(lane==='wasm'&&model)engine.dispose(model);}
      }
      row.equal=JSON.stringify(row.js)===JSON.stringify(row.wasm);rows.push(row);
    }
    return {count:rows.length,rows};
  });
  const path=testInfo.outputPath('er-lexical-dispatch.json');
  await writeFile(path,JSON.stringify(report,null,2));
  await testInfo.attach('er-lexical-dispatch',{path,contentType:'application/json'});
  expect(report.count).toBe(70);
  expect(report.rows.filter(r=>r.valid&&!r.js.ok).map(r=>({id:r.id,error:r.js}))).toEqual([]);
  expect(report.rows.filter(r=>!r.equal).map(r=>({id:r.id,js:r.js,wasm:r.wasm}))).toEqual([]);
});
