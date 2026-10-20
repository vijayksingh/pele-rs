import {test, expect} from '@playwright/test';

test.skip(process.env.PELE_CAPTURED_SANKEY !== '1', 'Requires the opt-in captured-sankey WASM build');

// Opt-in feature consumer. The default renderer must continue using its current
// backends. Each page creates that control before enabling the private candidate.
test('captured Sankey source prepared and raw models preserve public feature behavior', async ({page}, info) => {
  test.setTimeout(120000);
  await page.goto('/examples/web/');
  const report = await page.evaluate(async () => {
    const {loadRenderer} = await import('/dist/package-web/package/full.js');
    const control = await loadRenderer();
    const {enableCapturedSankeyForTesting, capturedSankeyBackend} = await import('/dist/package-web/package/captured-owner.js');
    enableCapturedSankeyForTesting();
    const candidate = await loadRenderer();
    const oracle = await import('/dist/evidence/pele-oracle.js');
    const referenceEntry = (await import('/dist/evidence/pele-esm/diagrams/sankey.js')).default;
    const source = 'sankey\nA,B,5\nB,C,3';
    const view = new DataView(new ArrayBuffer(8));
    const bits = number => { view.setFloat64(0, number); return view.getBigUint64(0).toString(16).padStart(16, '0'); };
    const signature = root => {
      const ids = new Map(), pending = [], nodes = [];
      const atom = value => {
        if (value === undefined) return ['undefined'];
        if (typeof value === 'number') return ['number', bits(value)];
        if (typeof value === 'function') return ['function', value.name, value.length];
        if (value === null || typeof value !== 'object') return value;
        if (!ids.has(value)) { ids.set(value, pending.length); pending.push(value); }
        return ['ref', ids.get(value)];
      };
      const result = atom(root);
      for (let i = 0; i < pending.length; i++) {
        const value = pending[i], node = {kind: Array.isArray(value) ? 'array' : value instanceof Map ? 'map' : value instanceof Set ? 'set' : value instanceof Date ? 'date' : 'object'};
        node.properties = Object.getOwnPropertyNames(value).map(key => {
          const d = Object.getOwnPropertyDescriptor(value, key);
          return [key, d.enumerable, !!d.writable, d.configurable, 'value' in d ? atom(d.value) : ['accessor']];
        });
        if (value instanceof Map) node.entries = [...value].map(([key, item]) => [atom(key), atom(item)]);
        if (value instanceof Set) node.values = [...value].map(atom);
        if (value instanceof Date) node.time = bits(Date.prototype.getTime.call(value));
        nodes.push(node);
      }
      return {root: result, nodes};
    };
    const rendered = value => ({...value, width: bits(value.width), height: bits(value.height)});
    const options = {now: 1704067200000, measurer: {width: (text, size, style) => text.length * size * (style & 1 ? .71 : .6)}};
    const rows = [];
    try {
      for (const lane of ['source', 'prepared', 'raw']) {
        const row = {lane};
        for (const [name, engine] of [['control', control], ['candidate', candidate], ['oracle', oracle]]) {
          const config = {}, model = lane === 'source' ? engine.parse(source)
            : lane === 'prepared' ? name === 'oracle' ? referenceEntry.parse(source, config, 'Prepared', 7, {maxEdges: undefined}) : engine.parsePrepared('sankey', source, config, 'Prepared', 7, undefined)
            : {type: 'sankey', nodes: [{id: 'A', index: 0}, {id: 'B', index: 1}], links: [], lookup: new Map()};
          if (lane === 'raw') {
            model.links.push({source: model.nodes[0], target: model.nodes[1], value: 5});
            for (const node of model.nodes) model.lookup.set(node.id, node);
          }
          const raw = () => name === 'oracle' ? referenceEntry.render(model, config, options) : engine.renderRaw('sankey', model, config, options);
          const draw = lane === 'source' ? () => engine.render(model, options) : raw;
          const result = {initial: rendered(draw()), before: signature(model)};
          if (name !== 'oracle') result.backend = capturedSankeyBackend(model);
          if (lane !== 'raw') {
            const added = model.findOrCreateNode('D');
            model.addLink(model.lookup.get('B'), added, 2);
            result.alias = model.links.at(-1).target === added && model.lookup.get('D') === added;
          }
          const extra = {text: '\ud800A\udc00', nan: NaN, zero: -0, values: new Set([NaN, -0])};
          extra.self = extra; model.extra = extra;
          model.links[0].value = 7; model.nodes.reverse();
          result.edited = rendered(draw());
          result.after = signature(model);
          result.identity = model.extra === extra && extra.self === extra && Object.is(extra.zero, -0);
          if (name !== 'oracle') {
            engine.dispose(model); result.disposedBackend = capturedSankeyBackend(model);
            try { draw(); result.disposedError = null; } catch (error) { result.disposedError = {name: error.name, message: error.message}; }
          }
          row[name] = result;
        }
        rows.push(row);
      }
      return rows;
    } finally {control.close(); candidate.close();}
  });
  await info.attach('captured-sankey-lanes.json', {body: JSON.stringify(report, null, 2), contentType: 'application/json'});
  for (const row of report) {
    expect(row.control.backend).toBeUndefined();
    expect(row.candidate.backend).toBe(row.lane);
    expect(row.candidate.disposedBackend).toBe('disposed');
    for (const key of ['initial', 'before', 'edited', 'after', 'identity', ...(row.lane === 'raw' ? [] : ['alias'])]) {
      expect(row.candidate[key], `${row.lane}/${key}`).toEqual(row.control[key]);
      expect(row.candidate[key], `${row.lane}/oracle/${key}`).toEqual(row.oracle[key]);
    }
    expect(row.candidate.disposedError).toEqual(row.control.disposedError);
  }
});

