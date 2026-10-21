import {createHostLedger,retainHostValue,LiveFieldBindings,hostReply,performHostOperation,retainJobValue,releaseJobValues,materializeHostError} from './host-values.js';
import {LayoutGraph,encodeSourceInput} from './layout-storage.js';
import {decodeLayoutOutcome} from './layout-packet.js';
import {CapturedOwner,capturedSankeyAvailable,rememberCapturedModel,capturedObservedFrame} from './captured-owner.js';
import {HostErOwner} from './host-er-store.js';
import initialize from './initialize.js';
import {modelHostFunctions} from "./host-functions.js";
import {liveRenderRead,detachedRenderResult,releaseLiveRenderValues} from './live-render.js';
import {builtinDiagramType,builtinDiagramOrigin,ownRawModel,withBuiltinIcons,builtinInlineIcons} from './builtin-entries.js';
import {parseLimitOptions, parseLimitValues} from './parse-limits.js';
export {loadRegistry} from './registry.js';
import {mountWith} from './mount.js';
export {enableZoom} from './zoom.js';
import { parse_v2, render_numbers_with_resources, builtin_capabilities, NativeRegistry, foreign_decode_entities } from './pkg/pele.js';
import * as nativeCapabilities from './pkg/pele.js';
import {renderNumericOptions} from './render-numbers.js';
import {encodeGraphBinary,decodeGraphBinary} from './graph-binary.js';
import {fromNativeError} from './errors.js';
export {PeleError, PeleError as PeleCompatibilityError} from './errors.js';
import { ModelBridge, ModelRegistry, stringValue } from './graph.js';
import {bridgeObservationToken,completedObservationToken,completedPublicationToken,releaseBridgeObservation} from './graph-storage.js';
import { defaultMeasurer } from './resources.js';
export { canvasMeasurer, metricsMeasurer, forgetTextWidths } from './resources.js';
function result(json) {
  const envelope = JSON.parse(json);
  if (envelope.version !== 2) throw new Error('Unsupported compatibility contract');
  if (!envelope.ok) throw fromNativeError(envelope.error);
  return renderedValue(envelope.result);
}
// Success-only additive capability. Errors retain the exact v2 JSON boundary.
function sourceResult(response) {
  if(typeof response==='string')return result(response);
  if(!Array.isArray(response)||response.length!==3||response[0]!==1||!Number.isSafeInteger(response[1])||response[1]<0||response[1]>64*1024*1024)throw new TypeError('Invalid source render packet');
  const value=response[2],keys=['type','svg','width','height','links'],actual=value&&Object.keys(value);
  if(!value||Object.getPrototypeOf(value)!==Object.prototype||actual.length!==keys.length||!keys.every((key,index)=>actual[index]===key)||typeof value.type!=='string'||typeof value.svg!=='string'||typeof value.width!=='number'||typeof value.height!=='number'||!Array.isArray(value.links))throw new TypeError('Invalid source render packet');
  for(const link of value.links){
    const keys=link&&Object.keys(link);
    if(!link||Object.getPrototypeOf(link)!==Object.prototype||keys.length!==3||keys[0]!=='id'||keys[1]!=='href'||keys[2]!=='internal'||typeof link.id!=='string'||typeof link.href!=='string'||typeof link.internal!=='boolean')throw new TypeError('Invalid source render packet');
  }
  return value;
}
function nativeValue(call) {
  try { return result(call()); }
  catch (error) { if (typeof error === 'string') return result(error); throw error; }
}
// Foreign callbacks can throw any JS value, including strings. Native envelopes
// and one-element callback wrappers remain disjoint until this boundary.
function foreignValue(call) {
  try { return call(); }
  catch (error) {
    if (Array.isArray(error) && error.length === 1) throw error[0];
    if (typeof error === 'string') return result(error);
    throw error;
  }
}
// Captured storage throws retain the existing one-element arbitrary-value lane.
function capturedNativeValue(call) {
  try { return call(); }
  catch(error) {
    if(Array.isArray(error)&&error.length===1)throw error[0];
    if(typeof error==='string')return result(error);
    throw error;
  }
}
function finiteConfigJSON(config) {
  return JSON.stringify(config, (_key, value) => {
    if (typeof value === 'number' && !Number.isFinite(value)) throw new TypeError('Foreign provider Config requires finite numbers');
    return value;
  });
}
function binaryResult(call) {
  try {const exchange=decodeGraphBinary(call());return {revision:exchange.revision,graph:exchange.graph,result:exchange.metadata};}
  catch(error){if(typeof error==='string')return result(error);throw error;}
}
function layoutResult(call,baseline,revision,raw,type){
  try{return decodeLayoutOutcome(call(),baseline,revision,raw,type);}
  catch(error){if(typeof error==='string')return result(error);throw error;}
}
// Additive retained render packet. Every outcome still carries the full graph.
// Decode/admit both pieces before publication; original callback errors remain
// the facade's highest-priority failure after committed graph restoration.
function arenaRenderResult(call,revision,raw,type) {
  try {
    const response=call();
    if(response instanceof Uint8Array){const exchange=decodeGraphBinary(response);return {revision:exchange.revision,graph:exchange.graph,result:exchange.metadata};}
    const invalid=()=>{throw new TypeError('Invalid arena render packet');};
    if(!Array.isArray(response)||response.length!==5||response[0]!==1||!Number.isSafeInteger(response[1])||response[1]<0||response[1]>64*1024*1024||!(response[2] instanceof Uint8Array)||(response[3]!==0&&response[3]!==1))invalid();
    const bytes=response[2],metadataBytes=response[1];
    if(bytes.length+metadataBytes>64*1024*1024)invalid();
    const exchange=decodeGraphBinary(bytes);
    if(new DataView(bytes.buffer,bytes.byteOffset,bytes.byteLength).getUint32(20,true)!==0||BigInt(exchange.revision)!==BigInt(revision)+1n)invalid();
    const outcome={revision:exchange.revision,graph:exchange.graph};
    if(response[3]===1){outcome.factoryError=response[4];return outcome;}
    const packet=response[4];
    if(!Array.isArray(packet)||packet.length!==3||packet[0]!==1||packet[1]!==metadataBytes)invalid();
    const value=packet[2],keys=raw?['svg','width','height','links']:['type','svg','width','height','links'],actual=value&&Object.keys(value);
    if(!value||Object.getPrototypeOf(value)!==Object.prototype||actual.length!==keys.length||!keys.every((key,index)=>actual[index]===key)||typeof value.svg!=='string'||typeof value.width!=='number'||typeof value.height!=='number'||!Array.isArray(value.links)||(!raw&&value.type!==type))invalid();
    for(const link of value.links){const keys=link&&Object.keys(link);if(!link||Object.getPrototypeOf(link)!==Object.prototype||keys.length!==3||keys[0]!=='id'||keys[1]!=='href'||keys[2]!=='internal'||typeof link.id!=='string'||typeof link.href!=='string'||typeof link.internal!=='boolean')invalid();}
    outcome.result={Ok:value};return outcome;
  }catch(error){if(typeof error==='string')return result(error);throw error;}
}
// Only these pinned renderers pass the mono family into defaultMeasurer.
function builtinMeasurer(options,type) {
  return defaultMeasurer(options.fontFamily,
    type==='eventmodeling'||type==='railroad'?options.fontFamilyMono:undefined);
}
function renderedValue(value) {
  if(value&&typeof value.svg==='string'&&value.$numbers) {
    const bits=new DataView(new ArrayBuffer(8));
    for(const [key,hex] of Object.entries(value.$numbers)) {
      if(key!=='width'&&key!=='height')throw new Error('Invalid dimension transport');
      bits.setBigUint64(0,BigInt('0x'+hex));value[key]=bits.getFloat64(0);
    }
    delete value.$numbers;
  }
  return value;
}
function renderError(error) {
  return error instanceof RangeError?fromNativeError({name:'PeleError',message:'Diagram is nested too deeply.',code:'limit',type:null,line:0,column:0,snippet:''}):error;
}

