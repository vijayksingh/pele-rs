// Pure offline extraction of the frozen profiler's classification and weighted summary.
import assert from 'node:assert/strict';
const groupRules=[
 ['native-admission',/(captured_admission|captured_transport|admit_observed|admit_outbound|canonical_frame_bytes|validate_schema|validate_structure|preflight_encoding)/],
 ['native-flat-storage',/public::flat_store::/],
 ['native-flat-observation',/flat_store::observation::/],
 ['native-flat-write',/flat_store::write::|NativeSourceWriter/],
 ['native-flat-page-cow',/flat_store::pages::/],
 ['native-lease-directories',/native_canonical::(Directory|NativeCanonicalEpoch|Lease)/],
 ['native-owner-wrapper',/pele_wasm::native_canonical::/],
 ['sankey-import-projection',/sankey::public_model::(imported|imported_from|normalized_source|stored_field|root_field|optional_text|array|string|index)(?:\b|::)/],
 ['sankey-raw-parser',/sankey::parser|sankey::Sankey as .*DiagramModule>::parse/],
 ['sankey-render-layout',/sankey::(render|layout|measure)/],
 ['initial-native-construction',/sankey::public_model::(initial_into|initial_graph)|NativeSourceWriter/],
 ['native-svg-and-json',/(render_numbers|serde_json|compat::svg|svg_document|escape_xml)/],
 ['js-native-storage-view-or-emitter',/\/native-storage\.js/],
 ['native-pod-refresh',/#refresh .*\/native-storage\.js/],
 ['native-pod-window-copy',/#windowWord .*\/native-storage\.js/],
 ['native-pod-string-decode',/(?:#copyUnits|stringValueAt) .*\/native-storage\.js/],
 ['js-observation-emitter',/(?:#record|#flush|#strings|#atom|beginArguments|argument|beginNode|beginItems|mapEntry|setItem|finishNode|finish|abort|property|root) .*\/native-storage\.js/],
 ['shared-publication',/\breconcileGraph\b/],
 ['shared-observation-and-match',/\b(snapshotGraph|matchesGraph)\b/],
 ['owner-transport-gates',/\b(capturedObservedFrame|outcomePacket|publication_started|exactEquals)\b|\/captured-owner\.js/],
 ['legacy-binary',/\b(encodeGraphBinary|decodeGraphBinary)\b|encode_binary_exchange|snapshot_binary/],
 ['allocator',/\b(malloc|calloc|realloc|free|__rust_alloc|__rust_dealloc)\b|dlmalloc/],
];
// Ordered leaf-only roles form a disjoint partition. Inclusive groups below do
// not: they are each de-duplicated per sample and must never be added together.
const roleRules=[
 ['admission',/(captured_admission|captured_transport|admit_observed|admit_outbound|canonical_frame_bytes|validate_schema|validate_structure|preflight_encoding)/],
 ['allocator',/\b(malloc|calloc|realloc|free|__rust_alloc|__rust_dealloc)\b|dlmalloc/],
 ['flat-native-storage',/public::flat_store::/],
 ['native-wrapper-or-leases',/pele_wasm::native_canonical::/],
 ['native-parse',/sankey::parser|sankey::Sankey as .*DiagramModule>::parse/],
 ['native-projection',/sankey::public_model::/],
 ['native-render-layout',/sankey::(render|layout|measure)/],
 ['native-svg-json',/(render_numbers|serde_json|compat::svg|svg_document|escape_xml)/],
 ['js-storage-adapter',/\/native-storage\.js/],
 ['js-observer-publisher',/\/graph-storage\.js/],
 ['js-owner-transport',/\/(captured-owner|full)\.js/],
 ['legacy-graph-boundary',/\/(graph|graph-binary)\.js|encode_binary_exchange|snapshot_binary/],
];
const unionRules=[
 ['native-admission-or-flat-or-wrapper',['native-admission','native-flat-storage','native-owner-wrapper']],
 ['all-canonical-storage-boundary',['native-admission','native-flat-storage','native-owner-wrapper','js-native-storage-view-or-emitter','shared-publication','shared-observation-and-match','owner-transport-gates']],
 ['sankey-parse-projection-layout',['sankey-raw-parser','sankey-import-projection','sankey-render-layout']],
 ['shared-observation-publication',['shared-publication','shared-observation-and-match']],
 ['native-pod-read-windows',['native-pod-refresh','native-pod-window-copy','native-pod-string-decode']],
];
const normalizedFrameText=frame=>frame.functionName.replace(/\[[0-9a-f]+\]/g,'')+' '+frame.url;
const languageFor=frame=>frame.functionName==='(garbage collector)'?'gc':(frame.url.startsWith('wasm://')||/\.wasm(?:$|[?#])/.test(frame.url))?'wasm':/^https?:/.test(frame.url)?'javascript':'engine-or-profiler';
function summarize(profile){
 const nodes=new Map(profile.nodes.map(node=>[node.id,node])),parents=new Map();
 for(const node of profile.nodes)for(const child of node.children??[]){assert(!parents.has(child),'CPU profile is not a tree');parents.set(child,node.id);}
 assert(profile.samples?.length&&profile.timeDeltas?.length===profile.samples.length,'Weighted CPU samples unavailable');
 const self=new Map(),inclusive=new Map(),frames=new Map(),categories=new Map(),roles=new Map(),unions=new Map(unionRules.map(([name])=>[name,0])),groups=new Map(groupRules.map(([name])=>[name,{selfUs:0,inclusiveUs:0}]));let totalUs=0;
 const keyFor=frame=>JSON.stringify([frame.functionName,frame.url,frame.lineNumber,frame.columnNumber,frame.scriptId]);
 for(let i=0;i<profile.samples.length;i++){
  const weight=profile.timeDeltas[i];assert(Number.isFinite(weight)&&weight>=0);totalUs+=weight;
  const leaf=nodes.get(profile.samples[i]);assert(leaf);const frame=leaf.callFrame,key=keyFor(frame);frames.set(key,frame);self.set(key,(self.get(key)??0)+weight);
  const category=languageFor(frame);categories.set(category,(categories.get(category)??0)+weight);
  const role=category==='gc'?'gc':(roleRules.find(([,pattern])=>pattern.test(normalizedFrameText(frame)))?.[0]??category+'-other');
  roles.set(role,(roles.get(role)??0)+weight);
  const seenFrames=new Set(),seenGroups=new Set(),seenNodes=new Set();let node=leaf;
  while(node){
   assert(!seenNodes.has(node.id),'CPU profile parent cycle');seenNodes.add(node.id);
   const f=node.callFrame,k=keyFor(f);frames.set(k,f);if(!seenFrames.has(k)){inclusive.set(k,(inclusive.get(k)??0)+weight);seenFrames.add(k);}
   const text=normalizedFrameText(f);
   for(const[name,pattern]of groupRules)if(pattern.test(text)){if(node===leaf)groups.get(name).selfUs+=weight;seenGroups.add(name);}
   const parent=parents.get(node.id);node=parent===undefined?undefined:nodes.get(parent);
  }
  for(const name of seenGroups)groups.get(name).inclusiveUs+=weight;
  for(const[name,members]of unionRules)if(members.some(member=>seenGroups.has(member)))unions.set(name,unions.get(name)+weight);
 }
 assert(totalUs>0);
 assert.equal([...categories.values()].reduce((a,b)=>a+b,0),totalUs);assert.equal([...roles.values()].reduce((a,b)=>a+b,0),totalUs);
 const rank=table=>[...table].sort((a,b)=>b[1]-a[1]).slice(0,60).map(([key,us])=>({frame:frames.get(key),weightedUs:us,percent:100*us/totalUs}));
 return {sampleCount:profile.samples.length,weightedUs:totalUs,profileSpanUs:profile.endTime-profile.startTime,selfCategories:Object.fromEntries([...categories].map(([name,us])=>[name,{weightedUs:us,percent:100*us/totalUs}])),disjointSelfRoles:Object.fromEntries([...roles].map(([name,us])=>[name,{weightedUs:us,percent:100*us/totalUs}])),deduplicatedAncestorUnions:Object.fromEntries([...unions].map(([name,us])=>[name,{weightedUs:us,percent:100*us/totalUs,members:unionRules.find(([key])=>key===name)[1]}])),topSelf:rank(self),topInclusive:rank(inclusive),architectureGroups:Object.fromEntries([...groups].map(([name,g])=>[name,{...g,selfPercent:100*g.selfUs/totalUs,inclusivePercent:100*g.inclusiveUs/totalUs}])),groupPatterns:Object.fromEntries(groupRules.map(([name,pattern])=>[name,pattern.source])),phaseCallCounts:null,phaseCallCountsReason:'CPU samples are not invocation counts. No production instrumentation or monkey-patching is installed.'};
}

export {summarize};
