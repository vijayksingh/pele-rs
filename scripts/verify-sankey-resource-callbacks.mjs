import {readFile, writeFile, mkdir, readdir} from 'node:fs/promises';
import {resolve, join, dirname} from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
import {cases, runCase, compareCase, ORACLE_SHA256} from '../tests/web/support/sankey-callback-scenarios.js';

// Run from the repository root after integration, against an explicitly built
// package directory. This runner never builds or mutates that package.
const packageRoot = resolve(process.argv[2] ?? 'dist/package-web/package');
const reportPath = resolve(process.argv[3] ?? 'dist/evidence/sankey-resource-callbacks-node.json');
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
async function packageFiles(root, prefix = '') {
  const rows = {};
  for (const item of (await readdir(join(root, prefix), {withFileTypes: true})).sort((a, b) => a.name.localeCompare(b.name))) {
    const name = join(prefix, item.name);
    if (item.isDirectory()) Object.assign(rows, await packageFiles(root, name));
    else if (item.isFile()) rows[name] = hash(await readFile(join(root, name)));
    else throw Error(`Unsupported package entry: ${name}`);
  }
  return rows;
}
const oraclePath = resolve('node_modules/pele/dist/pele.js');
const oracleHash = hash(await readFile(oraclePath));
assert.equal(oracleHash, ORACLE_SHA256, 'Pinned Pele oracle changed');
const before = await packageFiles(packageRoot);
const oracle = await import(pathToFileURL(oraclePath).href);
const {loadRenderer} = await import(pathToFileURL(join(packageRoot, 'full.js')).href);
const engine = await loadRenderer(await readFile(join(packageRoot, 'pkg/pele_bg.wasm')));
const report = {node: process.version, packageRoot, oracle: {path: oraclePath, sha256: oracleHash},
  scenariosSha256: hash(await readFile(new URL('../tests/web/support/sankey-callback-scenarios.js', import.meta.url))), before, rows: []};
try {
  for (const spec of cases) {
    const expected = runCase(oracle, spec), actual = runCase(engine, spec, model => engine.dispose(model));
    report.rows.push({spec, oracle: expected, candidate: actual, problems: compareCase(spec, expected, actual)});
  }
} finally {
  try { engine.close(); } catch (value) { report.closeError = {name: value?.name, message: value?.message}; }
  report.after = await packageFiles(packageRoot);
  report.oracleAfter = hash(await readFile(oraclePath));
  await mkdir(dirname(reportPath), {recursive: true});
  await writeFile(reportPath, JSON.stringify(report, null, 2) + '\n');
}
assert.equal(report.closeError, undefined);
assert.equal(report.oracleAfter, ORACLE_SHA256);
assert.deepEqual(report.after, report.before, 'Candidate package changed during correctness run');
assert.equal(report.rows.length, cases.length);
assert.deepEqual(report.rows.filter(row => row.problems.length).map(row => ({id: row.spec.id, problems: row.problems})), []);
console.log(JSON.stringify({cases: report.rows.length, report: reportPath}));
