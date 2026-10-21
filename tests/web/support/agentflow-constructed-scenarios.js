export const MODES=['baseline','title','later-label','style-replace','add-node','throw-object','throw-undefined','nested','nested-throw'];
const bits=n=>{const b=new ArrayBuffer(8);new DataView(b).setFloat64(0,n);return [...new Uint8Array(b)].map(v=>v.toString(16).padStart(2,'0')).join('');};
export function runConstructedAgentflow(entry,C,dispose,trigger,mode){
 let model,fired=false,inNested=false,nested=null;const trace=[],original={tag:'original'};
 function render(){try{const v=entry.render(model,{}, {measurer:{width(text,size){trace.push(['width',text,bits(size)]);return text.length*size*.6;}}});return{ok:true,svg:v.svg,width:bits(v.width),height:bits(v.height),links:v.links};}catch(e){return{ok:false,original:e===original||e===undefined,name:e?.name,message:e?.message,undefined:e===undefined};}}
 function log(level,message){trace.push(['log',level,message]);if(fired)return;fired=true;
  if(mode==='title')model.title='Changed title';
  if(mode==='later-label')model.nodes.get('B').text='Changed Beta';
  if(mode==='style-replace')model.nodes.get('B').styles=['fill:red'];
  if(mode==='add-node')model.addVertex('C',{text:'Gamma',type:'text'},'task');
  if(mode==='throw-object')throw original;
  if(mode==='throw-undefined')throw undefined;
  if((mode==='nested'||mode==='nested-throw')&&!inNested){inNested=true;nested=render();inNested=false;if(mode==='nested-throw')throw original;}
 }
 model=new C({log});model.title='Initial';
 model.addVertex('A',{text:'Alpha',type:'text'},trigger==='shape'?'circle':'task');
 model.addVertex('B',{text:'Beta',type:'text'},'task');
 model.addSingleLink('A','B',{type:'arrow_point',stroke:'normal',length:1});
 if(trigger==='containment'){
  model.addSubGraph({text:'X'},['Y','A'],{text:'Group X',type:'text'});
  model.addSubGraph({text:'Y'},['X','B'],{text:'Group Y',type:'text'});
 }
 const originalStyles=model.nodes.get('B').styles;
 const state=()=>structuredClone({title:model.title,nodes:[...model.nodes].map(([k,n])=>[k,{text:n.text,styles:n.styles}]),diagnostics:model.diagnostics,reported:[...model.reported],oldStyles:originalStyles});
 const first=render(),firstTrace=trace.splice(0),after=state();
 model.nodes.get('B').text='After cache';originalStyles.push('stroke:blue');
 const recovery=render(),recoveryTrace=trace.splice(0),afterRecovery=state();
 const result={trigger,mode,fired,first,firstTrace,after,nested,recovery,recoveryTrace,afterRecovery};dispose(model);return JSON.parse(JSON.stringify(result));
}