// Disposed bridges may stay alive until the current WeakRef job ends. Drop
// transport storage without changing caller-owned objects or allocating empty
// containers. Keep registry identity for helpers borrowed by another document.
function disposedBridgeInvoke() { throw new TypeError('Public model document is disposed'); }
function releaseDisposedBridge(bridge) {
  releaseLiveRenderValues(bridge);
  releaseBridgeObservation(bridge);
  bridge.graph = null;
  bridge.strings = null;
  bridge.stringIds = null;
  bridge.objects = null;
  bridge.root = null;
  bridge.rootBinding = undefined;
  bridge.liveFields?.release();
  bridge.liveFields = null;
  bridge.hostLedger = null;
  bridge.hostIds = null;
  bridge.hostValues = null;
  // Break adoption closures that can otherwise retain the model environment.
  bridge.invoke = disposedBridgeInvoke;
  bridge.activeInvoke = undefined;
}

function operationError(error, bridge) {
  if (error.kind === 'pele') return fromNativeError(error.value);
  if (error.kind === 'peleUtf16') {
    const value = error.value;
    return fromNativeError({...value, message: stringValue(value.message),
      type: value.type == null ? null : stringValue(value.type), snippet: stringValue(value.snippet)});
  }
  if (error.kind === 'javaScript') {
    const classes = {Error, TypeError, RangeError, ReferenceError, SyntaxError, EvalError, URIError};
    const constructor = classes[error.value.name];
    if (!constructor) throw new TypeError('Unknown native JavaScript exception class');
    return new constructor(stringValue(error.value.message));
  }
  if (error.kind === 'hostException') return bridge.hostValues[error.value];
  return new TypeError('Unknown native method exception');
}
/** Implemented built-ins. Core rendering remains Rust; this adapter owns model lifetimes only. */
export async function loadRenderer(moduleOrPath) {
  await initialize(moduleOrPath === undefined ? undefined : { module_or_path: moduleOrPath });
  const [{allDiagramEntries},{loadDiagramEntries}] = await Promise.all([import('./all-diagram-entries.js'),import('./diagram-entries.js')]);
  await loadDiagramEntries(moduleOrPath);
  return createRenderer(new NativeRegistry(true),allDiagramEntries);
}

