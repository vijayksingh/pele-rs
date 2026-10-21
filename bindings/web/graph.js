import {createHostLedger,LiveFieldBindings} from './host-values.js';
import {LayoutGraph,LayoutStorage,applyLayoutDelta} from './layout-storage.js';
import {tryInitializeSourceLayout} from './source-initializer.js';
import {completeBridgePublication,invalidateBridgeObservation,getGraphTraversal,numberValue, reconcileNestedGraph as reconcileGraph, matchesNestedGraph as matchesGraph, snapshotNestedGraph as snapshotGraph} from './graph-storage.js';
export {stringValue} from './graph-storage.js';
const layoutTraversal=getGraphTraversal(LayoutStorage);
const layoutMethods={
  reconcile(graph,changed){invalidateBridgeObservation(this);if(!(graph instanceof LayoutGraph))throw new TypeError('Source layout outcome lost its baseline');return layoutTraversal.reconcileGraph(this,graph,changed);},
  reconcileDelta(delta){invalidateBridgeObservation(this);return this.reconcile(applyLayoutDelta(this.graph,delta),delta.nodes.map(([id])=>id));},
  matchesPublishedGraph(){return layoutTraversal.matchesGraph(this);},
  snapshot(values=[],delta=false){return layoutTraversal.snapshotGraph(this,values,delta);},
  hostReferenceId(value){if(!this.hostIds.has(value)){if(this.hostValues.length>=1_000_000)throw new RangeError('Host reference budget exceeded');this.hostIds.set(value,this.hostValues.length);this.hostValues.push(value);}return this.hostIds.get(value);}
};
/** Schema functions/prototypes share identity across documents of one renderer. */
export class ModelRegistry {
  constructor() { this.schemas = new Map(); this.owners = new WeakMap(); this.sharedFunctions = new Map(); }
  registration(schema) {
    const key = `${schema.id}:${schema.version}`;
    if (this.schemas.has(key)) {
      const known=this.schemas.get(key);
      if (JSON.stringify(known.schema)!==JSON.stringify(schema)) throw new TypeError('Conflicting native model schema identity');
      return known;
    }
    const registration = {schema, functions: new Map(), functionIds: new WeakMap(), prototypes: new Map(), prototypeIds: new WeakMap(), documents: [], constructors: new Map(), sharedIds: new Map((schema.sharedFunctions??[]).map(entry=>[entry.identity,entry.method]))};
    this.schemas.set(key, registration);
    const aliases=new Map((schema.sharedFunctions??[]).map(entry=>[entry.method,entry.identity]));
    const shapes=new Map((schema.functionShapes??[]).map(entry=>[entry.method,entry.constructible]));
    const live=registration=>{
      registration.documents=registration.documents.filter(reference=>{const value=reference.deref();return value&&!value.disposed;});
      return registration.documents.at(-1)?.deref();
    };
    for (const method of schema.methods) {
      const registry = this, identity=aliases.get(method.id), constructible=shapes.get(method.id)??false;
      let shared=identity&&this.sharedFunctions.get(identity);
      if(shared&&(shared.name!==method.name||shared.arity!==method.arity||shared.constructible!==constructible))throw new TypeError('Conflicting shared native function descriptor');
      if(identity&&!shared){shared={name:method.name,arity:method.arity,constructible,registrations:new Set(),callable:null};this.sharedFunctions.set(identity,shared);}
      if(shared)shared.registrations.add(registration);
      const dispatch=(receiver,argumentsValue)=>{
        const direct=receiver&&registry.owners.get(receiver);
        let owner;
        if(shared) {
          if(direct&&!direct.disposed&&direct.registration.sharedIds.has(identity))owner=direct;
          if(!owner)for(const candidate of [...shared.registrations].reverse()){owner=live(candidate);if(owner)break;}
        } else owner=direct?.registration===registration&&!direct.disposed?direct:live(registration);
        if (!owner) throw new TypeError('Public method has no live document');
        return owner.invoke(shared?owner.registration.sharedIds.get(identity):method.id, receiver, argumentsValue);
      };
      // Keep built-in helpers' intrinsic arity: rewriting it changes
      // Firefox's observed ordering of ordinary function properties.
      const callable=(shared&&shared.callable)||(constructible?
        method.arity===1?{[method.name]:function(_first){return dispatch(this,Array.from(arguments));}}[method.name]:
        {[method.name]:function(_first,_second){return dispatch(this,Array.from(arguments));}}[method.name]:
        {[method.name](...argumentsValue){return dispatch(this,argumentsValue);}}[method.name]);
      // Firefox materializes ordinary function properties lazily. Pinned
      // exported helpers have their prototype before length is first observed.
      if(constructible)void callable.prototype;
      if(!constructible||(method.arity!==1&&method.arity!==2))Object.defineProperty(callable, 'length', {value: method.arity, configurable: true});
      if(shared)shared.callable=callable;
      registration.functions.set(method.id, callable);
      registration.functionIds.set(callable, method.id);
    }
    for (const prototype of schema.prototypes) {
      // Shell owns JavaScript newTarget/receiver semantics. Only a captured
      // native capability can install initialization; unimplemented classes
      // retain the explicit construction error. Arguments are not inspected.
      const constructor = class { constructor() {
        const construct=registration.constructors.get(prototype.id);
        if(!construct)throw new TypeError('Public model construction is unavailable');
        construct(this,arguments);
      }};
      Object.defineProperty(constructor, 'name', {value: prototype.name, configurable: true});
      const value=constructor.prototype;
      registration.prototypes.set(prototype.id, value);
      registration.prototypeIds.set(value, prototype.id);
    }
    for (const prototype of schema.prototypes) {
      const value = registration.prototypes.get(prototype.id), parent = prototype.parent;
      Object.setPrototypeOf(value, parent.kind === 'null' ? null : parent.kind === 'prototype' ? registration.prototypes.get(parent.prototype) : Object.prototype);
      for (const member of prototype.members) {
        const descriptor = {enumerable: !!(member.flags & 1), configurable: !!(member.flags & 4)};
        if (member.kind === 'method') {
          descriptor.value = registration.functions.get(member.method);
          descriptor.writable = !!(member.flags & 2);
        } else {
          descriptor.get = member.get == null ? undefined : registration.functions.get(member.get);
          descriptor.set = member.set == null ? undefined : registration.functions.get(member.set);
        }
        Object.defineProperty(value, member.name, descriptor);
      }
    }
    registration.prototypeBaseline = [...registration.prototypes.values()].map(value => ({value, parent:Object.getPrototypeOf(value), descriptors:Object.getOwnPropertyDescriptors(value)}));
    return registration;
  }
}
export class ModelBridge {
  constructor(graph, schema, registry, invoke, prevalidatedRegistration, rootBinding, hostLedger, liveDefinitions, deferLiveBinding=false) {
    const publication=invalidateBridgeObservation(this);
    this.registry = registry;
    if(prevalidatedRegistration && (prevalidatedRegistration.schema !== schema || registry.schemas.get(`${schema.id}:${schema.version}`) !== prevalidatedRegistration)) throw new TypeError('Invalid prevalidated model registration');
    this.registration = prevalidatedRegistration ?? registry.registration(schema);
    if(this.registration.documents.length>256)this.registration.documents=this.registration.documents.filter(reference=>{const value=reference.deref();return value&&!value.disposed;});
    this.registration.documents.push(new WeakRef(this));
    this.disposed=false;
    this.invoke = (...args) => this.activeInvoke ? this.activeInvoke(...args) : invoke(...args);
    this.ids = new WeakMap();
    this.hostLedger = hostLedger ?? createHostLedger();
    this.hostIds = this.hostLedger.ids;
    this.hostValues = this.hostLedger.values;
    this.objects = [];
    // Only this exact receiver/prototype pair aliases its native root codec.
    // It does not authorize the subclass prototype for unrelated objects.
    this.rootBinding = rootBinding;
    if(graph instanceof LayoutGraph)Object.assign(this,layoutMethods);
    if (graph instanceof LayoutGraph && tryInitializeSourceLayout(this,graph)) completeBridgePublication(this,graph,publication);
    else this.reconcile(graph);
    if(liveDefinitions?.length){this.liveFields=new LiveFieldBindings(this,liveDefinitions);if(!deferLiveBinding)this.liveFields.bind(this.root);}
  }
  reconcile(graph, changed) {
    return reconcileGraph(this, graph, changed);
  }
  matchesPublishedGraph() {
    return matchesGraph(this);
  }
  reconcileDelta(delta) {
    invalidateBridgeObservation(this);
    if (delta.stringsStart !== this.graph.strings.length || delta.nodesStart !== this.graph.nodes.length) throw new TypeError('Stale native model delta');
    const nodes=this.graph.nodes.slice();
    for(const [id,node] of delta.nodes) nodes[id]=node;
    return this.reconcile({...this.graph,strings:this.graph.strings.concat(delta.strings),nodes},delta.nodes.map(([id])=>id));
  }
  hostReference(value) {
    if (!this.hostIds.has(value)) {
      if(this.hostValues.length>=1_000_000)throw new RangeError('Host reference budget exceeded');
      this.hostIds.set(value,this.hostValues.length);this.hostValues.push(value);
    }
    return {kind:'hostReference',value:this.hostIds.get(value)};
  }
  hostException(value) {
    if(this.hostValues.length>=1_000_000)throw new RangeError('Host reference budget exceeded');
    const id=this.hostValues.length;this.hostValues.push(value);return {kind:'hostException',value:id};
  }
  value(atom) {
    switch (atom.kind) {
      case 'undefined': return undefined;
      case 'null': return null;
      case 'bool': return atom.value;
      case 'number': return numberValue(atom.value);
      case 'string': return this.strings[atom.value];
      case 'ref': return this.objects[atom.value];
      case 'builtinFunction': return this.registration.functions.get(atom.value);
      case 'hostReference': return this.hostValues[atom.value];
      default: throw new TypeError('Unknown public atom kind');
    }
  }
  // All retained nodes are scanned, including externally held detached arrays.
  // Extra arguments join the same arena; calls never serialize nested objects.
  snapshot(values = [], delta = false) {
    return snapshotGraph(this, values, delta);
  }
}
