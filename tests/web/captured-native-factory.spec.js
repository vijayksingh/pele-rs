import {test, expect} from '@playwright/test';
import {writeFile} from 'node:fs/promises';

test.skip(process.env.PELE_NATIVE_CANONICAL_SANKEY !== '1', 'Requires the native-canonical-sankey WASM build');

test('native Sankey factory throws publish restored state and preserve original values without replay', async ({page}, info) => {
  test.setTimeout(120000);
  await page.route('**/dist/package-web/package/host-er-store.js', async route => {
    const response = await route.fetch(), text = await response.text();
    const declaration = 'export function hostErRenderResult(';
    if (!text.includes(declaration)) throw new Error('Expected shared result factory');
    await route.fulfill({response, body: text.replace(declaration, 'function originalNativeFactoryResult(') + `
export function hostErRenderResult(...args) {
  if (globalThis.__nativeFactoryArmed) {
    globalThis.__nativeFactoryTrace.push(['factory']);
    throw globalThis.__nativeFactorySentinel;
  }
  return originalNativeFactoryResult(...args);
}
`});
  });
  await page.goto('/examples/web/');
  const report = await page.evaluate(async () => {
    const {loadRenderer} = await import('/dist/package-web/package/full.js');
    // Loader selection is captured before opt-in; both models come from parse().
    const control = await loadRenderer();
    const {enableCapturedSankeyForTesting, capturedSankeyStorage} = await import('/dist/package-web/package/captured-owner.js');
    enableCapturedSankeyForTesting();
    const candidate = await loadRenderer();
    const bitsView = new DataView(new ArrayBuffer(8));
    const bits = number => {bitsView.setFloat64(0, number); return bitsView.getBigUint64(0).toString(16).padStart(16, '0');};
    const rendered = value => ({keys: Object.keys(value), ...value, width: bits(value.width), height: bits(value.height)});
    const signature = root => {
      const ids = new Map(), pending = [], nodes = [];
      const atom = value => {
        if (value === undefined) return ['undefined'];
        if (typeof value === 'number') return ['number', bits(value)];
        if (typeof value === 'function') return ['function', value.name, value.length];
        if (value === null || typeof value !== 'object') return value;
        if (!ids.has(value)) {ids.set(value, pending.length); pending.push(value);}
        return ['ref', ids.get(value)];
      };
      const result = atom(root);
      for (let index = 0; index < pending.length; index++) {
        const value = pending[index], node = {kind: Array.isArray(value) ? 'array' : value instanceof Map ? 'map' : value instanceof Set ? 'set' : 'object'};
        node.properties = Object.getOwnPropertyNames(value).map(key => {
          const d = Object.getOwnPropertyDescriptor(value, key);
          return [key, d.enumerable, !!d.writable, d.configurable, 'value' in d ? atom(d.value) : ['accessor']];
        });
        if (value instanceof Map) node.entries = [...value].map(([key, item]) => [atom(key), atom(item)]);
        if (value instanceof Set) node.values = [...value].map(atom);
        nodes.push(node);
      }
      return {root: result, nodes};
    };
    const rows = [];
    try {
      for (const raw of [false, true]) for (const kind of ['object', 'undefined', 'symbol']) {
        const row = {raw, kind};
        for (const [name, engine] of [['control', control], ['candidate', candidate]]) {
          const model = engine.parse('sankey\nA,B,5\nB,C,3');
          const extra = {n: 1, text: '\ud800A\udc00', zero: -0};
          extra.self = extra;
          Object.defineProperty(extra, 'hidden', {value: 'original', writable: true, enumerable: false, configurable: true});
          const detached = [extra];
          // Recreated properties append in both backends. Keep extra last among
          // retained root keys so exact ordered restoration remains meaningful.
          model.alias = extra; model.extra = extra; model.detached = detached;
          const nodes = model.nodes, links = model.links, lookup = model.lookup;
          const firstNode = nodes[0], firstLink = links[0], target = firstLink.target;
          const saved = {model, extra, detached, nodes, links, lookup, firstNode, firstLink, target};
          const draw = options => raw ? engine.renderRaw('sankey', model, {}, options) : engine.render(model, options);
          const options = trace => ({now: 1704067200000, measurer: {width(text, size, style) {
            trace.push(['width', text, bits(size), style]);
            return text.length * size * .6;
          }}});
          try {
            // Admit all saved aliases, then make one object historical/detached.
            draw(options([]));
            delete model.detached;
            const baselineTrace = [], baseline = rendered(draw(options(baselineTrace)));
            const before = signature(saved), trace = [];
            const sentinel = kind === 'undefined' ? undefined : kind === 'symbol' ? Symbol('factory sentinel') : {factory: true};
            globalThis.__nativeFactorySentinel = sentinel;
            globalThis.__nativeFactoryTrace = trace;
            globalThis.__nativeFactoryArmed = true;
            let fired = false, threw = false, original = false;
            try {
              draw({now: 1704067200000, measurer: {width(text, size, style) {
                trace.push(['width', text, bits(size), style]);
                if (!fired) {
                  fired = true;
                  extra.n = 99; extra.text = 'changed'; extra.zero = 0; extra.self = {};
                  Object.defineProperty(extra, 'hidden', {value: 'changed', writable: false, enumerable: true, configurable: true});
                  extra.added = {}; detached[0] = {}; detached.push(extra);
                  delete model.extra; model.alias = {}; model.callbackAdded = {};
                  firstNode.id = 'callback node'; firstLink.target = firstNode;
                  lookup.delete('A'); lookup.set('callback key', firstNode);
                }
                return text.length * size * .6;
              }}});
            } catch (error) {
              threw = true; original = Object.is(error, sentinel);
            } finally {
              globalThis.__nativeFactoryArmed = false;
            }
            // Read public objects immediately after the factory error, before recovery.
            const after = signature(saved);
            const aliases = {
              extra: model.extra === extra && model.alias === extra && extra.self === extra,
              detached: detached[0] === extra && detached.length === 1 && !Object.hasOwn(model, 'detached'),
              modelArrays: model.nodes === nodes && model.links === links && model.lookup === lookup,
              graph: nodes[0] === firstNode && links[0] === firstLink && firstLink.target === target && lookup.get('A') === firstNode,
            };
            const recoveryTrace = [], recovery = rendered(draw(options(recoveryTrace)));
            row[name] = {storage: capturedSankeyStorage(model), fired, threw, original, baseline, baselineTrace, trace, before, after, aliases, recovery, recoveryTrace, recoveredState: signature(saved)};
          } finally {
            globalThis.__nativeFactoryArmed = false;
            engine.dispose(model);
          }
        }
        rows.push(row);
      }
      return rows;
    } finally {
      delete globalThis.__nativeFactoryArmed;
      delete globalThis.__nativeFactorySentinel;
      delete globalThis.__nativeFactoryTrace;
      control.close(); candidate.close(); enableCapturedSankeyForTesting(false);
    }
  });
  const path = info.outputPath('captured-native-factory-errors.json');
  await writeFile(path, JSON.stringify(report, null, 2));
  await info.attach('captured-native-factory-errors.json', {path, contentType: 'application/json'});
  expect(report).toHaveLength(6);
  for (const row of report) {
    const label = `${row.raw ? 'raw' : 'normal'}/${row.kind}`;
    expect(row.control.storage, label).toBeUndefined();
    expect(row.candidate.storage, label).toBe('native');
    const {storage: controlStorage, ...control} = row.control;
    const {storage: candidateStorage, ...candidate} = row.candidate;
    expect(candidate, label).toEqual(control);
    for (const result of [control, candidate]) {
      expect(result, label).toMatchObject({fired: true, threw: true, original: true});
      expect(result.baselineTrace.length, label).toBeGreaterThan(0);
      // Public callback traces expose replay without native counters or queries.
      expect(result.trace, label).toEqual([...result.baselineTrace, ['factory']]);
      expect(result.after, label).toEqual(result.before);
      expect(Object.values(result.aliases).every(Boolean), label).toBe(true);
      expect(result.recovery, label).toEqual(result.baseline);
      expect(result.recoveryTrace, label).toEqual(result.baselineTrace);
      expect(result.recoveredState, label).toEqual(result.before);
    }
  }
});
