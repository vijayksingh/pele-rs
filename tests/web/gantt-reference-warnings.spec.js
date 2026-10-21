import {test, expect} from '@playwright/test';
import {readFile, writeFile} from 'node:fs/promises';

const driver = (await readFile(new URL('./support/constructor-feature.js', import.meta.url), 'utf8'))
  + '\n' + (await readFile(new URL('./support/gantt-reference-warnings.js', import.meta.url), 'utf8'));
const packageRoot = process.env.PELE_TEST_PACKAGE_ROOT ?? '/dist/package-web/package';

// Populate only from captured same-engine reports. Each allowance is tied to
// one named scenario and exact candidate/oracle messages; never normalize graphs.
// Observed WebKit report SHA256: 8be88b3a745c3bcb27d8ca1a3d0dcfc2f572cb02899bd42f31c1acae0130f817
const observedWordingPairs = [
  {
    "id": "shadowed-warn",
    "candidate": "this.warn is not a function",
    "oracle": "this.warn is not a function. (In 'this.warn(()=>`Gantt: \"${e}\" refers to task ids that do not exist: ${a.join(\", \")}.`)', 'this.warn' is null)"
  },
  {
    "id": "second-not-callable",
    "candidate": "this.options.warn is not a function",
    "oracle": "this.options.warn is not a function. (In 'this.options.warn(e())', 'this.options.warn' is 0)"
  }
];
for (const semantic of [false, true]) {
const title = (semantic ? 'semantic references parity: ' : 'exact references diagnostics: ')
  + 'Gantt references preserve lazy warning order, captured task aliases, reentry and original throws';
test(title, async ({page}, info) => {
  test.setTimeout(120000);
  await page.goto('/examples/web/');
  const report = await page.evaluate(async ({source, packageRoot}) => {
    const url = URL.createObjectURL(new Blob([source], {type: 'text/javascript'}));
    try {
      const {runGanttReferences, ganttReferenceCaseIds} = await import(url);
      const {loadDiagramEntries, disposeDiagramModel} = await import(location.origin + packageRoot + '/diagram-entries.js');
      await loadDiagramEntries();
      const candidate = (await import(location.origin + packageRoot + '/diagrams/gantt.js')).default;
      const oracle = (await import(location.origin + '/dist/evidence/pele-esm/diagrams/gantt.js')).default;
      const fixture = await (await fetch('/fixtures/public-gantt-reference-warnings.json')).json();
      return {ids: ganttReferenceCaseIds, oracle: runGanttReferences(oracle), candidate: runGanttReferences(candidate, disposeDiagramModel), fixture};
    } finally { URL.revokeObjectURL(url); }
  }, {source: driver, packageRoot});
  report.comparison = {mode: semantic ? 'semantic' : 'exact', observedWordingPairs};
  const path = info.outputPath('gantt-reference-warnings.json');
  await writeFile(path, JSON.stringify(report, null, 2));
  await info.attach('gantt-reference-warnings.json', {path, contentType: 'application/json'});

  expect(report.fixture.oracleSHA256).toBe('d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d');
  expect(report.ids).toHaveLength(12);
  expect(report.oracle.map(row => row.id)).toEqual(report.fixture.cases.map(row => row.id));
  expect(report.candidate.map(row => row.id)).toEqual(report.ids);
  for (const row of report.oracle) {
    expect(row.fatal, row.id + ' oracle reachable').toBeUndefined();
    const pinned = report.fixture.cases.find(item => item.id === row.id);
    // Node and browser engine-generated TypeError text may differ. Pin every
    // other oracle value to the recorded fixture; compare actual same-engine
    // messages strictly between candidate and oracle below.
    const {error: oracleError, ...first} = row.first;
    const {error: pinnedError, ...pinnedFirst} = pinned.first;
    expect({...row, first}, row.id + ' pinned state/trace').toEqual({...pinned, first: pinnedFirst});
    if (pinnedError && typeof pinnedError === 'object') expect(oracleError?.name, row.id).toBe(pinnedError.name);
    else expect(oracleError, row.id + ' original exception identity').toEqual(pinnedError);
    expect(row.recovery.returnedAlias, row.id + ' recovered alias').toBe(true);
  }
  for (const id of ['known-only', 'shadowed-known', 'warn-false']) {
    const row = report.oracle.find(item => item.id === id);
    expect(row.first.error, id).toBeNull();
    expect(row.first.trace, id).toEqual(id === 'warn-false' ? [['get', 1]] : []);
    expect(row.first.returnedAlias, id).toBe(true);
  }
  for (const id of ['normal', 'keyword-reentry', 'warn-reentry']) {
    const row = report.oracle.find(item => item.id === id);
    expect(row.first.error, id).toBeNull();
    expect(row.first.trace.map(event => event[0]), id).toEqual(['get', 'get', 'keyword', 'call']);
    expect(row.first.returnedAlias, id).toBe(true);
  }
  const actual = structuredClone(report.candidate);
  if (semantic) for (const pair of observedWordingPairs) {
    const candidate = actual.find(row => row.id === pair.id)?.first?.error;
    const oracle = report.oracle.find(row => row.id === pair.id)?.first?.error;
    if (candidate?.name === 'TypeError' && oracle?.name === 'TypeError'
      && candidate.message === pair.candidate && oracle.message === pair.oracle) {
      candidate.message = oracle.message;
    }
  }
  expect(actual).toEqual(report.oracle);
});
}

