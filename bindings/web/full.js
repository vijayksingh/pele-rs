import initialize from './initialize.js';
import {modelHostFunctions} from "./host-functions.js";
import {builtinDiagramType,builtinDiagramOrigin,ownRawModel,withBuiltinIcons,builtinInlineIcons} from './builtin-entries.js';
import {parseLimitOptions, parseLimitValues} from './parse-limits.js';
export {loadRegistry} from './registry.js';
import {mountWith} from './mount.js';
export {enableZoom} from './zoom.js';
import { parse_v2, render_numbers_with_resources, builtin_capabilities, NativeRegistry, foreign_decode_entities } from './pkg/pele.js';
import {renderNumericOptions} from './render-numbers.js';
import {encodeGraphBinary,decodeGraphBinary} from './graph-binary.js';
import {fromNativeError} from './errors.js';
export {PeleError, PeleError as PeleCompatibilityError} from './errors.js';
import { ModelBridge, ModelRegistry, stringValue } from './graph.js';
import { defaultMeasurer } from './resources.js';
export { canvasMeasurer, metricsMeasurer, forgetTextWidths } from './resources.js';
function result(json) {
  const envelope = JSON.parse(json);
  if (envelope.version !== 2) throw new Error('Unsupported compatibility contract');
  if (!envelope.ok) throw fromNativeError(envelope.error);
  return renderedValue(envelope.result);
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
  const liveProviders=new Map(canonicalProviders.map(provider=>[provider.type,{provider,native:true}]));
  function syncProviders(){for(const [type,entry] of liveProviders){const native=builtinDiagramType(entry.provider)===type;if(native!==entry.native){if(native)registryCall(()=>nativeValue(()=>{nativeRegistry.register_builtin(type);return '{"version":2,"ok":true,"result":null}';}));else registryCall(()=>foreignValue(()=>nativeRegistry.register_foreign(type,entry.provider)));entry.native=native;}}}
  const documents = new WeakMap(), rawDisposed = new WeakSet(), registry = new ModelRegistry();
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
  // bridge.graph tracks observations, including edits rejected by native apply.
  // Only a completely published native outcome can acknowledge that baseline.
  function prepareMutableInput(entry, values = []) {
    const confirmed = entry.confirmedGraph;
    entry.confirmedGraph = false;
    const baseline = entry.bridge.graph;
    const input = entry.bridge.snapshot(values, true), delta = input.delta;
    const unchanged = confirmed && entry.unchangedApplyIsNoop &&
      input.graph.root.kind === baseline.root.kind && input.graph.root.value === baseline.root.value &&
      delta.nodes.length === 0 && delta.strings.length === 0 &&
      input.graph.nodes.length === baseline.nodes.length && input.graph.strings.length === baseline.strings.length;
    if (!unchanged) entry.revision = result(entry.handle.apply_binary(encodeGraphBinary(entry.revision, input.graph)));
    return input;
  }
  function publishMutableOutcome(entry, outcome) {
    entry.revision = outcome.revision;
    entry.bridge.reconcile(outcome.graph);
    entry.confirmedGraph = true;
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
  function adoptMutable(mutable, preparedType, existingBridge) {
          const snapshot = binaryResult(()=>mutable.snapshot_binary()), schema = result(mutable.schema());
          const rawType=mutable.entry_type(), live=preparedType===undefined?liveProviders.get(rawType):undefined;
          const entry = {handle: mutable, revision: snapshot.revision, disposed: false, preparedType,rawType,provider:live?.provider, confirmedGraph: false, unchangedApplyIsNoop: mutable.unchanged_public_graph_is_noop?.() === true};
          const invoke = (method, receiver, argumentsValue) => nativeModelCall(entry, () => {
            if (entry.disposed) throw new TypeError('Public model document is disposed');
            const input = prepareMutableInput(entry, [receiver, ...argumentsValue]);
            const outcome = binaryResult(()=>mutable.invoke_binary_with_functions(entry.revision, JSON.stringify({method, receiver: input.values[0], arguments: input.values.slice(1)}),modelHostFunctions(bridge,operationError)));
            publishMutableOutcome(entry, outcome);
            if (Object.hasOwn(outcome.result, 'Err')) throw operationError(outcome.result.Err, bridge);
            return bridge.value(outcome.result.Ok);
          });
          const bridge = existingBridge ?? new ModelBridge(snapshot.graph,schema,registry,invoke);
          if(existingBridge){bridge.invoke=invoke;bridge.reconcile(snapshot.graph);}
          entry.bridge = bridge;
          entry.confirmedGraph = true;
          const model = bridge.root;
          documents.set(model, entry);
          ownRawModel(model,rawType,(config,options)=>renderer.renderRaw(rawType,model,config,options));
          finalizer?.register(model, mutable, model);
          return model;
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
    // Internal low-level entry adapters. Models stay in their native document;
    // neither prepared sources nor raw results cross core preprocessing.
    parsePrepared(type, source, config, title, lineOffset, maxEdges) {
      const limits=parseLimitValues({maxEdges});
      const handle=registryCall(()=>{try{return nativeRegistry.parse_prepared(type,source,finiteConfigJSON(config),title,lineOffset,limits.edgeKind,limits.edgeValue);}catch(error){if(typeof error==='string')return result(error);throw error;}});
      try{return adoptMutable(handle,type);}catch(error){handle.free();throw error;}
    },
    renderRaw(type, model, config, options = {}) {
      if(rawDisposed.has(model))throw new TypeError('Public model document is disposed');
      let entry=documents.get(model);
      if(!entry){
        const schema=registryCall(()=>nativeValue(()=>nativeRegistry.entry_schema(type)));
        const bridge=new ModelBridge({version:1,root:{kind:'undefined'},strings:[],nodes:[]},schema,registry,()=>{throw new TypeError('Public model construction is incomplete');});
        bridge.root=model;
        const graph=bridge.snapshot().graph;
        const handle=registryCall(()=>{try{return nativeRegistry.create_prepared_model_binary(type,encodeGraphBinary('0',graph),finiteConfigJSON(config));}catch(error){if(typeof error==='string')return result(error);throw error;}});
        try{adoptMutable(handle,type,bridge);}catch(error){handle.free();throw error;}
        entry=documents.get(model);
      }
      if(!entry?.bridge||entry.rawType!==type)throw new TypeError('Raw diagram render requires a model of this diagram');
      if(entry.disposed)throw new TypeError('Public model document is disposed');
      return nativeModelCall(entry,()=>{
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
        prepareMutableInput(entry);
        const outcome=binaryResult(()=>entry.handle.render_raw_operation_binary_with_functions(entry.revision,finiteConfigJSON(config),numeric.json,numeric.mask,numeric.numbers,width,icons,clock,inline,modelHostFunctions(entry.bridge,operationError,()=>{const error=callbackFailed?{error:callbackError}:null;callbackFailed=false;callbackError=undefined;return error;})));
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
          document = nativeRegistry.parse(source, limits.json, limits.sourceKind, limits.sourceValue, limits.edgeKind, limits.edgeValue);
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
            json = nativeRegistry.render_numbers_with_resources(input, numeric.json, numeric.mask, numeric.numbers, numeric.outputKind, numeric.output,
              limits.sourceKind, limits.sourceValue, limits.edgeKind, limits.edgeValue, width, icons, clock);
          } else json = render_numbers_with_resources(input, ...numericArguments);
        }
        else {
          const document = documents.get(input);
          if (!document) throw new TypeError('render() takes diagram text or a model that parse() returned.');
          if (document.bridge) {
            bridge=document.bridge;
            operation=nativeModelCall(document,()=>{
              prepareMutableInput(document);
              const outcome=binaryResult(()=>document.handle.render_numbers_operation_binary_with_functions(document.revision,...numericArguments,modelHostFunctions(bridge,operationError,()=>{const error=callbackFailed?{error:callbackError}:null;callbackFailed=false;callbackError=undefined;return error;})));
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
        return result(json);
      } catch(error) { if (callbackFailed) throw renderError(callbackError); throw renderError(error); }
    },
    mount(element, source, options = {}) { return mountWith(renderer, element, source, options); },
    query(model, operation, argumentsValue = null) {
      const document = documents.get(model);
      if (!document) throw new TypeError('query() takes a model that parse() returned.');
      if (document.foreign) throw new TypeError('Custom provider models are ordinary host objects; call their methods directly.');
      if (document.bridge) {
        if(document.bridge.registration.schema.id==='pele/agentflow@1'&&operation==='semanticModel'&&argumentsValue===null)return model.semanticModel();
        throw new TypeError('This mutable model does not expose read-only queries.');
      }
      return result((document.handle ?? document).query(operation, JSON.stringify(argumentsValue)));
    },
    dispose(model) {
      const document = documents.get(model);
      if (document?.busy || document?.nativeBusy) throw new TypeError('Public model document is already in use');
      if (document) { if(document.bridge)rawDisposed.add(model); documents.delete(model); document.disposed = true; if(document.bridge)document.bridge.disposed=true; finalizer?.unregister(model); document.handle.free();document.policy?.free(); }
    }
  };
  return renderer;
}
