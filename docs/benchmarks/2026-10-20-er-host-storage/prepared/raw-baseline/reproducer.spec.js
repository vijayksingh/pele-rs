import {test, expect} from '@playwright/test';
import {writeFile} from 'node:fs/promises';

// Public differential baseline, deliberately independent of native parse/owners.
// Any parity failure remains visible in the JSON artifact before assertions run.
test('caller-owned plain ER roots retain live rows, original throws and nested raw renders', async ({page}, testInfo) => {
  await page.goto('/examples/web/');
  let report;
  try {
    report = await page.evaluate(async () => {
      const {loadRenderer} = await import('/dist/package-web/package/full.js');
      const entries = await import('/dist/package-web/package/diagram-entries.js');
      await entries.loadDiagramEntries();
      const nativeEntry = (await import('/dist/package-web/package/diagrams/er.js')).default;
      const oracleEntry = (await import('/dist/evidence/pele-esm/diagrams/er.js')).default;
      const engine = await loadRenderer();
      const source = 'erDiagram\nA { int aid PK }\nB { string bid FK }\nA ||--o{ B : owns\n';
      const config = {}, title = 'Caller raw title', rows = [];
      const numberBits = value => {
        const view = new DataView(new ArrayBuffer(8));
        view.setFloat64(0, value);
        return view.getBigUint64(0).toString(16).padStart(16, '0');
      };

      // Copy data descriptors and container membership through one identity map.
      // Ordinary objects intentionally use plain/null prototypes: the ER class
      // prototype and its helpers are never copied or adopted from the oracle.
      const callerCopy = (input, rootKind = 'Object') => {
        const copies = new Map();
        const copy = value => {
          if (value === null || typeof value !== 'object') return value;
          if (copies.has(value)) return copies.get(value);
          const result = value === input && rootKind === 'Array' ? [] : value === input && rootKind === 'Map' ? new Map()
            : value === input && rootKind === 'Set' ? new Set()
            : value === input && rootKind === 'Date' ? new Date(1704067200000)
            : value === input && rootKind === 'Error' ? new Error('caller-owned root')
            : Array.isArray(value) ? [] : value instanceof Map ? new Map()
            : value instanceof Set ? new Set() : value instanceof Date ? new Date(value.getTime())
            : Object.create(Object.getPrototypeOf(value) === null ? null : Object.prototype);
          copies.set(value, result);
          if (value instanceof Map) for (const [key, item] of Map.prototype.entries.call(value)) {
            Map.prototype.set.call(result, copy(key), copy(item));
          }
          if (value instanceof Set) for (const item of Set.prototype.values.call(value)) {
            Set.prototype.add.call(result, copy(item));
          }
          // Array length comes after indices, so read-only lengths remain valid.
          const names = Reflect.ownKeys(value);
          if (Array.isArray(value)) names.sort((a, b) => a === 'length' ? 1 : b === 'length' ? -1 : 0);
          for (const key of names) {
            const descriptor = Object.getOwnPropertyDescriptor(value, key);
            if (Object.hasOwn(descriptor, 'value')) descriptor.value = copy(descriptor.value);
            Object.defineProperty(result, key, descriptor);
          }
          return result;
        };
        const root = copy(input);
        return {root, copiedObjects: copies.size, allDistinct: [...copies].every(([old, value]) => old !== value)};
      };

      // Canonical complete reachable graph: identities, own-key order, data and
      // accessor descriptors, prototypes, collection membership and IEEE bits.
      const signature = roots => {
        const ids = new Map(), pending = [], nodes = [], symbols = new Map();
        const standard = new Map([
          [Object.prototype, 'Object.prototype'], [Array.prototype, 'Array.prototype'],
          [Map.prototype, 'Map.prototype'], [Set.prototype, 'Set.prototype'],
          [Date.prototype, 'Date.prototype'], [Function.prototype, 'Function.prototype'],
          [Error.prototype, 'Error.prototype'], [TypeError.prototype, 'TypeError.prototype'],
        ]);
        const diagnostics = new Set(['stack', 'fileName', 'lineNumber', 'columnNumber', 'sourceURL']);
        const atom = value => {
          if (value === undefined) return ['undefined'];
          if (value === null) return ['null'];
          if (typeof value === 'number') return ['number', numberBits(value)];
          if (typeof value === 'bigint') return ['bigint', String(value)];
          if (typeof value === 'symbol') {
            if (!symbols.has(value)) symbols.set(value, symbols.size);
            return ['symbol', symbols.get(value), Symbol.keyFor(value), value.description];
          }
          if (typeof value !== 'object' && typeof value !== 'function') return [typeof value, value];
          if (!ids.has(value)) { ids.set(value, pending.length); pending.push(value); }
          return ['ref', ids.get(value)];
        };
        const root = roots.map(atom);
        for (let index = 0; index < pending.length; index++) {
          const value = pending[index], proto = Object.getPrototypeOf(value);
          const kind = Array.isArray(value) ? 'Array' : value instanceof Map ? 'Map'
            : value instanceof Set ? 'Set' : value instanceof Date ? 'Date'
            : value instanceof Error ? 'Error' : typeof value === 'function' ? 'Function' : 'Object';
          const prototype = proto === null ? ['null'] : standard.has(proto) ? ['standard', standard.get(proto)] : atom(proto);
          const properties = Reflect.ownKeys(value)
            .filter(key => !(value instanceof Error && diagnostics.has(key)))
            .map(key => {
              const d = Object.getOwnPropertyDescriptor(value, key);
              return [atom(key), d.enumerable, d.configurable, Object.hasOwn(d, 'value')
                ? ['data', d.writable, atom(d.value)] : ['accessor', atom(d.get), atom(d.set)]];
            });
          const members = kind === 'Map' ? [...Map.prototype.entries.call(value)].map(([key, item]) => [atom(key), atom(item)])
            : kind === 'Set' ? [...Set.prototype.values.call(value)].map(atom)
            : kind === 'Date' ? numberBits(value.getTime()) : null;
          nodes.push({kind, prototype, properties, members, extensible: Object.isExtensible(value)});
        }
        return {root, nodes};
      };
      const basicWidth = (text, size, style) => text.length * size * ((style & 1) ? .71 : .6);
      const errorData = error => ({
        kind: typeof error, name: error?.name, message: error?.message, code: error?.code,
        type: error?.type, line: error?.line, column: error?.column, snippet: error?.snippet,
      });
      const report = {rows, rootControls: []};
      try {
        for (const lane of ['standalone-caller', 'renderer-caller']) {
          for (const variant of ['live-row', 'original-throw', 'nested-raw']) {
            const comparison = {lane, variant};
            for (const implementation of ['oracle', 'native']) {
              const oracle = implementation === 'oracle';
              const render = (root, options) => oracle ? oracleEntry.render(root, config, options)
                : lane === 'standalone-caller' ? nativeEntry.render(root, config, options)
                : engine.renderRaw('er', root, config, options);
              let model;
              try {
                // No native parse or parsePrepared call is used in either lane.
                const parsed = oracleEntry.parse(source, config, title, 7, {maxEdges: undefined});
                const seed = {root: parsed, entity: parsed.entities.get('B'), numbers: [NaN, -0, 0, Infinity, -Infinity]};
                seed.self = seed;
                seed.map = new Map([[seed.entity, seed], ['alias', seed.entity]]);
                seed.set = new Set([seed, seed.entity, seed.numbers]);
                Object.defineProperty(seed, 'hiddenAlias', {value: seed.numbers, enumerable: false, writable: false, configurable: true});
                parsed.callerFixture = seed;
                const copied = callerCopy(parsed);
                model = copied.root;
                const entities = model.entities, relationships = model.relationships;
                const entity = entities.get('B'), attributes = entity.attributes, savedRow = attributes[0];
                const fixture = model.callerFixture, numeric = fixture.numbers;
                const cycle = {model, savedRow}; cycle.self = cycle;
                const sentinel = {marker: 'caller-original-throw', model, savedRow, number: -0};
                sentinel.self = sentinel;
                const allRoots = () => [model, entities, relationships, entity, attributes, savedRow, fixture, numeric, cycle, sentinel];
                const identities = () => ({
                  plainRoot: Object.getPrototypeOf(model) === Object.prototype,
                  noERHelper: !('addEntity' in model),
                  entities: model.entities === entities, relationships: model.relationships === relationships,
                  entity: model.entities.get('B') === entity, attributes: entity.attributes === attributes,
                  row: attributes[0] === savedRow, fixture: model.callerFixture === fixture,
                  rootCycle: fixture.root === model, selfCycle: fixture.self === fixture,
                  mapKey: fixture.map.get(entity) === fixture, mapAlias: fixture.map.get('alias') === entity,
                  setAlias: fixture.set.has(fixture) && fixture.set.has(entity) && fixture.set.has(numeric),
                  hiddenAlias: fixture.hiddenAlias === numeric,
                });
                const before = signature(allRoots()), beforeIdentity = identities();
                const construction = {
                  plainRoot: Object.getPrototypeOf(model) === Object.prototype,
                  distinctFromParsed: model !== parsed, allCopiedObjectsDistinct: copied.allDistinct,
                  copiedObjects: copied.copiedObjects,
                  noCopiedERPrototype: Object.getPrototypeOf(model) !== Object.getPrototypeOf(parsed),
                };
                let fired = false, nested = null;
                const trace = [];
                const outcome = run => {
                  try {
                    const value = run();
                    return {ok: true, svg: value.svg, widthBits: numberBits(value.width), heightBits: numberBits(value.height), result: signature([value])};
                  } catch (error) {
                    return {ok: false, original: Object.is(error, sentinel), ...errorData(error), value: signature([error])};
                  }
                };
                const measure = (phase, args) => {
                  const value = basicWidth(...args);
                  trace.push([phase, 'width', signature(args), numberBits(value)]);
                  return value;
                };
                const first = outcome(() => render(model, {measurer: {width(...args) {
                  // Record the outer call before a nested call or arbitrary throw.
                  trace.push(['first', 'width-enter', signature(args)]);
                  if (!fired) {
                    fired = true;
                    savedRow.name = 'Callback changed caller identifier';
                    savedRow.comment = 'Callback changed caller comment';
                    model.callerCallbackExtension = cycle;
                    trace.push(['first', 'mutation']);
                    if (variant === 'nested-raw') {
                      nested = outcome(() => render(model, {measurer: {width(...inner) { return measure('nested', inner); }}}));
                      trace.push(['first', 'nested-return', nested.ok]);
                    }
                    if (variant === 'original-throw') {
                      model.callerThrown = sentinel;
                      trace.push(['first', 'throw-original']);
                      throw sentinel;
                    }
                  }
                  return measure('first', args);
                }}}));
                const firstTrace = trace.slice(), after = signature(allRoots()), afterIdentity = identities();
                const retained = {
                  name: savedRow.name, comment: savedRow.comment,
                  callbackExtension: model.callerCallbackExtension === cycle,
                  thrownAlias: variant !== 'original-throw' || model.callerThrown === sentinel,
                };
                // Reuse exactly the same root/rows after success or failure; no repair/reparse.
                trace.length = 0;
                const next = outcome(() => render(model, {measurer: {width(...args) { return measure('next', args); }}}));
                comparison[implementation] = {
                  construction, before, beforeIdentity, fired, first, nested, firstTrace, after, afterIdentity,
                  retained, next, nextTrace: trace.slice(), afterNext: signature(allRoots()), nextIdentity: identities(),
                };
              } catch (error) {
                comparison[implementation] = {setupFailure: true, ...errorData(error)};
              } finally {
                if (!oracle && model) {
                  try { if (lane === 'standalone-caller') entries.disposeDiagramModel(model); else engine.dispose(model); }
                  catch (error) { comparison[implementation].cleanupFailure = errorData(error); }
                }
              }
            }
            const keys = new Set([...Object.keys(comparison.oracle), ...Object.keys(comparison.native)]);
            comparison.mismatches = [...keys].filter(key => JSON.stringify(comparison.oracle[key]) !== JSON.stringify(comparison.native[key]));
            rows.push(comparison);
          }
        }
        // Existing drawable admission accepts any supported object reference
        // carrying ER fields. Container/Date/Error roots are controls, not a new guard.
        for (const lane of ['standalone-caller', 'renderer-caller']) {
          for (const rootKind of ['Array', 'Map', 'Set', 'Date', 'Error']) {
            const control = {lane, rootKind};
            for (const implementation of ['oracle', 'native']) {
              const oracle = implementation === 'oracle';
              let model;
              try {
                const parsed = oracleEntry.parse(source, config, title, 7, {maxEdges: undefined});
                model = callerCopy(parsed, rootKind).root;
                const entities = model.entities, relationships = model.relationships;
                const entity = entities.get('B'), attributes = entity.attributes, savedRow = attributes[0];
                const graph = () => signature([model, entities, relationships, entity, attributes, savedRow]);
                const before = graph(), trace = [];
                const render = phase => {
                  const options = {measurer: {width(...args) {
                    const value = basicWidth(...args);
                    trace.push([phase, signature(args), numberBits(value)]);
                    return value;
                  }}};
                  try {
                    const value = oracle ? oracleEntry.render(model, config, options)
                      : lane === 'standalone-caller' ? nativeEntry.render(model, config, options)
                      : engine.renderRaw('er', model, config, options);
                    return {ok: true, svg: value.svg, widthBits: numberBits(value.width), heightBits: numberBits(value.height), result: signature([value])};
                  } catch (error) { return {ok: false, ...errorData(error), value: signature([error])}; }
                };
                const first = render('first'), after = graph(), next = render('next');
                control[implementation] = {
                  first, next, before, after, afterNext: graph(), trace,
                  correctRootKind: rootKind === 'Array' ? Array.isArray(model)
                    : rootKind === 'Map' ? model instanceof Map : rootKind === 'Set' ? model instanceof Set
                    : rootKind === 'Date' ? model instanceof Date : model instanceof Error,
                  originalPrototype: Object.getPrototypeOf(model) === ({Array, Map, Set, Date, Error})[rootKind].prototype,
                  identity: model.entities === entities && model.relationships === relationships
                    && entities.get('B') === entity && entity.attributes === attributes && attributes[0] === savedRow,
                };
              } catch (error) { control[implementation] = {setupFailure: true, ...errorData(error)}; }
              finally {
                if (!oracle && model) {
                  try { if (lane === 'standalone-caller') entries.disposeDiagramModel(model); else engine.dispose(model); }
                  catch (error) { control[implementation].cleanupFailure = errorData(error); }
                }
              }
            }
            const keys = new Set([...Object.keys(control.oracle), ...Object.keys(control.native)]);
            control.mismatches = [...keys].filter(key => JSON.stringify(control.oracle[key]) !== JSON.stringify(control.native[key]));
            report.rootControls.push(control);
          }
        }
      } finally {
        try { engine.close(); } catch (error) { report.cleanupFailure = errorData(error); }
      }
      return report;
    });
  } catch (error) {
    report = {harnessFailure: {name: error.name, message: error.message}, rows: []};
  }

  // Preserve full SVG, exact numeric bits, traces and complete graph snapshots,
  // including both sides of an expected baseline failure, before any assertion.
  const reportPath = testInfo.outputPath('er-raw-caller-baseline.json');
  await writeFile(reportPath, JSON.stringify(report, null, 2) + '\n');
  await testInfo.attach('er-raw-caller-baseline', {path: reportPath, contentType: 'application/json'});
  expect(report.harnessFailure).toBeUndefined();
  expect(report.cleanupFailure).toBeUndefined();
  expect(report.rows).toHaveLength(6);
  expect(report.rootControls).toHaveLength(10);
  for (const row of report.rootControls) {
    for (const implementation of ['oracle', 'native']) {
      const actual = row[implementation], label = `${row.lane}/${row.rootKind}/${implementation}`;
      expect(actual.setupFailure, label).toBeUndefined();
      expect(actual.cleanupFailure, label).toBeUndefined();
      expect(actual.correctRootKind, label).toBe(true);
      expect(actual.originalPrototype, label).toBe(true);
      expect(actual.identity, label).toBe(true);
      expect(actual.first.ok, `${label}: first raw render`).toBe(true);
      expect(actual.next.ok, `${label}: next raw render`).toBe(true);
    }
  }
  expect(report.rootControls.filter(row => row.mismatches.length).map(({lane, rootKind, mismatches}) => ({lane, rootKind, mismatches}))).toEqual([]);
  for (const row of report.rows) {
    for (const implementation of ['oracle', 'native']) {
      const actual = row[implementation], label = `${row.lane}/${row.variant}/${implementation}`;
      expect(actual.setupFailure, label).toBeUndefined();
      expect(actual.cleanupFailure, label).toBeUndefined();
      for (const key of ['plainRoot', 'distinctFromParsed', 'allCopiedObjectsDistinct', 'noCopiedERPrototype']) {
        expect(actual.construction[key], `${label}: ${key}`).toBe(true);
      }
      expect(actual.fired, `${label}: callback`).toBe(true);
      for (const phase of ['beforeIdentity', 'afterIdentity', 'nextIdentity']) {
        expect(Object.values(actual[phase]).every(Boolean), `${label}: ${phase}`).toBe(true);
      }
      expect(actual.retained.name, label).toBe('Callback changed caller identifier');
      expect(actual.retained.comment, label).toBe('Callback changed caller comment');
      expect(actual.retained.callbackExtension, label).toBe(true);
      expect(actual.retained.thrownAlias, label).toBe(true);
      expect(actual.next.ok, `${label}: next raw render`).toBe(true);
      if (row.variant === 'original-throw') {
        expect(actual.first.ok, label).toBe(false);
        expect(actual.first.original, `${label}: original arbitrary throw`).toBe(true);
      } else expect(actual.first.ok, label).toBe(true);
      if (row.variant === 'nested-raw') expect(actual.nested?.ok, `${label}: nested raw render`).toBe(true);
    }
  }
  expect(report.rows.filter(row => row.mismatches.length).map(({lane, variant, mismatches}) => ({lane, variant, mismatches}))).toEqual([]);
});