// Internal host factory: one native registration owner, independent model lifetimes.
export function createRenderer(nativeRegistry, canonicalProviders = []) {
  const capturedSankey=capturedSankeyAvailable(nativeRegistry,nativeCapabilities);
  const liveProviders=new Map(canonicalProviders.map(provider=>[provider.type,{provider,native:true}]));
  function syncProviders(){for(const [type,entry] of liveProviders){const native=builtinDiagramType(entry.provider)===type;if(native!==entry.native){if(native)registryCall(()=>nativeValue(()=>{nativeRegistry.register_builtin(type);return '{"version":2,"ok":true,"result":null}';}));else registryCall(()=>foreignValue(()=>nativeRegistry.register_foreign(type,entry.provider)));entry.native=native;}}}
  const documents = new WeakMap(), constructedDocuments = new WeakMap(), rawDisposed = new WeakSet(), registry = new ModelRegistry();
  const rawConstructors = new Map();
  // Native keys identify captured immutable metadata, scoped to this renderer.
  const schemaRegistrations = new Map();
  // Native baseline authority is independent of live host equality. Tokens are
  // replaced before observations/publications, including attempts that throw.
  const acceptedBaselines = new WeakMap();
  function acceptedBaseline(entry) {
    const receipt = acceptedBaselines.get(entry);
    return receipt && receipt.revision === entry.revision &&
      receipt.token === bridgeObservationToken(entry.bridge) && receipt.graph === entry.bridge.graph &&
      receipt.root === entry.bridge.root ? receipt : undefined;
  }
  function acceptBaseline(entry, token) {
    if (token && token === bridgeObservationToken(entry.bridge))
      acceptedBaselines.set(entry, {token, revision:entry.revision, graph:entry.bridge.graph, root:entry.bridge.root});
    else acceptedBaselines.delete(entry);
  }
  const finalizer = typeof FinalizationRegistry === 'function' ? new FinalizationRegistry(handle => handle.free()) : null;
  let registryBusy = false;
  const registryCall = operation => {
    if (registryBusy) throw new TypeError('Registry is already in use');
    registryBusy = true; try { return operation(); } finally { registryBusy = false; }
  };
  function nativeModelCall(entry, run) {
    if(entry.nativeBusy)throw new TypeError("Public model document is already in use");
    entry.nativeBusy=true;try{return run();}finally{entry.nativeBusy=false;}
  }
  function nativeDetachedCall(run) {
    try { return run(); }
    catch (error) { if (typeof error === 'string') return result(error); throw error; }
  }
  function runDetachedRender(entry, job, raw, width, icons, clock, inline, dispatcher) {
    let original, originalFailed = false, error, failed = false, ack, outcome;
    const capture = value => { if (!originalFailed) { originalFailed = true; original = value; } };
    const resource = callable => callable && ((...args) => {
      try { return callable(...args); } catch (value) { capture(value); throw value; }
    });
    const unlocked = run => {
      entry.nativeBusy = false;
      try { return run(); } finally { entry.nativeBusy = true; }
    };
    const fail = value => { if (!failed) { failed = true; error = value; } };
    entry.detachedDepth = (entry.detachedDepth ?? 0) + 1;
    try {
      try {
        unlocked(() => job.run(entry.bridge.root, liveRenderRead(capture, entry.bridge), resource(width),
          resource(icons), resource(clock), resource(inline), dispatcher));
      } catch (value) { fail(value); }
      // Every dispatched frame finishes, including resource/read failures. This
      // short call has no user callback and acknowledges the latest revision.
      let revision, finished=false;
      try {
        const receipt = acceptedBaseline(entry);
        ack = nativeDetachedCall(() => entry.handle.finish_detached_render(job));
        revision = ack.revision;
        const keepsBaseline = receipt && acceptedBaseline(entry) === receipt &&
          BigInt(revision) === BigInt(receipt.revision) + 1n;
        entry.revision = revision;
        if (keepsBaseline) acceptBaseline(entry, receipt.token);
        else acceptedBaselines.delete(entry);
        finished=true;
      } catch (value) { acceptedBaselines.delete(entry); fail(value); }
      if (finished) {
        // The factory may run newer helpers. Never restore an outer receipt
        // after it. A malformed returned transport conservatively loses proof.
        let response, materialized=false;
        try { response=unlocked(() => ack.materialize()); materialized=true; }
        catch (value) { fail(value); }
        if (materialized) try { outcome=detachedRenderResult(response,revision,raw,entry.rawType); }
        catch (value) { acceptedBaselines.delete(entry); fail(value); }
      }
      if (originalFailed) throw original;
      if (failed) throw error;
      return outcome;
    } finally {
      entry.confirmedGraph = false;
      entry.detachedDepth--;
      try { ack?.free(); } finally { job.free(); }
    }
  }
  // bridge.graph tracks observations, including edits rejected by native apply.
  // Only a completely published native outcome can acknowledge that baseline.
  function prepareMutableInput(entry, values = []) {
    const receipt = acceptedBaseline(entry);
    entry.confirmedGraph = false;
    if(receipt&&entry.unchangedApplyIsNoop&&values.length===0) {
      if(entry.bridge.matchesPublishedGraph()&&acceptedBaseline(entry)===receipt)
        return {graph:entry.bridge.graph,values:[]};
    }
    const prior = acceptedBaseline(entry), baseline = entry.bridge.graph;
    const input = entry.bridge.snapshot(values, entry.sourceIngressIdentity ? 'packed' : true), delta = input.delta;
    const token = completedObservationToken(entry.bridge,input);
    const unchanged = prior && token && token===bridgeObservationToken(entry.bridge) && entry.unchangedApplyIsNoop &&
      input.graph.root.kind === baseline.root.kind && input.graph.root.value === baseline.root.value &&
      delta.nodes.length === 0 && delta.strings.length === 0 &&
      (input.graph instanceof LayoutGraph ? input.graph.nodeCount === baseline.nodeCount : input.graph.nodes.length === baseline.nodes.length) && input.graph.strings.length === baseline.strings.length;
    if (!unchanged) {
      const sparse=entry.sourceIngressIdentity?encodeSourceInput(entry.revision,input.graph,entry.sourceIngressIdentity):null;
      entry.revision=result(sparse===null?entry.handle.apply_binary(encodeGraphBinary(entry.revision,input.graph)):entry.handle.apply_source_input(sparse));
      // Only sealed source ingress promises this exact observed baseline.
      // Other providers obtain authority from their complete publication.
      if(entry.sourceIngressIdentity)acceptBaseline(entry,token);
    } else {
      // Existing no-op policy proved the completed observation unchanged. The
      // successful snapshot token also rejects nested superseding observations.
      acceptBaseline(entry,token);
    }
    return input;
  }
  function publishMutableOutcome(entry, outcome) {
    entry.revision = outcome.revision;
    acceptedBaselines.delete(entry);
    entry.bridge.reconcile(outcome.graph);
    entry.confirmedGraph = true;
    acceptBaseline(entry,completedPublicationToken(entry.bridge,outcome.graph));
    if(Object.hasOwn(outcome,'factoryError'))throw outcome.factoryError;
  }
  function foreignParse(source, options) {
    syncProviders();
    if (!nativeRegistry?.has_foreign()) return null;
    const limits = parseLimitOptions(options);
    const handle = registryCall(() => foreignValue(() => nativeRegistry.parse_foreign(source, limits.json,
      limits.sourceKind, limits.sourceValue, limits.edgeKind, limits.edgeValue)));
    if (!handle) return null;
    try {
      if (options.config !== undefined) finiteConfigJSON(options.config);
      const [model, provider, config] = foreignValue(() => handle.take_values());
      return {handle, model, provider, config, foreign: true, busy: false, disposed: false};
    } catch (error) { handle.free(); throw error; }
  }
  function foreignRender(entry, model, options) {
    if (entry.disposed) throw new TypeError('Public model document is disposed');
    if (entry.busy) throw new TypeError('Public model document is already in use');
    entry.busy = true;
    try {
      const originalIcons = options.icons;
      const callbackOptions = {...options, responsive: options.responsive, icons: originalIcons && (name => originalIcons(foreignValue(() => foreign_decode_entities(name))))};
      // Passive parse options remain visible to custom renderers. Only their
      // native render-policy decoding is omitted on this retained-model path.
      const numeric = renderNumericOptions({...options, limit: undefined, maxEdges: undefined, config: undefined});
      return withBuiltinIcons(callbackOptions.icons,()=>foreignValue(() => (entry.policy??entry.handle).render(model, entry.provider, entry.config, finiteConfigJSON(entry.config), callbackOptions,
        numeric.json, numeric.mask, numeric.numbers, numeric.outputKind, numeric.output)));
    } finally { entry.busy = false; }
  }
  function hostOperation(entry,kind,values,call) {
    const owner=entry.hostStore,token=owner.begin(kind,values,kind==='render'&&typeof entry.handle.render_host_frame_packet==='function');
    let original,failed=false,outcome,error,thrown=false,json,packetBytes;
    const capture=value=>{if(!failed){failed=true;original=value;}};
    const resource=fn=>fn&&((...args)=>owner.resource(()=>{try{return fn(...args);}catch(value){capture(value);throw value;}}));
    try {
      const response=call(owner.primitive.bind(owner),capture,()=>owner.structure(token),resource,token.audit.args,()=>{const frame=token.audit.readFrame;token.audit.readFrame=undefined;return frame;});
      if(typeof response==='string'){
        json=response;outcome=JSON.parse(json);
        if(kind==='method'&&outcome&&Object.hasOwn(outcome,'Ok'))outcome.Ok=nativeAtomValue(owner,outcome.Ok);
      }else{
        if(kind==='method'||!Array.isArray(response)||response.length!==3||response[0]!==1||!Number.isSafeInteger(response[1])||response[1]<0||response[1]>64*1024*1024)throw new TypeError('Invalid host render packet');
        const value=response[2],keys=kind==='raw'?['svg','width','height','links']:['type','svg','width','height','links'],actualKeys=value&&Object.keys(value);
        if(!value||Object.getPrototypeOf(value)!==Object.prototype||actualKeys.length!==keys.length||!keys.every((key,index)=>actualKeys[index]===key)||typeof value.svg!=='string'||typeof value.width!=='number'||typeof value.height!=='number'||!Array.isArray(value.links)||(kind!=='raw'&&value.type!=='er'))throw new TypeError('Invalid host render packet');
        for(const link of value.links)if(!link||Object.getPrototypeOf(link)!==Object.prototype||Object.keys(link).length!==3||typeof link.id!=='string'||typeof link.href!=='string'||typeof link.internal!=='boolean')throw new TypeError('Invalid host render packet');
        packetBytes=response[1];outcome={Ok:value};
      }
    }
    catch(value){thrown=true;error=value;}
    try {owner.finish(token,packetBytes??(json===undefined?0:new TextEncoder().encode(json).length));}catch(value){if(!thrown){thrown=true;error=value;}}
    if(failed)throw original;
    if(thrown){if(typeof error==='string')return result(error);throw error;}
    if(!outcome||Object.hasOwn(outcome,'Ok')===Object.hasOwn(outcome,'Err'))throw new TypeError('Invalid host operation outcome');
    if(Object.hasOwn(outcome,'Err'))throw operationError(outcome.Err,owner);
    return outcome.Ok;
  }
  function adoptHostEr(handle,owner,provider,preparedType,callerRoot=false) {
    const model=owner.root,rawType='er';
    const entry={handle,rawType,provider,preparedType,disposed:false,hostStore:owner};
    owner.invoke=(method,receiver,args)=>hostOperation(entry,'method',[receiver,...args],(primitive,capture,finish,_resource,values)=>handle.invoke_host(method,primitive,capture,finish,values));
    owner.phase='idle';
    if(callerRoot)documents.set(model,entry); // Preserve native weak-key error before render admission.
    if(owner.registration.documents.length>256)owner.registration.documents=owner.registration.documents.filter(reference=>{const value=reference.deref();return value&&!value.disposed;});
    owner.registration.documents.push(new WeakRef(owner));if(!callerRoot)documents.set(model,entry);
    ownRawModel(model,rawType,(config,options)=>renderer.renderRaw(rawType,model,config,options),()=>renderer.dispose(model));
    finalizer?.register(model,handle,model);
    installConstructors(handle,owner.registration,rawType);
    return model;
  }
  function nativeAtomValue(owner,atom) {
    if(atom.kind==='undefined')return undefined;if(atom.kind==='null')return null;
    if(atom.kind==='bool')return atom.value;
    const tags={number:4,string:5,ref:6,builtinFunction:7,hostReference:8};return owner.value([tags[atom.kind],atom.value]);
  }
  function capturedConstructorFactory(capability, registration, rawType, liveDefinitions) {
    const transfers=typeof capability.argument_transfers==='function'?Array.from(capability.argument_transfers()):null;
    if(transfers&&(transfers.length>2||transfers.some(mode=>mode!==0&&mode!==1)))throw new TypeError('Invalid native constructor argument metadata');
    return (receiver,args)=>{
      const hostLedger=createHostLedger(),construction={receiver,prototypeId:capability.prototype_id(),registration,hostLedger,liveDefinitions,deferLiveBinding:!!transfers};
      if(transfers&&typeof capability.begin_construct==='function'){
        const values=[[0,undefined],[0,undefined]];
        for(let index=0;index<transfers.length;index++){
          const value=args[index];
          values[index]=transfers[index]===0&&value!==undefined?[5,retainHostValue(hostLedger,value)]:hostReply({hostLedger},value);
        }
        const job=foreignValue(()=>capability.begin_construct(...values[0],...values[1]));
        let model;
        try{model=adoptMutable(job,rawType,undefined,construction);}
        catch(error){job.free();throw error;}
        const entry=constructedDocuments.get(model);
        return nativeModelCall(entry,()=>runDetachedInvocation(entry,job,true));
      }
      const first=capability.ignores_arguments()?undefined:args[0];
      const mutable=foreignValue(()=>capability.ignores_arguments()?capability.construct():capability.construct_with_host_reference(first===undefined?undefined:retainHostValue(hostLedger,first)));
      try{return adoptMutable(mutable,rawType,undefined,construction);}
      catch(error){mutable.free();throw error;}
    };
  }
  function installConstructors(mutable, registration, rawType) {
    const ownedDefinitions=[];
    if(typeof mutable.capture_constructor!=='function')return ownedDefinitions;
    for(const prototype of registration.schema.prototypes){
      // Capture from THIS document even when its schema already has a shell.
      // Shared schema metadata cannot authorize live fields on another creator.
      const capability=mutable.capture_constructor(prototype.id);
      if(!capability)continue;
      const liveDefinitions=typeof capability.live_fields==='function'?JSON.parse(capability.live_fields()):[];
      if(!capability.ignores_arguments()&&typeof capability.construct_with_host_reference!=='function'){capability.free();continue;}
      ownedDefinitions.push(...liveDefinitions);
      let rawFactories=rawConstructors.get(rawType);
      if(!rawFactories){rawFactories=new Map();rawConstructors.set(rawType,rawFactories);}
      // Keep the actual captured entry even when its class shell is shared.
      // A live shell may retain an older registration; never free its factory.
      rawFactories.set(prototype.id,capability);
      if(registration.constructors.has(prototype.id))continue;
      registration.constructors.set(prototype.id,capturedConstructorFactory(capability,registration,rawType,liveDefinitions));
      if(typeof capability.arity==='function'){
        const constructor=Object.getOwnPropertyDescriptor(registration.prototypes.get(prototype.id),'constructor').value;
        Object.defineProperty(constructor,'length',{value:capability.arity(),configurable:true});
      }
    }
    return ownedDefinitions;
  }
  function runDetachedInvocation(entry,job,construction=false){
    let replyState=0,valueTag=0,value,original,originalFailed=false,observationFailed=false,finishedGraph,errorContext;
    const temporaryValues=new Set();
    const bridge=entry.bridge;
    const unlocked=run=>{entry.nativeBusy=false;try{return run();}finally{entry.nativeBusy=true;}};
    const retainFailure=error=>{
      if(!originalFailed){originalFailed=true;original=error;}
      replyState=2;valueTag=0;value=bridge.hostException(error).value;
    };
    entry.detachedDepth=(entry.detachedDepth??0)+1;
    acceptedBaselines.delete(entry);
    try{
      for(;;){
        const outcome=binaryResult(()=>construction?job.resume(replyState,valueTag,value):entry.handle.resume_detached_invocation(job,replyState,valueTag,value));
        entry.revision=outcome.revision;
        if(outcome.result?.complete===true){errorContext=outcome.result.errorContext;break;}
        const operation=outcome.result?.host, resourceStep=outcome.result?.resource===true;
        if((!operation&&!resourceStep)||(operation&&resourceStep))throw new TypeError('Invalid detached invocation step');
        // Native mutation is visible before the next user operation. Publishing
        // itself may reenter through caller descriptors; no native borrow lives.
        unlocked(()=>publishMutableOutcome(entry,outcome));
        let returned,thrown,failed=false;
        try {
          if(resourceStep){
            const resource=foreignValue(()=>job.take_resource());
            try {
              unlocked(()=>resource.run(modelHostFunctions(bridge,operationError)));
              foreignValue(()=>job.accept_resource(resource));
            } finally {resource.free();}
          }else returned=unlocked(()=>performHostOperation(bridge,operation));
        } catch(error){failed=true;thrown=error;}
        try{prepareMutableInput(entry);}
        catch(error){observationFailed=true;if(!failed){failed=true;thrown=error;}}
        if(failed)retainFailure(thrown);
        else if(resourceStep){replyState=4;valueTag=0;value=undefined;}
        else {replyState=1;[valueTag,value]=operation.kind==='call'&&operation.result==='retain'?[5,retainJobValue(bridge.hostLedger,returned,temporaryValues)]:hostReply(bridge,returned);}
      }
      let outcome;
      if(construction){
        const packet=foreignValue(()=>job.finish());
        if(!Array.isArray(packet)||packet.length!==2||!packet[0]||typeof packet[0].snapshot_binary!=='function')throw new TypeError('Invalid native constructor completion');
        try{outcome=binaryResult(()=>packet[1]);}catch(error){packet[0].free();throw error;}
        // Keep the provisional receiver/owner: callbacks may already retain it.
        // Future helpers resolve the completed handle through this same entry.
        finalizer?.unregister(bridge.root);
        entry.handle=packet[0];
        finalizer?.register(bridge.root,entry.handle,bridge.root);
        entry.unchangedApplyIsNoop=entry.handle.unchanged_public_graph_is_noop?.()===true;
        installConstructors(entry.handle,bridge.registration,entry.rawType);
      }else outcome=binaryResult(()=>entry.handle.finish_detached_invocation(job));
      finishedGraph=outcome.graph;
      entry.revision=outcome.revision;
      if(!observationFailed)unlocked(()=>publishMutableOutcome(entry,outcome));
      else acceptedBaselines.delete(entry);
      if(construction)bridge.liveFields?.bind(bridge.root);
      if(originalFailed)throw original;
      if(Object.hasOwn(outcome.result,'Err'))throw errorContext?materializeHostError(bridge,errorContext):operationError(outcome.result.Err,bridge);
      return construction?bridge.root:bridge.value(outcome.result.Ok);
    }catch(error){if(originalFailed)throw original;throw error;}
    finally{
      entry.confirmedGraph=false;entry.detachedDepth--;
      if(construction&&entry.handle===job){
        // A transport failure cannot leave an escaped receiver calling freed WASM.
        finalizer?.unregister(bridge.root);acceptedBaselines.delete(entry);
        constructedDocuments.delete(bridge.root);rawDisposed.add(bridge.root);
        entry.disposed=true;bridge.disposed=true;releaseDisposedBridge(bridge);
      }
      try{job.free();}finally{if(bridge.hostLedger)releaseJobValues(bridge.hostLedger,temporaryValues,finishedGraph);}
    }
  }
  function adoptMutable(mutable, preparedType, existingBridge, construction) {
          let sourceSnapshot;
          try{sourceSnapshot=mutable.snapshot_source_layout?.();}catch(error){if(typeof error==='string')return result(error);throw error;}
          const snapshot = sourceSnapshot == null ? binaryResult(()=>mutable.snapshot_binary()) : decodeLayoutOutcome(sourceSnapshot);
          // Raw-model adoption already has a bridge. Validate its native metadata
          // afresh before reusing it; never trust a cache hit on this path.
          const schemaKey = existingBridge ? undefined : mutable.schema_cache_key?.();
          let registration = typeof schemaKey === 'string' ? schemaRegistrations.get(schemaKey) : undefined;
          if(!registration) {
            registration = registry.registration(result(mutable.schema()));
            if(existingBridge && registration !== existingBridge.registration) throw new TypeError('Conflicting native model schema identity');
            if(typeof schemaKey === 'string') schemaRegistrations.set(schemaKey,registration);
          }
          if(construction&&registration!==construction.registration)throw new TypeError('Conflicting native constructor schema identity');
          const schema = registration.schema;
          let rootBinding;
          if(construction){
            const root=snapshot.graph.root,node=root?.kind==='ref'&&snapshot.graph.nodes?.[root.value];
            if(!node||node.kind!=='instance'||node.prototype!==construction.prototypeId)throw new TypeError('Invalid native constructor root');
            rootBinding={object:construction.receiver,id:root.value,prototypeId:construction.prototypeId,prototype:Object.getPrototypeOf(construction.receiver)};
          }
          const rawType=mutable.entry_type(), live=preparedType===undefined?liveProviders.get(rawType):undefined;
          const entry = {handle: mutable, revision: snapshot.revision, disposed: false, preparedType,rawType,provider:live?.provider, confirmedGraph: false, unchangedApplyIsNoop: mutable.unchanged_public_graph_is_noop?.() === true};
          // Captured concrete provider capability, never selected by schema.
          entry.sourceIngressIdentity=snapshot.graph instanceof LayoutGraph&&typeof mutable.apply_source_input==='function'
            ?mutable.source_ingress_identity?.():undefined;
          const invoke = (method, receiver, argumentsValue) => nativeModelCall(entry, () => {
            const mutable=entry.handle;
            if (entry.disposed) throw new TypeError('Public model document is disposed');
            bridge.liveFields?.bind(receiver,method);
            const input = prepareMutableInput(entry, [receiver, ...argumentsValue]);
            const call=JSON.stringify({method,receiver:input.values[0],arguments:input.values.slice(1)}),functions=modelHostFunctions(bridge,operationError);
            acceptedBaselines.delete(entry);
            if(typeof mutable.prepare_detached_invocation==='function'){
              const job=nativeDetachedCall(()=>mutable.prepare_detached_invocation(entry.revision,call));
              if(job)return runDetachedInvocation(entry,job);
            }
            const outcome = bridge.graph instanceof LayoutGraph
              ? layoutResult(()=>mutable.invoke_source_layout_with_functions(entry.revision,call,functions),bridge.graph,entry.revision)
              : binaryResult(()=>mutable.invoke_binary_with_functions(entry.revision,call,functions));
            publishMutableOutcome(entry, outcome);
            if (Object.hasOwn(outcome.result, 'Err')) throw operationError(outcome.result.Err, bridge);
            return bridge.value(outcome.result.Ok);
          });
          const bridge = existingBridge ?? new ModelBridge(snapshot.graph,schema,registry,invoke,registration,rootBinding,construction?.hostLedger,construction?.liveDefinitions,construction?.deferLiveBinding);
          if(existingBridge){bridge.invoke=invoke;bridge.reconcile(snapshot.graph);}
          entry.bridge = bridge;
          entry.confirmedGraph = true;
          acceptBaseline(entry,completedPublicationToken(bridge,snapshot.graph));
          const model = bridge.root;
          (construction?constructedDocuments:documents).set(model, entry);
          ownRawModel(model,rawType,(config,options)=>renderer.renderRaw(rawType,model,config,options),()=>renderer.dispose(model));
          finalizer?.register(model, mutable, model);
          const liveDefinitions=installConstructors(mutable,registration,rawType);
          if(liveDefinitions.length&&!bridge.liveFields){bridge.liveFields=new LiveFieldBindings(bridge,liveDefinitions);bridge.liveFields.bind(model);}
          return model;
  }
  function adoptCaptured(handle,epoch,preparedType,existingOwner) {
    // Native already completed initial snapshot/schema/+17 frame admission.
    // Registration still occurs here, after that gate, as in adoptMutable.
    let graph;
    try {
      const schemaKey=existingOwner?undefined:handle.schema_cache_key?.();
      let registration=typeof schemaKey==='string'?schemaRegistrations.get(schemaKey):undefined;
      if(!registration){
        registration=registry.registration(result(handle.schema()));
        if(existingOwner&&registration!==existingOwner.registration)throw new TypeError('Conflicting native model schema identity');
        if(typeof schemaKey==='string')schemaRegistrations.set(schemaKey,registration);
      }
      const rawType=handle.entry_type(),live=preparedType===undefined?liveProviders.get(rawType):undefined;
      const entry={handle,revision:handle.revision(),disposed:false,preparedType,rawType,provider:live?.provider,confirmedGraph:false,unchangedApplyIsNoop:handle.unchanged_public_graph_is_noop?.()===true};
      const owner=existingOwner??new CapturedOwner(registration,registry,capturedNativeValue,result,operationError,nativeCapabilities.native_canonical_memory(),handle);
      entry.bridge=owner;entry.capturedOwner=owner;
      const admitted=epoch;epoch=undefined;graph=owner.view(admitted);
      owner.attach(entry);owner.reconcile(graph);entry.confirmedGraph=true;
      const model=owner.root;
      documents.set(model,entry);
      ownRawModel(model,rawType,(config,options)=>renderer.renderRaw(rawType,model,config,options));
      finalizer?.register(model,handle,model);
      rememberCapturedModel(model,owner,existingOwner?'raw':preparedType===undefined?'source':'prepared');
      return model;
    } finally {graph?.release();if(epoch){try{epoch.close();}finally{epoch.free?.();}}}
  }
  const renderer = {
    capabilities() { return nativeRegistry ? nativeValue(() => nativeRegistry.capabilities()) : result(builtin_capabilities()); },
    registered(type) { return nativeValue(() => nativeRegistry.registered(type)); },
    register(...providers) {
      for (const provider of providers) {
        const builtin=builtinDiagramType(provider);
        if(builtin!==undefined){registryCall(()=>nativeValue(()=>{nativeRegistry.register_builtin(builtin);return '{"version":2,"ok":true,"result":null}';}));liveProviders.set(builtin,{provider,native:true});}
        else {registryCall(() => foreignValue(() => nativeRegistry.register_foreign(provider.type, provider)));if(builtinDiagramOrigin(provider)!==undefined)liveProviders.set(provider.type,{provider,native:false});else liveProviders.delete(provider.type);}
      }
    },
    registerBuiltin(type) { registryCall(() => nativeValue(() => { nativeRegistry.register_builtin(type); return '{"version":2,"ok":true,"result":null}'; }));liveProviders.delete(type); },
    detectType(source) { return nativeValue(() => nativeRegistry.detect_type(source)); },
    supports(source) { return nativeValue(() => nativeRegistry.supports(source)); },
    close() { registryCall(() => nativeRegistry.close()); },
    // Internal low-level entry adapters. Prepared sources and raw results
    // do not cross core preprocessing.
    parsePrepared(type, source, config, title, lineOffset, maxEdges) {
      const limits=parseLimitValues({maxEdges});
      if(capturedSankey&&type==='sankey'){
        const parsed=registryCall(()=>capturedNativeValue(()=>nativeRegistry.parse_prepared_with_native_sankey(type,source,finiteConfigJSON(config),title,lineOffset,limits.edgeKind,limits.edgeValue)));
        const handle=parsed[1];
        try{return parsed[0]===1?adoptCaptured(handle,parsed[2],type):adoptMutable(handle,type);}
        catch(error){handle.free();throw error;}
      }
      if(nativeRegistry.parse_prepared_with_host_er){
        let owner,failed=false,original,parsed;
        const capture=error=>{if(!failed){failed=true;original=error;}};
        try{
          parsed=registryCall(()=>nativeRegistry.parse_prepared_with_host_er(type,source,finiteConfigJSON(config),title,lineOffset,limits.edgeKind,limits.edgeValue,(key,schema)=>{
            let registration=schemaRegistrations.get(key);
            if(!registration){if(schema===undefined)return undefined;registration=registry.registration(result(schema));schemaRegistrations.set(key,registration);}
            owner=new HostErOwner(registration,registry,undefined,()=>{throw new TypeError('Public model construction is incomplete');});
            return owner.primitive.bind(owner);
          },capture));
        }catch(error){if(failed)throw original;if(typeof error==='string')return result(error);throw error;}
        if(failed){parsed?.[1]?.free();throw original;}
        const handle=parsed[1];
        try{return parsed[0]===1?adoptHostEr(handle,owner,undefined,type):adoptMutable(handle,type);}
        catch(error){handle.free();throw error;}
      }
      const handle=registryCall(()=>{try{return nativeRegistry.parse_prepared(type,source,finiteConfigJSON(config),title,lineOffset,limits.edgeKind,limits.edgeValue);}catch(error){if(typeof error==='string')return result(error);throw error;}});
      try{return adoptMutable(handle,type);}catch(error){handle.free();throw error;}
    },
    renderRaw(type, model, config, options = {}) {
      if(rawDisposed.has(model))throw new TypeError('Public model document is disposed');
      let entry=documents.get(model)??constructedDocuments.get(model);
      if(!entry&&capturedSankey&&type==='sankey'){
        const selection=registryCall(()=>capturedNativeValue(()=>nativeRegistry.select_prepared_native_sankey(type)));
        if(selection){
          let graph,handle,epoch;
          try{
            const schema=registryCall(()=>result(selection.schema()));
            const registration=registry.registration(schema);
            // Preserve provisional registration and failure timing. Existing
            // caller objects acquire IDs but never fresh native ownership.
            handle=registryCall(()=>capturedNativeValue(()=>nativeRegistry.create_prepared_native_sankey(selection)));
            const owner=new CapturedOwner(registration,registry,capturedNativeValue,result,operationError,nativeCapabilities.native_canonical_memory(),handle);
            owner.seed(capturedNativeValue(()=>handle.initial_epoch()));
            owner.root=model;graph=owner.snapshot().graph.retain();
            epoch=registryCall(()=>{
              capturedObservedFrame(graph);
              return capturedNativeValue(()=>nativeRegistry.bind_prepared_native_sankey(selection,handle,finiteConfigJSON(config)));
            });
            const admitted=epoch;epoch=undefined;
            adoptCaptured(handle,admitted,type,owner);handle=undefined;
            entry=documents.get(model);
          }catch(error){handle?.free();throw error;}
          finally{if(epoch){try{epoch.close();}finally{epoch.free?.();}}try{graph?.releaseArguments();}finally{graph?.release();selection.free();}}
        }
      }
      if(!entry&&nativeRegistry.select_prepared_host_er&&nativeRegistry.bind_prepared_host_er){
        const selection=registryCall(()=>{try{return nativeRegistry.select_prepared_host_er(type);}catch(error){if(typeof error==='string')return result(error);throw error;}});
        if(selection){
          let owner,handle,committed=false;
          try{
            const schema=registryCall(()=>result(selection.schema()));
            const registration=registry.registration(schema);
            owner=new HostErOwner(registration,registry,model,()=>{throw new TypeError('Public model construction is incomplete');});
            const admission=owner.admitCallerRoot();
            handle=registryCall(()=>{
              // Empty metadata frame precedes config serialization. Bind then
              // rechecks registry-open before native config decoding.
              owner.callerFrameAdmission(admission);
              try{return nativeRegistry.bind_prepared_host_er(selection,finiteConfigJSON(config));}
              catch(error){if(typeof error==='string')return result(error);throw error;}
            });
            owner.validateCallerAdmission(admission);
            // The old adoptMutable snapshot adds exactly {"type":"er"}.
            owner.frameAdmission(admission,13);
            if(registry.registration(result(selection.schema()))!==registration)throw new TypeError('Conflicting native model schema identity');
            adoptHostEr(handle,owner,undefined,type,true);committed=true;
            entry=documents.get(model);
          }finally{
            selection.free();
            if(!committed){handle?.free();owner?.abandonCallerRoot();}
          }
        }
      }
      if(!entry){
        const schema=registryCall(()=>nativeValue(()=>nativeRegistry.entry_schema(type)));
        const bridge=new ModelBridge({version:1,root:{kind:'undefined'},strings:[],nodes:[]},schema,registry,()=>{throw new TypeError('Public model construction is incomplete');});
        bridge.root=model;
        const graph=bridge.snapshot().graph;
        const handle=registryCall(()=>{try{return nativeRegistry.create_prepared_model_binary(type,encodeGraphBinary('0',graph),finiteConfigJSON(config));}catch(error){if(typeof error==='string')return result(error);throw error;}});
        try{adoptMutable(handle,type,bridge);}catch(error){handle.free();throw error;}
        entry=documents.get(model);
      }
      if(entry?.hostStore){
        if(entry.rawType!==type)throw new TypeError('Raw diagram render requires a model of this diagram');
        const measurer=options.measurer??builtinMeasurer(options,type),inherited=builtinInlineIcons();
        const numeric=renderNumericOptions({...options,limit:undefined,maxEdges:undefined,config:undefined,outputLimit:undefined,nativeTheme:undefined});
        return renderedValue(hostOperation(entry,'raw',[],(primitive,capture,finish,resource)=>(entry.handle.render_raw_host_packet??entry.handle.render_raw_host).call(entry.handle,primitive,capture,finish,finiteConfigJSON(config),numeric.json,numeric.mask,numeric.numbers,resource((text,size,style)=>measurer.width(text,size,style)),resource(options.icons),resource(()=>Date.now()),resource(inherited))));
      }
      const targetCandidates=entry?.bridge&&entry.rawType!==type?rawConstructors.get(type):undefined;
      if(!entry?.bridge||(entry.rawType!==type&&!targetCandidates))throw new TypeError('Raw diagram render requires a model of this diagram');
      if(entry.disposed)throw new TypeError('Public model document is disposed');
      return nativeModelCall(entry,()=>{
      let targetRaw;
      if(targetCandidates){
        for(const capability of targetCandidates.values()){
          if(capability.accepts_raw_document(entry.handle)){targetRaw=capability;break;}
        }
        if(!targetRaw)throw new TypeError('Raw diagram render requires a model of this diagram');
      }
      let callbackFailed=false,callbackError;
      const capture=run=>{try{return run();}catch(error){callbackFailed=true;callbackError=error;throw error;}};
      const measurer=options.measurer??builtinMeasurer(options,type);
      const width=(text,size,style)=>capture(()=>measurer.width(text,size,style));
      const icons=options.icons&&(name=>capture(()=>options.icons(name)));
      const clock=()=>capture(()=>Date.now());
      const inherited=builtinInlineIcons(),inline=inherited&&(name=>capture(()=>inherited(name)));
      // These fields belong to core.parse/core.render and are passive in a raw
      // built-in renderer. Removing them also avoids traversing unused config.
      const numeric=renderNumericOptions({...options,limit:undefined,maxEdges:undefined,config:undefined,outputLimit:undefined,nativeTheme:undefined});
      try {
        if(entry.capturedOwner){
          const owner=entry.capturedOwner,input=owner.prepare(entry);input.graph.releaseArguments();
          const outcome=owner.packet(entry,'raw',()=>entry.handle.render_raw_native(entry.revision,finiteConfigJSON(config),numeric.json,numeric.mask,numeric.numbers,width,icons,clock,inline));
          if(callbackFailed)throw callbackError;
          if(Object.hasOwn(outcome,'Err'))throw operationError(outcome.Err,owner);
          return renderedValue(outcome.Ok);
        }
        prepareMutableInput(entry);
        const dispatcher=modelHostFunctions(entry.bridge,operationError,()=>{const error=callbackFailed?{error:callbackError}:null;callbackFailed=false;callbackError=undefined;return error;});
        const configJSON=finiteConfigJSON(config);
        const job=!targetRaw&&typeof entry.handle.prepare_detached_render_raw==='function'
          ?nativeDetachedCall(()=>entry.handle.prepare_detached_render_raw(entry.revision,configJSON,numeric.json,numeric.mask,numeric.numbers)):undefined;
        if(job){
          const outcome=runDetachedRender(entry,job,true,width,icons,clock,inline,dispatcher);
          if(Object.hasOwn(outcome,'Err'))throw operationError(outcome.Err,entry.bridge);
          return renderedValue(outcome.Ok);
        }
        const packet=entry.handle.render_raw_operation_packet_with_functions;
        acceptedBaselines.delete(entry);
        const outcome=targetRaw
          ?arenaRenderResult(()=>targetRaw.render_raw_operation_packet_with_functions(entry.handle,entry.revision,configJSON,numeric.json,numeric.mask,numeric.numbers,width,icons,clock,inline,dispatcher),entry.revision,true,type)
          :entry.bridge.graph instanceof LayoutGraph
          ?layoutResult(()=>entry.handle.render_raw_operation_source_layout_with_functions(entry.revision,configJSON,numeric.json,numeric.mask,numeric.numbers,width,icons,clock,inline,dispatcher),entry.bridge.graph,entry.revision,true,entry.rawType)
          :typeof packet==='function'
          ?arenaRenderResult(()=>packet.call(entry.handle,entry.revision,configJSON,numeric.json,numeric.mask,numeric.numbers,width,icons,clock,inline,dispatcher),entry.revision,true,entry.rawType)
          :binaryResult(()=>entry.handle.render_raw_operation_binary_with_functions(entry.revision,configJSON,numeric.json,numeric.mask,numeric.numbers,width,icons,clock,inline,dispatcher));
        publishMutableOutcome(entry, outcome);
        if(callbackFailed)throw callbackError;
        if(Object.hasOwn(outcome.result,'Err'))throw operationError(outcome.result.Err,entry.bridge);
        return renderedValue(outcome.result.Ok);
      } catch(error){if(callbackFailed)throw callbackError;throw error;}
      });
    },
    parse(source, options = {}) {
      let foreign;
      try { foreign = foreignParse(source, options); } catch (error) { throw renderError(error); }
      if (foreign) {
        const model = foreign.model;
        delete foreign.model;
        if (typeof model === 'object' && model !== null) {
          const old = documents.get(model);
          if (old?.busy) { foreign.handle.free(); throw new TypeError('Public model document is already in use'); }
          if (old) { finalizer?.unregister(model); old.handle.free(); }
          documents.set(model, foreign); finalizer?.register(model, foreign.handle, model);
        } else foreign.handle.free();
        return model;
      }
      let document;
      try {
        if (nativeRegistry) {
          const limits = parseLimitOptions(options);
          if(capturedSankey||nativeRegistry.parse_with_host_er){
            let owner,failed=false,original;
            const capture=error=>{if(!failed){failed=true;original=error;}};
            let parsed;
            const make=(key,schema)=>{
              let registration=schemaRegistrations.get(key);
              if(!registration){if(schema===undefined)return undefined;registration=registry.registration(result(schema));schemaRegistrations.set(key,registration);}
              owner=new HostErOwner(registration,registry,undefined,()=>{throw new TypeError('Public model construction is incomplete');});
              return owner.primitive.bind(owner);
            };
            try{parsed=capturedSankey
              ?capturedNativeValue(()=>nativeRegistry.parse_with_native_storage(source,limits.json,limits.sourceKind,limits.sourceValue,limits.edgeKind,limits.edgeValue,make,capture))
              :nativeRegistry.parse_with_host_er(source,limits.json,limits.sourceKind,limits.sourceValue,limits.edgeKind,limits.edgeValue,make,capture);
            }catch(error){if(failed)throw original;throw error;}
            if(failed){parsed?.[1]?.free();throw original;}
            if(capturedSankey&&parsed[0]===2){
              const handle=parsed[1];
              try{return adoptCaptured(handle,parsed[2]);}catch(error){handle.free();throw error;}
            }
            if(parsed[0]===1)return adoptHostEr(parsed[1],owner,liveProviders.get('er')?.provider);
            document=parsed[1];
          }else document = nativeRegistry.parse(source, limits.json, limits.sourceKind, limits.sourceValue, limits.edgeKind, limits.edgeValue);
        } else document = parse_v2(source, JSON.stringify(options));
      }
      catch (error) { if (typeof error === 'string') return result(error); throw error; }
      try {
        const mutable = nativeRegistry ? document : document.to_public();
        if (mutable) {
          if (document !== mutable) document.free(); document = mutable;
          return adoptMutable(mutable);
        }
        const projection = result(document.model());
        const model = projection.model;
        if (projection.type === 'gitGraph') {
          model.commits = new Map(model.commits);model.branches = new Map(model.branches);
          if(model.head)model.head=model.commits.get(model.head.id)??model.head;
        }
        if (projection.type === 'architecture') for(const key of ['nodes','groups','registered'])model[key]=new Map(model[key]);
        if (projection.type === 'sankey') for(const link of model.links) {
          link.source=model.nodes[link.source.index];link.target=model.nodes[link.target.index];
        }
        if (projection.type === 'usecase') {
          for(const key of ['actors','useCases','systemBoundaries','notes','jsonNodes','classDefs'])model[key]=new Map(model[key]);
          for(const node of model.jsonNodes.values())node.propertyOrder=new Map(node.propertyOrder);
        }
        if (projection.type === 'venn') for(const style of model.styles)style.styles=new Map(style.styles);
        if (projection.type === 'cynefin') model.domains = new Map(model.domains);
        if (projection.type === 'agentflow') for(const key of ['nodes','connectors','classes','tooltips'])model[key]=new Map(model[key]);
        if (projection.type === 'requirement') for(const key of ['requirements','elements','classes'])model[key]=new Map(model[key]);
        if (projection.type === 'block') for(const key of ['classes','byId'])model[key]=new Map(model[key]);
        if (projection.type === 'treemap') model.classes = new Map(model.classes);
        if (projection.type === 'c4') {
          const pending = [model];
          while (pending.length) {
            const value = pending.pop();
            if (!value || typeof value !== 'object') continue;
            if (Object.hasOwn(value, 'extra')) value.extra = new Map(value.extra);
            for (const [key, child] of Object.entries(value)) if (key !== 'extra') {
              if (Array.isArray(child)) pending.push(...child);
              else if (child && typeof child === 'object') pending.push(child);
            }
          }
        }
        if (projection.type === 'pie') model.sections = new Map(model.sections);
        if (projection.type === 'quadrantChart') model.classes = new Map(model.classes);
        if (projection.type === 'state') for (const key of ['states', 'classes', 'links']) model[key] = new Map(model[key]);
        if (projection.type === 'gantt') {
          model.links = new Map(model.links);
          model.byId = new Map(model.byId);
          for (const task of [...model.tasks, ...model.byId.values()]) for (const key of ['startTime', 'endTime', 'renderEndTime']) if (typeof task[key] === 'string') task[key] = new Date(task[key]);
        }
        if (projection.type === 'sequence') {
          for (const key of ['actors', 'createdActors', 'destroyedActors']) model[key] = new Map(model[key]);
          for (const actor of model.actors.values()) for (const key of ['links', 'properties']) actor[key] = new Map(actor[key]);
          for (const box of model.boxes) for (const id of box.actorKeys) { const actor = model.actors.get(id); if (actor) actor.box = box; }
        }
        if (projection.type === 'er') for (const key of ['entities', 'classes']) model[key] = new Map(model[key]);
        if (projection.type === 'class') {
          const namespaces = values => new Map(values.map(([id, value]) => [id, {...value, classes: new Map(value.classes), notes: new Map(value.notes), children: namespaces(value.children)}]));
          for (const key of ['classes', 'notes', 'styleClasses', 'withCssClass']) model[key] = new Map(model[key]);
          model.namespaces = namespaces(model.namespaces);
        }
        if ((projection.type === 'flowchart' || projection.type === 'swimlane')) for (const key of ['nodes', 'classes', 'tooltips']) model[key] = new Map(model[key]);
        documents.set(model, {handle: document, root: model});
        finalizer?.register(model, document, model);
        return model;
      } catch (error) { document.free(); if (typeof error === 'string') return result(error); throw error; }
    },
    render(input, options = {}) {
      let callbackError, callbackFailed = false;
      try {
        if (typeof input === 'string') {
          const foreign = foreignParse(input, options);
          if (foreign) {
            try { return foreignRender(foreign, foreign.model, options); }
            finally { foreign.handle.free(); }
          }
        } else {
          const foreign = documents.get(input);
          if (foreign?.foreign) return foreignRender(foreign, input, options);
          if(foreign?.provider&&(foreign.policy||builtinDiagramType(foreign.provider)!==foreign.rawType)){
            if(!foreign.policy){const values=foreignValue(()=>foreign.handle.live_provider_view());foreign.config=values[1];foreign.policy=values[2];}
            return foreignRender(foreign,input,options);
          }
        }
        let measurer = options.measurer;
        const width = (text, size, style) => { try {
          if(measurer==null) {
            // Source/options/grammar errors have already been decided by Rust
            // before its first measurement. Inspect only now, never ahead of
            // parsing; retained models use their actual registration identity.
            const type=options.fontFamilyMono===undefined?undefined:typeof input==='string'
              ?nativeRegistry&&nativeValue(()=>nativeRegistry.detect_type(input))
              :documents.get(input)?.rawType;
            measurer=builtinMeasurer(options,type);
          }
          return measurer.width(text,size,style);
        } catch (e) { callbackFailed = true; callbackError = e; throw e; } };
        const icons = options.icons && (name => { try { return options.icons(name); } catch (e) { callbackFailed = true; callbackError = e; throw e; } });
        const clock = () => { try { return Date.now(); } catch (e) { callbackFailed = true; callbackError = e; throw e; } };
        const numeric = renderNumericOptions(typeof input === 'string' ? options : {...options, limit: undefined, maxEdges: undefined, config: undefined});
        const numericArguments = [numeric.json, numeric.mask, numeric.numbers, numeric.outputKind, numeric.output, width, icons, clock];
        let json, operation, bridge;
        if (typeof input === 'string') {
          if (nativeRegistry) {
            const limits = parseLimitValues(options);
            const render=typeof nativeRegistry.render_numbers_packet_with_resources==='function'?nativeRegistry.render_numbers_packet_with_resources:nativeRegistry.render_numbers_with_resources;
            json = render.call(nativeRegistry,input, numeric.json, numeric.mask, numeric.numbers, numeric.outputKind, numeric.output,
              limits.sourceKind, limits.sourceValue, limits.edgeKind, limits.edgeValue, width, icons, clock);
          } else {
            const render=typeof nativeCapabilities.render_numbers_packet_with_resources==='function'?nativeCapabilities.render_numbers_packet_with_resources:render_numbers_with_resources;
            json = render(input, ...numericArguments);
          }
        }
        else {
          const document = documents.get(input);
          if (!document) throw new TypeError('render() takes diagram text or a model that parse() returned.');
          if(document.hostStore){
            const value=hostOperation(document,'render',[],(primitive,capture,finish,resource,_values,takeFrame)=>(document.handle.render_host_frame_packet??document.handle.render_host_packet??document.handle.render_host).call(document.handle,primitive,capture,finish,numeric.json,numeric.mask,numeric.numbers,numeric.outputKind,numeric.output,resource(width),resource(icons),resource(clock),takeFrame()));
            if(callbackFailed)throw callbackError;return renderedValue(value);
          }
          if(document.capturedOwner){
            bridge=document.capturedOwner;
            operation=bridge.render(document,numericArguments);
          } else if (document.bridge) {
            bridge=document.bridge;
            operation=nativeModelCall(document,()=>{
              prepareMutableInput(document);
              const dispatcher=modelHostFunctions(bridge,operationError,()=>{const error=callbackFailed?{error:callbackError}:null;callbackFailed=false;callbackError=undefined;return error;});
              const job=typeof document.handle.prepare_detached_render_numbers==='function'
                ?nativeDetachedCall(()=>document.handle.prepare_detached_render_numbers(document.revision,numeric.json,numeric.mask,numeric.numbers,numeric.outputKind,numeric.output)):undefined;
              if(job)return runDetachedRender(document,job,false,width,icons,clock,undefined,dispatcher);
              acceptedBaselines.delete(document);
              const packet=document.handle.render_numbers_operation_packet_with_functions;
              const outcome=bridge.graph instanceof LayoutGraph
                ?layoutResult(()=>document.handle.render_numbers_operation_source_layout_with_functions(document.revision,...numericArguments,dispatcher),bridge.graph,document.revision,false,document.rawType)
                :typeof packet==='function'
                ?arenaRenderResult(()=>packet.call(document.handle,document.revision,...numericArguments,dispatcher),document.revision,false,document.rawType)
                :binaryResult(()=>document.handle.render_numbers_operation_binary_with_functions(document.revision,...numericArguments,dispatcher));
              publishMutableOutcome(document, outcome);return outcome.result;
            });
          } else json = document.handle.render_numbers_with_resources(...numericArguments);
        }
        // Publish native partial state before restoring the original host throw.
        if (callbackFailed) throw callbackError;
        if(operation) {
          if(Object.hasOwn(operation,'Err'))throw operationError(operation.Err,bridge);
          return renderedValue(operation.Ok);
        }
        return typeof input==='string'?sourceResult(json):result(json);
      } catch(error) { if (callbackFailed) throw renderError(callbackError); throw renderError(error); }
    },
    mount(element, source, options = {}) { return mountWith(renderer, element, source, options); },
    query(model, operation, argumentsValue = null) {
      const document = documents.get(model);
      if (!document) throw new TypeError('query() takes a model that parse() returned.');
      if (document.foreign) throw new TypeError('Custom provider models are ordinary host objects; call their methods directly.');
      if(document.hostStore)throw new TypeError('This mutable model does not expose read-only queries.');
      if (document.bridge) {
        if(document.bridge.registration.schema.id==='pele/agentflow@1'&&operation==='semanticModel'&&argumentsValue===null)return model.semanticModel();
        throw new TypeError('This mutable model does not expose read-only queries.');
      }
      return result((document.handle ?? document).query(operation, JSON.stringify(argumentsValue)));
    },
    dispose(model) {
      const document = documents.get(model)??constructedDocuments.get(model);
      if (document?.busy || document?.nativeBusy || document?.detachedDepth) throw new TypeError('Public model document is already in use');
      if (document) acceptedBaselines.delete(document);
      if(document?.bridge)releaseLiveRenderValues(document.bridge);
      if(document?.hostStore){document.hostStore.dispose();rawDisposed.add(model);documents.delete(model);document.disposed=true;finalizer?.unregister(model);document.handle.free();document.policy?.free();return;}
      if (document) { if(document.bridge)rawDisposed.add(model); documents.delete(model); constructedDocuments.delete(model); document.disposed = true; if(document.bridge)document.bridge.disposed=true; finalizer?.unregister(model); document.handle.free();document.policy?.free();if(document.capturedOwner)document.capturedOwner.release();else if(document.bridge)releaseDisposedBridge(document.bridge); }
    }
  };
  return renderer;
}
