import initialize from './initialize.js';
// Low-level built-in host facade. Prepared parsing, models and drawing stay Rust.
import {markBuiltinDiagram,rawModelOwner,disposeRawModel} from './builtin-entries.js';
import {NativeRegistry} from './pkg/pele.js';
import {createRenderer} from './full.js';
let pending, renderer;
/** Explicit asynchronous WASM readiness; entry objects remain synchronous after it. */
export function loadDiagramEntries(moduleOrPath) {
  if (!pending) pending=initialize(moduleOrPath===undefined?undefined:{module_or_path:moduleOrPath}).then(()=>{renderer=createRenderer(new NativeRegistry(true));}).catch(error=>{pending=undefined;throw error;});
  return pending;
}
function ready(){if(!renderer)throw new Error('Call and await loadDiagramEntries() before parsing or rendering a diagram entry');return renderer;}
/** Free an owned low-level model; normal collection also has a native finalizer. */
export function disposeDiagramModel(model){if(!disposeRawModel(model))ready().dispose(model);}
export function builtinEntry(metadata){
  const diagram={type:metadata.type,...(Object.hasOwn(metadata,'section')?{section:metadata.section}:{}),...(Object.hasOwn(metadata,'keepComments')?{keepComments:metadata.keepComments}:{}),
    parse(source,config,title,lineOffset,limits){
      const maxEdges=['flowchart','agentflow','swimlane'].includes(metadata.type)?limits.maxEdges:limits?.maxEdges;
      return ready().parsePrepared(metadata.type,source,config,title,lineOffset,maxEdges);
    },
    render(model,config,options){const owner=rawModelOwner(model,metadata.type);return owner?owner.render(config,options):ready().renderRaw(metadata.type,model,config,options);}
  };
  Object.defineProperty(diagram.parse,'length',{value:metadata.parseArity,configurable:true});
  Object.defineProperty(diagram.render,'length',{value:metadata.renderArity,configurable:true});
  markBuiltinDiagram(diagram);
  return diagram;
}
