from pathlib import Path
import hashlib,json,shutil
base=Path('dist/perf-patches/parse-architecture')
out=base/'source-first-edit-profiler'; dest=out/'files/scripts/perf'
old=base/'source-fresh-initializer-profiler/files/scripts/perf/profile-source-canonical.mjs'
runner=base/'source-sparse-ingress-runner/files/scripts/perf'
harness=(runner/'compare-first-use-web-vm.mjs').read_text()
sha=lambda b:hashlib.sha256(b).hexdigest()
assert sha(harness.encode())=='5469078edefad399f109ad29f4c3f25a56ce3834bfdbe4a635c4f3b7ea463bb0'
start=harness.index(' const taskSignature=value=>{')
end=harness.index(' const baseline=verify();',start)
extracted=harness[start:end]
work_start=harness.index(' const work=Object.fromEntries(names.map(name=>')
work_end=harness.index('\n',work_start)
work_bytes=harness[work_start:work_end]
setup=''' await page.evaluate(async({now,workload})=>{
  Date.now=()=>now;const js=await import('/oracle.js'),controlAPI=await import('/control/index.js'),candidateAPI=await import('/candidate/index.js');
  const control=await controlAPI.loadRenderer(),candidate=await candidateAPI.loadRenderer();
  const runtimes={js,control,candidate},probeBackend=undefined,backendChecks=[],profile='fixed',applicable=['parse-first-edited-render'];
  const measurer=profile==='fixed'?{width:(text,size,style)=>text.length*size*((style&1)?.71:.6)}:controlAPI.canvasMeasurer('sans-serif','monospace');if(!measurer)throw Error('Canvas unavailable');
  const options={...workload.options,measurer};
'''+extracted+''' const names=['js','control','candidate'],stage='parse-first-edited-render';let checksum=0;
'''+work_bytes+'''
  window.profileWork={
   run(name,stage,ms){const initialChecksum=checksum,start=performance.now();let operations=0;do{work[name]();operations++;}while(performance.now()-start<ms);return{operations,checksum:checksum-initialChecksum,elapsedMs:performance.now()-start};},
   verify,
   close(){for(const name of ['control','candidate'])runtimes[name].close();}
  };
 },{now,workload});
'''
s=old.read_text()
s=s.replace("const duration=2000,warmup=500,interval=250,stages=['parse','render-model','render-source'];","const duration=2000,warmup=500,interval=250,stages=['parse-first-edited-render'];")
s=s.replace('profiles:9','profiles:3')
s=s.replace("['native-parser',/sankey::parser|sankey::indexed::/],","['native-parser',/sankey::parser::|sankey::indexed::(?:parse|parse_bounded|parse_with|parse_source)(?:\\b|::)/],")
s=s.replace(" ['native-projection',", " ['native-source-ingress',/source_input|prepare_overlay|admit_observed|apply_source_input|validate_read_observed|DecoderBudget/],\n ['native-projection',",1)
s=s.replace(" ['native-projection',", " ['native-overlay-preflight',/SourceCanonical::prepare_overlay/],\n ['native-observed-admission',/captured_transport::admit_observed/],\n ['native-size-counter',/captured_transport::Counter::/],\n ['native-decoder-budget',/captured_transport::DecoderBudget::/],\n ['native-captured-logical',/source_input::.*CapturedLogical|source_input::.*::(?:own_fields|string_is)/],\n ['native-import-string',/sankey::public_model::.*string_owned|sankey::public_model::.*imported_from/],\n ['native-projection',",1)
s=s.replace(" ['construction',groupRules", " ['source-ingress',groupRules.find(([n])=>n==='native-source-ingress')[1]],\n ['construction',groupRules",1)
a=s.index(' await page.evaluate(async({now,workload})=>{');b=s.index(' const cdp=',a)
s=s[:a]+setup+s[b:]
s=s.replace("assert.equal(final.svg,initial.svg);assert(Object.is(final.width,initial.width)&&Object.is(final.height,initial.height));","assert.deepEqual(final,initial,'First-edited whole output changed after profiling');")
s=s.replace("const before=await inputs();", "const originalHarness=await readFile(new URL('./compare-first-use-web-vm.mjs',import.meta.url),'utf8');\nassert.equal(sha(originalHarness),expected.originalHarnessSHA256);\nconst extracted=originalHarness.slice(expected.extraction.startByte,expected.extraction.endByte);assert.equal(sha(extracted),expected.extraction.sha256);assert((await readFile(harness,'utf8')).includes(extracted),'Scenario extraction drift');\nconst workExtracted=originalHarness.slice(expected.workExtraction.startByte,expected.workExtraction.endByte);assert.equal(sha(workExtracted),expected.workExtraction.sha256);assert((await readFile(harness,'utf8')).includes(workExtracted),'Operation closure drift');\nconst before=await inputs();")
s=s.replace("expected:sha(await readFile(expectedFile))", "expected:sha(await readFile(expectedFile)),originalHarness:sha(await readFile(new URL('./compare-first-use-web-vm.mjs',import.meta.url)))")
s=s.replace("sourcePins:expected.sourcePins,inputsBefore", "sourcePins:expected.sourcePins,scenarioExtraction:expected.extraction,originalHarnessSHA256:expected.originalHarnessSHA256,inputsBefore")
s=s.replace("'dist/perf-patches/parse-architecture/source-canonical-runtime/candidate-package'", "'dist/perf-patches/parse-architecture/source-sparse-ingress-runtime/candidate-package'")
s=s.replace("'dist/evidence/source-canonical-profiles'", "'dist/evidence/source-first-edit-profiles'")
(dest/'profile-source-first-edit.mjs').write_text(s)
for name in ['cases.mjs','compare-first-use-web-vm.mjs','probe-layout-continuity.mjs','probe-source-layout.mjs','probe-source-ingress.mjs','source-layout-pins.json']:
 shutil.copy2(runner/name,dest/name)
