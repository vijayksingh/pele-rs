// Owner-local live values. Only authenticated captured-constructor metadata can
// install field masks. No diagram names, field algorithms or global identity map.
export function createHostLedger(){return {values:[],ids:new Map(),opaque:new WeakSet(),freeTemporary:[]};}
export function retainHostValue(ledger,value){
  let id=ledger.ids.get(value);
  if(id===undefined){id=ledger.values.length;if(id>=1_000_000)throw new RangeError('Host reference budget exceeded');ledger.values.push(value);ledger.ids.set(value,id);}
  if((typeof value==='object'&&value!==null)||typeof value==='function')ledger.opaque.add(value);
  return id;
}
export class LiveFieldBindings {
  constructor(bridge,definitions){
    this.bridge=bridge;this.definitions=definitions;this.fields=new WeakMap();this.nodes=new Set();
  }
  isOpaque(value){return ((typeof value==='object'&&value!==null)||typeof value==='function')&&this.bridge.hostLedger.opaque.has(value);}
  isProtectedNode(id){return this.nodes.has(id);}
  field(object,key){return this.fields.get(object)?.get(key);}
  entries(object){return this.fields.get(object)?.entries()??[];}
  bind(receiver,method){
    if((typeof receiver!=='object'||receiver===null)&&typeof receiver!=='function')return;
    const applicable=this.definitions.filter(x=>method===undefined||x.methods.includes(method));
    if(!applicable.length)return;
    let fields=this.fields.get(receiver);if(!fields){fields=new Map();this.fields.set(receiver,fields);}
    const id=this.bridge.ids.get(receiver),node=id===undefined?undefined:this.bridge.graph.nodes[id];
    for(const definition of applicable){
      if(fields.has(definition.key))continue;
      // Read only the accepted native record. Never touch the live descriptor,
      // which may have become an accessor, been replaced, or been deleted.
      const prior=node?.properties.find(p=>this.bridge.strings[p.key]===definition.key);
      const value=prior?.value??{kind:'undefined'};
      fields.set(definition.key,{value,flags:prior?.flags??7});
      if(value.kind==='ref'){
        const object=this.bridge.objects[value.value];
        this.nodes.add(value.value);retainHostValue(this.bridge.hostLedger,object);
      }
    }
  }
  release(){this.bridge=null;this.fields=null;this.nodes=null;this.definitions=null;}
}
// PublicHostValue replies transfer scalars only when native requests their value.
// Opaque values remain owner-ledger references, never recursively observed.
export function hostReply(bridge,value){
  if(value===undefined)return [0,undefined];if(value===null)return [1,null];
  if(typeof value==='boolean')return [2,value];if(typeof value==='number')return [3,value];
  if(typeof value==='string')return [4,value];
  return [5,retainHostValue(bridge.hostLedger,value)];
}
export function performHostOperation(bridge,operation){
  if(operation.kind==='get')return bridge.value(operation.receiver)[operation.key];
  if(operation.kind==='call'){
    const callable=bridge.value(operation.function),receiver=bridge.value(operation.receiver);
    if(typeof callable!=='function')throw new TypeError(operation.notCallableError);
    const returned=Reflect.apply(callable,receiver,operation.arguments.map(value=>bridge.value(value)));
    return operation.result==='discard'?undefined:returned;
  }
  if(operation.kind==='greaterEqual')return bridge.value(operation.left)>=bridge.value(operation.right);
  if(operation.kind==='toString')return `${bridge.value(operation.value)}`;
  if(operation.kind==='truthy')return Boolean(bridge.value(operation.value));
  if(operation.kind==='strictEqual')return bridge.value(operation.left)===bridge.value(operation.right);
  throw new TypeError('Invalid detached host operation');
}

// Retained call results are temporary opaque operands, including primitives.
// Never intern them by SameValueZero: distinct -0/raw numbers must stay exact.
export function retainJobValue(ledger,value,tokens){
  const id=ledger.freeTemporary.length?ledger.freeTemporary.pop():ledger.values.length;
  if(id>=1_000_000)throw new RangeError('Host reference budget exceeded');
  ledger.values[id]=value;tokens.add(id);return id;
}
export function releaseJobValues(ledger,tokens,graph){
  // A failed transport cannot prove absence from native state; retain then.
  if(!tokens.size||!Array.isArray(graph?.nodes))return;
  const referenced=new Set(),visit=atom=>{if(atom?.kind==='hostReference'&&tokens.has(atom.value))referenced.add(atom.value);};
  visit(graph.root);
  for(const node of graph.nodes){
    if(!node)return;
    for(const property of node.properties)visit(property.value);
    if(node.kind==='map')for(const [key,value]of node.entries){visit(key);visit(value);}
    if(node.kind==='set')for(const value of node.values)visit(value);
  }
  for(const id of tokens)if(!referenced.has(id)){delete ledger.values[id];ledger.freeTemporary.push(id);}
}
