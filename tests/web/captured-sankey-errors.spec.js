import {test, expect} from '@playwright/test';
import {writeFile} from 'node:fs/promises';

test.skip(process.env.PELE_CAPTURED_SANKEY !== '1', 'Requires the opt-in captured-sankey WASM build');

const variants = [
  'readonly-node-length', 'readonly-link-length', 'malformed-lookup',
  'latent-arrays-and-repair', 'borrowed-helper-disposal',
  'raw-config-mutation', 'raw-config-original-throw', 'raw-observation-before-config',
];
for (const variant of variants) test(`captured Sankey error lifecycle: ${variant}`, async ({page}, info) => {
  test.setTimeout(90000);
  await page.goto('/examples/web/');
  const report = await page.evaluate(async variant => {
    const {loadRenderer} = await import('/dist/package-web/package/full.js');
    // Snapshot the default policy before enabling the experimental backend.
    const control = await loadRenderer();
    const {enableCapturedSankeyForTesting, capturedSankeyBackend} = await import('/dist/package-web/package/captured-owner.js');
    enableCapturedSankeyForTesting();
    const candidate = await loadRenderer();
    const source = 'sankey\nA,B,5\nB,C,3';
    const normal = {now: 1704067200000, measurer: {width: (text, size) => text.length * size * .6}};
    const view = new DataView(new ArrayBuffer(8));
    const bits = value => {view.setFloat64(0, value); return view.getBigUint64(0).toString(16).padStart(16, '0');};
    const diagnostics = new Set(['stack', 'fileName', 'lineNumber', 'columnNumber', 'sourceURL']);
    // Complete reachable data/prototype/function graph from all supplied roots.
    // Include held detached aliases, descriptor/order, holes, collections and bits.
    // Native error diagnostics are intentionally excluded, as in public adapters.
    const signature = roots => {
      const ids = new Map(), pending = [], nodes = [], symbols = new Map();
      const standard = new Map([[Object.prototype, 'Object'], [Array.prototype, 'Array'],
        [Map.prototype, 'Map'], [Set.prototype, 'Set'], [Date.prototype, 'Date'],
        [Function.prototype, 'Function'], [Error.prototype, 'Error'], [TypeError.prototype, 'TypeError'],
        [RangeError.prototype, 'RangeError']]);
      const atom = value => {
        if (value === undefined) return ['undefined'];
        if (value === null) return ['null'];
        if (typeof value === 'number') return ['number', bits(value)];
        if (typeof value === 'symbol') {if (!symbols.has(value)) symbols.set(value, symbols.size); return ['symbol', symbols.get(value), Symbol.keyFor(value), value.description];}
        if (typeof value === 'bigint') return ['bigint', String(value)];
        if (typeof value !== 'object' && typeof value !== 'function') return [typeof value, value];
        if (!ids.has(value)) {ids.set(value, pending.length); pending.push(value);}
        return ['ref', ids.get(value)];
      };
      const root = roots.map(atom);
      for (let i = 0; i < pending.length; i++) {
        const value = pending[i], proto = Object.getPrototypeOf(value);
        const kind = Array.isArray(value) ? 'Array' : value instanceof Map ? 'Map' : value instanceof Set ? 'Set'
          : value instanceof Date ? 'Date' : value instanceof Error ? 'Error' : typeof value === 'function' ? 'Function' : 'Object';
        const prototype = proto === null ? ['null'] : standard.has(proto) ? ['standard', standard.get(proto)] : atom(proto);
        const properties = Reflect.ownKeys(value).filter(key => !(value instanceof Error && diagnostics.has(key))).map(key => {
          const d = Object.getOwnPropertyDescriptor(value, key);
          return [atom(key), d.enumerable, d.configurable, Object.hasOwn(d, 'value')
            ? ['data', d.writable, atom(d.value)] : ['accessor', atom(d.get), atom(d.set)]];
        });
        const members = kind === 'Map' ? [...Map.prototype.entries.call(value)].map(([k, v]) => [atom(k), atom(v)])
          : kind === 'Set' ? [...Set.prototype.values.call(value)].map(atom) : kind === 'Date' ? bits(Date.prototype.getTime.call(value)) : null;
        nodes.push({kind, prototype, properties, members, extensible: Object.isExtensible(value)});
      }
      return {root, nodes};
    };
    const errorData = error => ({kind: typeof error, name: error?.name, message: error?.message,
      code: error?.code, type: error?.type, line: error?.line, column: error?.column, snippet: error?.snippet});
    const output = value => ({...value, width: bits(value.width), height: bits(value.height)});
    const attempt = (run, convert = value => signature([value])) => {
      try {return {ok: true, value: convert(run())};} catch (error) {return {ok: false, error: errorData(error)};}
    };
    const caller = () => {
      const a = {id: 'A', index: 0}, b = {id: 'B', index: 1};
      const model = {type: 'sankey', nodes: [a, b], links: [{source: a, target: b, value: 5}],
        title: undefined, accTitle: undefined, accDescr: undefined, lookup: new Map([['A', a], ['B', b]])};
      const extra = {zero: -0, text: '\ud800A\udc00', date: new Date(NaN)};extra.self = extra;
      extra.map = new Map([[a, extra], ['b', b]]);extra.set = new Set([extra, a, b]);
      Object.defineProperty(model, 'callerExtra', {value: extra, enumerable: false, writable: false, configurable: true});
      return model;
    };
    const rows = {};
    try {
      for (const [name, engine] of [['control', control], ['candidate', candidate]]) {
        const result = {}, owned = [], hold = []; let model;
        const parse = () => {const value = engine.parse(source); owned.push(value); return value;};
        const draw = () => engine.render(model, normal);
        const snapshot = () => signature([model, ...hold]);
        try {
          const raw = variant.startsWith('raw-');
          model = raw ? caller() : parse();
          if (raw) owned.push(model);
          const nodes = model.nodes, links = model.links, lookup = model.lookup;
          hold.push(nodes, links, lookup, nodes[0], nodes[1]);
          result.initialBackend = capturedSankeyBackend(model);
          if (!raw) result.initial = output(draw());
          result.before = snapshot();
          if (variant === 'readonly-node-length') {
            Object.defineProperty(nodes, 'length', {writable: false});
            result.first = attempt(() => model.findOrCreateNode('D'));
            const cached = lookup.get('D');hold.push(cached);
            result.partial = {cached: !!cached, notInNodes: !nodes.includes(cached), length: nodes.length,
              hit: attempt(() => model.findOrCreateNode('D')), sameCached: model.findOrCreateNode('D') === cached};
            result.afterFailure = snapshot();
            // A readonly Array length cannot be made writable again. Repair by
            // replacing the Array while keeping the old detached reference live.
            model.nodes = [...nodes, cached];
            result.repairedCall = attempt(() => model.addLink(nodes[2], cached, 2));
            result.recovery = attempt(draw, output);result.afterRecovery = snapshot();
          } else if (variant === 'readonly-link-length') {
            Object.defineProperty(links, 'length', {writable: false});
            result.first = attempt(() => model.addLink(nodes[0], nodes[2], -0));
            result.partial = {length: links.length, endpoints: links[0].source === nodes[0] && links[0].target === nodes[1]};
            result.afterFailure = snapshot();model.links = [...links];
            result.repairedCall = attempt(() => model.addLink(nodes[0], nodes[2], 2));
            result.recovery = attempt(draw, output);result.afterRecovery = snapshot();
          } else if (variant === 'malformed-lookup') {
            model.lookup = null;
            result.first = attempt(() => model.findOrCreateNode('D'));
            result.retry = attempt(() => model.findOrCreateNode('D'));
            result.afterFailure = snapshot();result.renderWhileMalformed = attempt(draw, output);
            model.lookup = lookup;result.repairedCall = attempt(() => model.findOrCreateNode('D'));
            result.recovery = attempt(draw, output);result.afterRecovery = snapshot();
          } else if (variant === 'latent-arrays-and-repair') {
            const saved = nodes[1];delete nodes[1];model.links = null;
            result.first = attempt(draw, output);result.retry = attempt(draw, output);result.afterFailure = snapshot();
            nodes[1] = saved;
            result.second = attempt(draw, output); // links error follows repaired node membership.
            model.links = links;links[0].source = {index: NaN};hold.push(links[0].source);
            result.third = attempt(draw, output);result.afterEndpointFailure = snapshot();
            links[0].source = nodes[0];
            result.recovery = attempt(draw, output);result.afterRecovery = snapshot();
          } else if (variant === 'borrowed-helper-disposal') {
            const live = parse(), helper = model.findOrCreateNode;hold.push(live, helper);
            engine.dispose(model);engine.dispose(model);
            result.disposed = {backend: capturedSankeyBackend(model), render: attempt(draw, output),
              raw: attempt(() => engine.renderRaw('sankey', model, {}, normal), output)};
            result.first = attempt(() => helper.call(model, 'D'));
            result.selection = {oldReceiver: lookup.has('D'), liveUntouched: !live.lookup.has('D'),
              retainedNodes: model.nodes === nodes, retainedLookup: model.lookup === lookup};
            result.afterBorrow = snapshot();
            result.liveRecovery = attempt(() => engine.render(live, normal), output);
            engine.dispose(live);
            result.noOwner = attempt(() => helper.call(model, 'E'));
            result.afterNoOwner = snapshot();
          } else if (variant === 'raw-config-mutation') {
            const trace = [], extension = {marker: 'first config'};extension.self = extension;hold.push(extension);
            const config = {get sankey() {
              trace.push('config');model.title = `Config pass ${trace.length}`;
              model.fromConfig = extension;model.links[0].value = 5 + trace.length;
              return {nodeWidth: 11};
            }};
            result.first = attempt(() => engine.renderRaw('sankey', model, config, normal), output);
            result.configTrace = trace.slice();result.afterFirst = snapshot();
            // Config is serialized after observation in both adoption and render.
            // Full publication restores that observed graph, including config writes.
            result.identity = {nodes: model.nodes === nodes, links: model.links === links,
              lookup: model.lookup === lookup, heldCycle: extension.self === extension};
            result.restoredConfigWrites = {extensionAbsent: !Object.hasOwn(model, 'fromConfig'),
              titleUndefined: model.title === undefined, linkValue: bits(model.links[0].value)};
            result.recovery = attempt(() => engine.renderRaw('sankey', model, {}, normal), output);result.afterRecovery = snapshot();
          } else if (variant === 'raw-config-original-throw') {
            const trace = [], sentinel = {marker: 'original config throw'};sentinel.self = sentinel;hold.push(sentinel);
            const config = {get sankey() {trace.push('config');model.title = 'Changed before throw';model.thrown = sentinel;throw sentinel;}};
            try {result.first = {ok: true, value: output(engine.renderRaw('sankey', model, config, normal))};}
            catch (error) {result.first = {ok: false, error: errorData(error), original: error === sentinel};}
            result.configTrace = trace.slice();result.afterFailure = snapshot();
            result.recovery = attempt(() => engine.renderRaw('sankey', model, {}, normal), output);result.afterRecovery = snapshot();
          } else if (variant === 'raw-observation-before-config') {
            let accessed = 0, configured = 0;const sentinel = {configMustNotRun: true};
            Object.defineProperty(model, 'invalidAccessor', {configurable: true, enumerable: true, get() {accessed++;return 1;}});
            const config = {get sankey() {configured++;throw sentinel;}};
            result.first = attempt(() => engine.renderRaw('sankey', model, config, normal), output);
            result.retry = attempt(() => engine.renderRaw('sankey', model, config, normal), output);
            result.priority = {accessed, configured};result.afterFailure = snapshot();
            delete model.invalidAccessor;
            result.recovery = attempt(() => engine.renderRaw('sankey', model, {}, normal), output);result.afterRecovery = snapshot();
          }
          result.finalBackend = capturedSankeyBackend(model);
        } catch (error) {result.setupFailure = errorData(error);}
        finally {
          for (const value of owned) try {engine.dispose(value);} catch (error) {result.cleanupFailure = errorData(error);}
        }
        rows[name] = result;
      }
      return {variant, ...rows};
    } finally {control.close();candidate.close();}
  }, variant);
  const path = info.outputPath(`captured-sankey-${variant}.json`);
  await writeFile(path, JSON.stringify(report, null, 2));
  await info.attach(`captured-sankey-${variant}`, {path, contentType: 'application/json'});
  expect(report.control.setupFailure).toBeUndefined();expect(report.candidate.setupFailure).toBeUndefined();
  expect(report.control.cleanupFailure).toBeUndefined();expect(report.candidate.cleanupFailure).toBeUndefined();
  expect(report.control.initialBackend).toBeUndefined();expect(report.control.finalBackend).toBeUndefined();
  expect(report.candidate.initialBackend).toBe(variant.startsWith('raw-') ? undefined : 'source');
  expect(report.candidate.finalBackend).toBe(variant === 'borrowed-helper-disposal' ? 'disposed' : variant.startsWith('raw-') ? 'raw' : 'source');
  const comparable = result => {
    const {initialBackend, finalBackend, ...rest} = result;
    if (rest.disposed) {const {backend, ...disposed} = rest.disposed;rest.disposed = disposed;}
    return rest;
  };
  expect(comparable(report.candidate), variant).toEqual(comparable(report.control));
  if (variant === 'borrowed-helper-disposal') {
    expect(report.candidate.first.ok).toBe(true);expect(report.candidate.noOwner.ok).toBe(false);
    expect(Object.values(report.candidate.selection).every(Boolean)).toBe(true);
  } else if (variant === 'raw-config-mutation') {
    expect(report.candidate.first.ok).toBe(true);expect(report.candidate.recovery.ok).toBe(true);
    expect(report.candidate.identity).toEqual({nodes: true, links: true, lookup: true, heldCycle: true});
    expect(report.candidate.restoredConfigWrites).toEqual({extensionAbsent: true,
      titleUndefined: true, linkValue: '4014000000000000'});
    expect(report.candidate.configTrace).toEqual(['config', 'config']);
  } else {
    expect(report.candidate.first.ok).toBe(false);expect(report.candidate.recovery.ok).toBe(true);
  }
  if (variant === 'readonly-node-length') {
    expect(report.candidate.partial.cached).toBe(true);expect(report.candidate.partial.notInNodes).toBe(true);
    expect(report.candidate.partial.sameCached).toBe(true);expect(report.candidate.repairedCall.ok).toBe(true);
  }
  if (variant === 'raw-config-original-throw') expect(report.candidate.first.original).toBe(true);
  if (variant === 'raw-observation-before-config') expect(report.candidate.priority).toEqual({accessed: 0, configured: 0});
});
