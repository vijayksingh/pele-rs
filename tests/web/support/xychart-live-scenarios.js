export const SOURCE='xychart\ntitle "Initial title"\nx-axis Month [Jan, Feb, Mar]\ny-axis Revenue -10 --> 20\nline Forecast [5 "PointA", 10 "PointB", 15 "PointC"]\nbar Sales [-5, 10, 15]';
export const MODES=['baseline','early-title','legend-current','legend-next','plots-replace','plots-append','axis-replace','category-current','categories-append','bounds','point-labels','data-value','data-replace','late-point','title-accessibility','throw-object','throw-undefined','nested'];
export const cases=()=>[false,true].flatMap(raw=>[false,true].flatMap(responsive=>MODES.map(mode=>({raw,responsive,mode}))));
const bits=n=>{const b=new DataView(new ArrayBuffer(8));b.setFloat64(0,n);return b.getBigUint64(0).toString(16).padStart(16,'0');};
const output=v=>({svg:v.svg,width:bits(v.width),height:bits(v.height)}),copy=v=>JSON.parse(JSON.stringify(v));
const state=m=>copy({title:m.title??null,accTitle:m.accTitle??null,accDescr:m.accDescr??null,orientation:m.orientation??null,xAxis:m.xAxis,yAxis:m.yAxis,hasSetYAxis:m.hasSetYAxis,plots:m.plots});
export function runXYChart(core,entry,dispose,raw,responsive,mode){
 const model=raw?entry.parse(SOURCE,{},undefined,0,{}):core.parse(SOURCE);
 if(responsive)model.orientation='horizontal';
 const x=model.xAxis,categories=x.categories,y=model.yAxis,plots=model.plots,line=plots[0],bar=plots[1],data=line.data,labels=line.pointLabels,sentinel={mode},trace=[];let fired=false,nested,hits=0;
 const config={xyChart:{showDataLabel:true}},settings=responsive?{maxWidth:320}:{};
 const draw=options=>raw?entry.render(model,config,options):core.render(model,{config,...options});
 const mutate=()=>{switch(mode){
 case'early-title':model.title='Early title';break;
 case'legend-current':line.title='Changed current';line.type='bar';break;
 case'legend-next':bar.title='Changed later';break;
 case'plots-replace':model.plots=[{type:'line',title:'Replacement',data:[['a',3],['b',7]],pointLabels:['R1','R2']}];break;
 case'plots-append':plots.push({type:'line',title:'Appended',data:[['a',2],['b',4]],pointLabels:['A1','A2']});break;
 case'axis-replace':model.xAxis={type:'linear',title:'Replacement axis',min:0,max:2};model.yAxis={type:'linear',title:'Replacement value',min:0,max:30};break;
 case'category-current':categories[0]='Changed current';categories[1]='Changed later';break;
 case'categories-append':categories.push('Apr');break;
 case'bounds':y.min=-20;y.max=40;model.hasSetYAxis=true;break;
 case'point-labels':labels[0]='Changed point';labels[1]='Changed later point';break;
 case'data-value':data[1][1]=18;bar.data[0][1]=-8;break;
 case'data-replace':line.data=[['a',1],['b',3],['c',7]];break;
 case'late-point':data[1][1]=19;labels[1]='Paint changed';break;
 case'title-accessibility':model.title='Changed title';model.accTitle='Changed access';model.accDescr='Changed description';break;
 case'throw-object':case'throw-undefined':model.title='Before failure';data[1][1]=17;throw mode.endsWith('undefined')?undefined:sentinel;
 case'nested':model.title='Nested title';nested=output(draw({measurer:{width:(t,s)=>t.length*s*.6}}));model.title='After nested';break;
 }};
 const target=mode==='early-title'?'0':mode==='title-accessibility'?'Initial title':['category-current','categories-append'].includes(mode)?'Jan':['point-labels','data-value','data-replace','late-point'].includes(mode)?'PointA':'Forecast';
 const resources=(record,active)=>({measurer:{width(t,s,style){record.push(['width',t,bits(s),style??0]);if(active&&!fired&&t===target){hits++;if(mode!=='late-point'||hits===2){fired=true;mutate();}}return t.length*s*.6;}}});
 try{
 let first;try{first={ok:true,...output(draw({...settings,...resources(trace,true)}))};}catch(e){first={ok:false,original:e===(mode.endsWith('undefined')?undefined:sentinel)};if(!first.original)first.error={name:e?.name,message:e?.message};}
 const after=state(model),original={x:copy(x),categories:copy(categories),y:copy(y),plots:copy(plots),line:copy(line),bar:copy(bar),data:copy(data),labels:copy(labels)},identity={x:model.xAxis===x,categories:model.xAxis.categories===categories,y:model.yAxis===y,plots:model.plots===plots,line:model.plots.includes(line),bar:model.plots.includes(bar),data:line.data===data,labels:line.pointLabels===labels},recoveryTrace=[];
 let recovery;try{recovery={ok:true,...output(draw({...settings,...resources(recoveryTrace,false)}))};}catch(e){recovery={ok:false,error:{name:e?.name,message:e?.message}};}
 return{raw,responsive,mode,fired,first,trace,after,original,identity,...(nested?{nested}:{}),recovery,recoveryTrace};
 }finally{dispose(model);}
}
