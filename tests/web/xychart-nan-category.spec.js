import {test, expect} from '@playwright/test';
import {readFile, writeFile} from 'node:fs/promises';
const oraclePin = 'd1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
const source = await readFile(new URL('./support/xychart-nan-category.js', import.meta.url), 'utf8');
const packageRoot = process.env.PELE_TEST_PACKAGE_ROOT ?? '/dist/package-web/package';

test('XYChart NaN category stride terminates with exact normal and raw oracle output and recovery', async ({page}, info) => {
  test.setTimeout(60000);
  await page.goto('/examples/web/');
  const report = await page.evaluate(async ({source, packageRoot, oraclePin}) => {
    const hash = async path => {
      const response = await fetch(path);
      if (!response.ok) throw Error('Missing ' + path);
      return Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256', await response.arrayBuffer())), n => n.toString(16).padStart(2, '0')).join('');
    };
    const paths = ['/dist/evidence/pele-oracle.js', '/dist/evidence/pele-esm/chunks/YO6PB664.js',
      packageRoot + '/full.js', packageRoot + '/pkg/pele_bg.wasm', packageRoot + '/diagrams/xychart.js',
      '/fixtures/public-xychart-nan-category.json'];
    const before = Object.fromEntries(await Promise.all(paths.map(async path => [path, await hash(path)])));
    if (before[paths[0]] !== oraclePin) throw Error('Pinned Pele changed');
    const fixture = await (await fetch(paths[5])).json();
    const url = URL.createObjectURL(new Blob([source], {type: 'text/javascript'}));
    try {
      const {SOURCE, runNaNCategory} = await import(url);
      const {loadRenderer} = await import(location.origin + packageRoot + '/full.js');
      const {loadDiagramEntries, disposeDiagramModel} = await import(location.origin + packageRoot + '/diagram-entries.js');
      await loadDiagramEntries();
      const candidate = (await import(location.origin + packageRoot + '/diagrams/xychart.js')).default;
      const oracle = await import('/dist/evidence/pele-esm/core.js');
      const original = (await import('/dist/evidence/pele-esm/diagrams/xychart.js')).default;
      oracle.register(original);
      const renderer = await loadRenderer(), rows = [];
      try {
        for (const raw of [false, true]) {
          const run = (core, entry, dispose) => {
            try { return runNaNCategory(core, entry, dispose, raw); }
            catch (error) { return {fatal: {name: error?.name, message: error?.message}}; }
          };
          rows.push({raw,
            oracle: run(oracle, original, () => {}),
            candidate: run(renderer, candidate, model => raw ? disposeDiagramModel(model) : renderer.dispose(model)),
          });
        }
      } finally { renderer.close(); }
      const after = Object.fromEntries(await Promise.all(paths.map(async path => [path, await hash(path)])));
      return {source: SOURCE, before, after, fixture, rows};
    } finally { URL.revokeObjectURL(url); }
  }, {source, packageRoot, oraclePin});
  const path = info.outputPath('xychart-nan-category.json');
  await writeFile(path, JSON.stringify(report, null, 2));
  await info.attach('xychart-nan-category.json', {path, contentType: 'application/json'});
  expect(report.after).toEqual(report.before);
  expect(report.fixture.oracleSHA256).toBe(oraclePin);
  expect(report.before['/dist/evidence/pele-esm/chunks/YO6PB664.js']).toBe(report.fixture.chunkSHA256);
  expect(report.source).toBe(report.fixture.source);
  expect(report.rows).toHaveLength(2);
  for (const row of report.rows) {
    const label = row.raw ? 'raw' : 'normal';
    expect(row.oracle.fatal, label + '/reachable').toBeUndefined();
    expect(row.oracle.first.error, label + '/first').toBeUndefined();
    expect(row.oracle.recovery.error, label + '/recovery').toBeUndefined();
    const {raw, ...pinned} = report.fixture.rows.find(item => item.raw === row.raw);
    expect(row.oracle.first, label + '/pinned').toEqual(pinned);
    expect(row.oracle.first.trace, label + '/callbacks').toHaveLength(raw ? 22 : 19);
    expect(row.oracle.recovery.trace.length, label + '/recovery callbacks').toBeGreaterThan(0);
    expect(row.oracle.identity, label + '/identity').toEqual({axis: true, categories: true, plots: true});
    expect(row.candidate, label + '/same-engine').toEqual(row.oracle);
  }
});