prior=json.loads((base/'source-fresh-initializer-profiler/files/scripts/perf/source-canonical-inputs.json').read_text())
package=base/'source-sparse-ingress-runtime/package.json'; manifest=json.loads(package.read_text())
prior['packages']['candidate']=manifest['package']
prior['sourcePins']['candidate']={'commit':manifest['commit'],'freezeManifestSHA256':sha(package.read_bytes()),'packageFiles':len(manifest['package'])}
prior['originalHarnessSHA256']=sha(harness.encode())
prior['workExtraction']={'startByte':work_start,'endByte':work_end,'sha256':sha(work_bytes.encode()),'unchangedBytes':len(work_bytes.encode())}
prior['extraction']={'startByte':len(harness[:start].encode()),'endByte':len(harness[:end].encode()),'sha256':sha(extracted.encode()),'includes':['taskSignature','scenario','verify'],'unchangedBytes':len(extracted.encode()),'source':'compare-first-use-web-vm.mjs'}
# Source is ASCII up to extraction, so JS slice byte offsets coincide.
assert start==prior['extraction']['startByte'] and end==prior['extraction']['endByte']
(dest/'source-canonical-inputs.json').write_text(json.dumps(prior,indent=2)+'\n')
# Preserve all existing package checks and serial probes; replace timing dispatch only.
r=(runner/'run-source-layout.mjs').read_text()
r=r.replace("const phase=arg('--phase');assert(['warm','first-use'].includes(phase));","const phase='first-edit-profile';")
a=r.index("const stages=phase===");b=r.index("assert(!args.some",a)
r=r[:a]+"const harnessName='profile-source-first-edit.mjs',reportPath=resolve(output,'cpu-profiles/summary.json');\nconst args=[resolve(here,harnessName),'--execute-approved','--control',roots.control,'--candidate',roots.candidate,'--output',resolve(output,'cpu-profiles')];\n"+r[b:]
a=r.index("const report=JSON.parse(await readFile(reportPath,'utf8'));");b=r.index("for(const[name,root]of Object.entries(roots))assert.deepEqual(await hashes(root),before[name]);",a)
r=r[:a]+"const report=JSON.parse(await readFile(reportPath,'utf8'));assert(report.succeeded&&report.profiles.length===3&&report.inputsUnchanged&&report.browserClosed&&report.modelsDisposed);assert.equal(report.inputsBefore.harness,pins.files[harnessName]);assert.equal(report.inputsBefore.cases,pins.files['cases.mjs']);assert.deepEqual(report.inputsBefore.packages,before);assert.equal(report.settings.durationMs,2000);assert.equal(report.settings.warmupMs,500);assert.equal(report.settings.samplingIntervalUs,250);\n"+r[b:]
r=r.replace('record.verifiedRows=24','record.verifiedProfiles=3')
(dest/'run-source-first-edit.mjs').write_text(r)
pins=json.loads((dest/'source-layout-pins.json').read_text())
pins['files']={p.name:sha(p.read_bytes()) for p in dest.iterdir() if p.name not in ['source-layout-pins.json','run-source-first-edit.mjs']}
(dest/'source-layout-pins.json').write_text(json.dumps(pins,indent=2)+'\n')
(out/'manifest.json').write_text(json.dumps({'status':'prepared-not-executed-awaiting-parent-review-GO','originalHarness':prior['originalHarnessSHA256'],'extraction':prior['extraction'],'workExtraction':prior['workExtraction'],'source':prior['sourcePins'],'files':{str(p.relative_to(out)):sha(p.read_bytes()) for p in dest.iterdir()},'profile':{'windows':3,'durationMs':2000,'warmupMs':500,'intervalUs':250,'case':'sankey/medium','profile':'fixed','stage':'parse-first-edited-render'},'probes':[8,16,12]},indent=2)+'\n')
print(json.dumps(prior['extraction'],indent=2))
