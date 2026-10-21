import {test, expect} from '@playwright/test';
import {readFile, writeFile} from 'node:fs/promises';
const oraclePin = 'd1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
const driver = await readFile(new URL('./support/gitgraph-tag-iterables.js', import.meta.url), 'utf8');

test('GitGraph tag iterators retain Set and String receivers through mutation, nested renders and original errors', async ({page}, info) => {
  test.setTimeout(120000);
  await page.goto('/examples/web/');
  const report = await page.evaluate(async ({source, pin}) => {
    const hash = async path => {
      const response = await fetch(path);
      if (!response.ok) throw Error('Missing ' + path);
      return Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256', await response.arrayBuffer())), n => n.toString(16).padStart(2, '0')).join('');
    };
    const paths = ['/dist/evidence/pele-oracle.js', '/dist/package-web/package/full.js', '/dist/package-web/package/pkg/pele_bg.wasm'];
    const before = Object.fromEntries(await Promise.all(paths.map(async p => [p, await hash(p)])));
    if (before[paths[0]] !== pin) throw Error('Pinned Pele changed');
    const url = URL.createObjectURL(new Blob([source], {type: 'text/javascript'}));
    try {
      const {cases, runGitGraph} = await import(url);
      const {loadRenderer} = await import('/dist/package-web/package/full.js');
      const {loadDiagramEntries, disposeDiagramModel} = await import('/dist/package-web/package/diagram-entries.js');
      await loadDiagramEntries();
      const oracle = await import('/dist/evidence/pele-esm/core.js');
      const original = (await import('/dist/evidence/pele-esm/diagrams/gitGraph.js')).default;
      oracle.register(original);
      const entry = (await import('/dist/package-web/package/diagrams/gitGraph.js')).default;
      const fixture = await (await fetch('/fixtures/public-gitgraph-tag-iterables.json')).json();
      const engine = await loadRenderer(), rows = [];
      try {
        for (const {raw, responsive, mode} of cases()) {
          const run = (core, entry, dispose) => {
            try { return runGitGraph(core, entry, dispose, raw, responsive, mode); }
            catch (error) { return {fatal: {name: error?.name, message: error?.message}}; }
          };
          rows.push({raw, responsive, mode,
            oracle: run(oracle, original, () => {}),
            candidate: run(engine, entry, model => raw ? disposeDiagramModel(model) : engine.dispose(model)),
            fixture: fixture.rows.find(row => row.raw === raw && row.responsive === responsive && row.mode === mode),
          });
        }
      } finally { engine.close(); }
      const after = Object.fromEntries(await Promise.all(paths.map(async p => [p, await hash(p)])));
      return {before, after, rows};
    } finally { URL.revokeObjectURL(url); }
  }, {source: driver, pin: oraclePin});
  const path = info.outputPath('gitgraph-tag-iterables.json');
  await writeFile(path, JSON.stringify(report, null, 2));
  await info.attach('gitgraph-tag-iterables.json', {path, contentType: 'application/json'});
  expect(report.after).toEqual(report.before);
  expect(report.rows).toHaveLength(52);
  for (const row of report.rows) {
    const id = [row.raw, row.responsive, row.mode].join('/');
    expect(row.oracle.fatal, id + '/reachable').toBeUndefined();
    expect(row.oracle, id + '/pinned').toEqual(row.fixture);
    expect(row.oracle.fired, id + '/callback').toBe(true);
    expect(row.candidate, id).toEqual(row.oracle);
  }
});
