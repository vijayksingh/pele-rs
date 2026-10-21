// Unselected source-construction sink. The caller supplies an empty FlatGraph
// configured for native constructor admission, not observer RangeError limits.
// No JS model shells, schema/prototype lookups, registry ownership or callbacks
// occur here. takeGraph returns an UNADMITTED private candidate, never a model.
import {createConstructorDecoder,invalidConstructorBatch} from './constructor-storage.js';
const flatKind=kind=>kind===0?0:kind===1?2:kind===2?3:kind===3?4:1;
const finish=(state,node)=>{
  state.graph.finishNode(node.id,flatKind(node.kind),node.kind===1?node.length:node.kind===4?state.prototypeId:0);
  return true;
};
const decodeFlatConstruction=createConstructorDecoder({
  rawAtoms:true,
  // Native encoder checks model table/unit limits before emission. The complete
  // native admission gate still runs before exposure. This sink adds no ER limit
  // category or earlier validation pass; storage retains physical/type guards.
  stringRoom(state,id,units){if(id!==state.graph.stringCount)invalidConstructorBatch();},
  addString(state,id,text){if(state.graph.strings.internFirst(text)!==id)invalidConstructorBatch();},
  nodeCount:state=>state.graph.nodeCount,
  beginNode(state,id,kind,length){state.graph.beginNode(id);return {id,kind,length,properties:false};},
  mapEntry(state,node,kt,kl,kh,key,vt,vl,vh,value){
    state.graph.appendItem(node.id,kt,kl,kh);state.graph.appendItem(node.id,vt,vl,vh);
  },
  setValue(state,node,tag,lo,hi,value){state.graph.appendItem(node.id,tag,lo,hi);},
  propertyKey:(state,key)=>key,
  property(state,node,key,name,flags,tag,lo,hi,value){state.graph.appendProperty(node.id,key,flags,tag,lo,hi);return true;},
  endNode:finish,
  layoutKey:(state,key)=>state.graph.stringValueAt(key),
  saveLayout:(state,names,ids)=>ids,
  record(state,id,keys,atom,registers){
    const graph=state.graph;graph.beginNode(id);
    for(const key of keys){
      atom();graph.appendProperty(id,key,7,registers.tag,registers.lo,registers.hi);
    }
    graph.finishNode(id,0,0);return 0;
  },
  dense(state,id,length,atom,registers){
    const graph=state.graph;graph.beginNode(id);
    for(let index=0;index<length;index++){
      atom();
      // Native construction interns canonical index strings before event12.
      const key=graph.strings.firstId(String(index));
      if(key===undefined)invalidConstructorBatch();
      graph.appendProperty(id,key,7,registers.tag,registers.lo,registers.hi);
    }
    graph.finishNode(id,2,length);return 0;
  },
});
export class FlatConstruction {
  constructor(graph,prototypeId){
    if(!graph||graph.nodeCount!==0||graph.stringCount!==0||!Number.isInteger(prototypeId)||prototypeId<0||prototypeId>0xffffffff)invalidConstructorBatch();
    this.graph=graph;this.prototypeId=prototypeId;
    this.constructionNode=null;this.constructionString=null;this.constructionLayouts=[];
  }
  publishBatch(bytes){return decodeFlatConstruction(this,bytes);}
  setRoot(tag,lo=0,hi=0){this.graph.setRoot(tag,lo,hi);}
  takeGraph(){
    if(!this.graph||this.constructionNode||this.constructionString)throw new TypeError('Incomplete host constructor stream');
    const graph=this.graph.finishBoundary();
    this.graph=null;this.constructionLayouts=[];return graph;
  }
  discard(){this.graph=null;this.constructionNode=null;this.constructionString=null;this.constructionLayouts=[];}
}

// Operation-local append stream. Same decoder, existing graph and canonical IDs;
// no root change, lifecycle adoption, model admission or graph detachment.
export class FlatAppendConstruction {
  constructor(graph,prototypeId){
    if(!graph||!Number.isInteger(prototypeId)||prototypeId<0||prototypeId>0xffffffff)invalidConstructorBatch();
    this.graph=graph;this.prototypeId=prototypeId;
    this.constructionNode=null;this.constructionString=null;this.constructionLayouts=[];
  }
  publishBatch(bytes){return decodeFlatConstruction(this,bytes);}
  finishBatch(){
    if(this.constructionNode||this.constructionString)throw new TypeError('Incomplete host constructor stream');
    return this.graph;
  }
}