test('captured Sankey callbacks restore accepted state and retain original throws like the arena', async ({page}, info) => {
  test.setTimeout(120000);
  await page.goto('/examples/web/');
  const report = await page.evaluate(async () => {
    const {loadRenderer} = await import('/dist/package-web/package/full.js');
    const control = await loadRenderer();
    const {enableCapturedSankeyForTesting, capturedSankeyBackend} = await import('/dist/package-web/package/captured-owner.js');
    enableCapturedSankeyForTesting();
    const candidate = await loadRenderer(), rows = [];
    const view = new DataView(new ArrayBuffer(8));
    const bits = value => {view.setFloat64(0, value); return view.getBigUint64(0).toString(16).padStart(16, '0');};
    const rendered = value => ({...value, width: bits(value.width), height: bits(value.height)});
    const errorView = error => ({name: error?.name, message: error?.message, code: error?.code, type: error?.type});
    const normal = {measurer: {width: (text, size) => text.length * size * .6}};
    try {
      for (const raw of [false, true]) for (const variant of ['edit', 'throw-object', 'throw-undefined', 'throw-symbol', 'nested', 'output-limit']) {
        const row = {raw, variant};
        for (const [name, engine] of [['control', control], ['candidate', candidate]]) {
          const model = engine.parse('sankey\nA,B,5\nB,C,3'), other = engine.parse('sankey\nX,Y,2');
          const extra = {n: 1}; extra.self = extra; model.extra = extra;
          const draw = options => raw ? engine.renderRaw('sankey', model, {}, options) : engine.render(model, options);
          draw(normal);
          const sentinel = variant === 'throw-undefined' ? undefined : variant === 'throw-symbol' ? Symbol('original') : {original: true};
          const result = {trace: []}; let fired = false;
          try {
            result.output = rendered(draw({outputLimit: variant === 'output-limit' ? 0 : Infinity, measurer: {width(text, size, style) {
              result.trace.push([text, bits(size), style]);
              if (!fired) {
                fired = true; extra.n = 99; extra.added = {}; delete model.extra;
                if (variant === 'nested') {
                  try {draw(normal);} catch (error) {result.sameDocument = errorView(error);}
                  result.otherDocument = rendered(engine.render(other, normal));
                }
                if (variant.startsWith('throw')) throw sentinel;
              }
              return text.length * size * .6;
            }}}));
          } catch (error) {result.error = errorView(error); result.original = Object.is(error, sentinel);}
          result.fired = fired;
          result.state = {root: model.extra === extra, cycle: extra.self === extra, n: extra.n, added: Object.hasOwn(extra, 'added')};
          result.recovery = rendered(draw(normal));
          if (name === 'candidate') result.backend = capturedSankeyBackend(model);
          engine.dispose(model); engine.dispose(other); row[name] = result;
        }
        rows.push(row);
      }
      return rows;
    } finally {control.close(); candidate.close();}
  });
  await info.attach('captured-sankey-callbacks.json', {body: JSON.stringify(report, null, 2), contentType: 'application/json'});
  for (const row of report) {
    expect(row.candidate.backend).toBe('source');
    delete row.candidate.backend;
    expect(row.candidate, `${row.raw}/${row.variant}`).toEqual(row.control);
    if (row.variant.startsWith('throw')) expect(row.candidate.original).toBe(true);
  }
});

test('captured source routing keeps ER live callbacks and other diagram families', async ({page}, info) => {
  test.setTimeout(120000);
  await page.goto('/examples/web/');
  const rows = await page.evaluate(async () => {
    const {loadRenderer} = await import('/dist/package-web/package/full.js');
    const control = await loadRenderer();
    const {enableCapturedSankeyForTesting, capturedSankeyBackend} = await import('/dist/package-web/package/captured-owner.js');
    enableCapturedSankeyForTesting();
    const candidate = await loadRenderer(), rows = [];
    const sources = {
      er: 'erDiagram\nA { int id }\nB { string name }\nA ||--o{ B : owns',
      pie: 'pie\n"A":2\n"B":3',
      flowchart: 'flowchart LR\nA[Alpha] --> B[Beta]',
      sankey: 'sankey\nA,B,5',
    };
    try {
      for (const [family, source] of Object.entries(sources)) {
        const row = {family};
        for (const [name, engine] of [['control', control], ['candidate', candidate]]) {
          const model = engine.parse(source), trace = []; let fired = false;
          const result = engine.render(model, {measurer: {width(text, size, style) {
            trace.push([text, size, style]);
            if (!fired) {
              fired = true;
              if (family === 'er') model.entities.get('B').attributes[0].name = 'Changed in callback';
            }
            return text.length * size * .6;
          }}});
          row[name] = {result, trace, fired, ...(family === 'er' ? {attribute: model.entities.get('B').attributes[0].name} : {})};
          if (name === 'candidate') row.backend = capturedSankeyBackend(model);
          engine.dispose(model);
        }
        rows.push(row);
      }
      return rows;
    } finally {control.close(); candidate.close();}
  });
  await info.attach('captured-source-routing.json', {body: JSON.stringify(rows, null, 2), contentType: 'application/json'});
  for (const row of rows) {
    expect(row.backend, row.family).toBe(row.family === 'sankey' ? 'source' : undefined);
    expect(row.candidate, row.family).toEqual(row.control);
  }
});
