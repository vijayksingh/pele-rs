import {test, expect} from '@playwright/test';
import {readFile, writeFile} from 'node:fs/promises';
import {cases, groups, ORACLE_SHA256} from './support/sankey-callback-scenarios.js';

// Serve only this test's shared scenario module. Candidate/oracle assets stay
// unmodified; no capability mocks, result rewriting, or implementation hooks.
const scenarioURL = '/dist/evidence/sankey-resource-callback-scenarios.js';
for (const group of groups) {
  test(`Sankey resource callbacks match pinned Pele: ${group}`, async ({page}, info) => {
    test.setTimeout(120000);
    const body = await readFile(new URL('./support/sankey-callback-scenarios.js', import.meta.url), 'utf8');
    await page.route(`**${scenarioURL}`, route => route.fulfill({status: 200, contentType: 'text/javascript', body}));
    await page.goto('/examples/web/');
    const report = await page.evaluate(async ({group, scenarioURL, expectedOracle}) => {
      const sha = async path => {
        const response = await fetch(path); if (!response.ok) throw Error(`Missing evidence asset: ${path}`);
        const hash = await crypto.subtle.digest('SHA-256', await response.arrayBuffer());
        return [...new Uint8Array(hash)].map(n => n.toString(16).padStart(2, '0')).join('');
      };
      const paths = ['/dist/evidence/pele-oracle.js', '/dist/package-web/package/full.js', '/dist/package-web/package/pkg/pele_bg.wasm', scenarioURL];
      const before = Object.fromEntries(await Promise.all(paths.map(async path => [path, await sha(path)])));
      if (before[paths[0]] !== expectedOracle) throw Error('Pinned Pele oracle changed');
      const oracle = await import('/dist/evidence/pele-oracle.js');
      const {loadRenderer} = await import('/dist/package-web/package/full.js');
      const {cases, runCase, compareCase} = await import(scenarioURL);
      const engine = await loadRenderer(), rows = []; let closeError;
      try {
        for (const spec of cases.filter(row => row.group === group)) {
          const expected = runCase(oracle, spec), actual = runCase(engine, spec, model => engine.dispose(model));
          rows.push({spec, oracle: expected, candidate: actual, problems: compareCase(spec, expected, actual)});
        }
      } finally { try { engine.close(); } catch (value) { closeError = {name: value?.name, message: value?.message}; } }
      const after = Object.fromEntries(await Promise.all(paths.map(async path => [path, await sha(path)])));
      return {group, userAgent: navigator.userAgent, before, after, closeError, rows};
    }, {group, scenarioURL, expectedOracle: ORACLE_SHA256});
    // Persist every row before any parity assertion, including failing SVGs and
    // trace/error/state differences. One mismatch never drops later cases.
    const name = `sankey-resource-${group}.json`, path = info.outputPath(name);
    await writeFile(path, JSON.stringify(report, null, 2) + '\n');
    await info.attach(name, {path, contentType: 'application/json'});
    expect(report.closeError).toBeUndefined(); expect(report.after).toEqual(report.before);
    expect(report.rows).toHaveLength(cases.filter(row => row.group === group).length);
    expect(report.rows.filter(row => row.problems.length).map(row => ({id: row.spec.id, problems: row.problems}))).toEqual([]);
  });
}
