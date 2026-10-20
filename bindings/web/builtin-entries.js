// Identity markers for actual linked built-ins, never structural provider guesses.
const originals=new WeakMap();
export function markBuiltinDiagram(diagram){
  originals.set(diagram,{type:diagram.type,parse:diagram.parse,render:diagram.render,section:diagram.section,keepComments:diagram.keepComments,sectionOwn:Object.hasOwn(diagram,'section'),commentsOwn:Object.hasOwn(diagram,'keepComments')});
}
export function builtinDiagramOrigin(diagram){return originals.get(diagram)?.type;}
export function builtinDiagramType(diagram){
  const original=originals.get(diagram);
  if(!original)return undefined;
  for(const key of ['type','parse','render','section','keepComments'])if(diagram[key]!==original[key])return undefined;
  if(Object.hasOwn(diagram,'section')!==original.sectionOwn||Object.hasOwn(diagram,'keepComments')!==original.commentsOwn)return undefined;
  return original.type;
}

// Pele src/text/icons.ts at458322a5 (MIT,kepano): scoped host icon lookup.
// Ownership/context only; no diagram-model algorithms.
const rawOwners = new WeakMap();
export function ownRawModel(model, type, render, dispose) { rawOwners.set(model, {type, render, dispose}); }
export function disposeRawModel(model) { const owner=rawOwners.get(model);if(!owner?.dispose)return false;owner.dispose();return true; }
export function rawModelOwner(model, type) { const owner=rawOwners.get(model); return owner?.type===type?owner:undefined; }
let inlineIcons;
export function withBuiltinIcons(icons, run) {
  const outer=inlineIcons, found=new Map();
  inlineIcons=icons&&(name=>{if(!found.has(name))found.set(name,icons(name)??'');return found.get(name);});
  try{return run();}finally{inlineIcons=outer;}
}
export function builtinInlineIcons(){return inlineIcons;}
