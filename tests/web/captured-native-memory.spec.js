import {test, expect} from '@playwright/test';

test.skip(process.env.PELE_NATIVE_CANONICAL_SANKEY !== '1', 'Requires the native-canonical-sankey WASM build');

for (const raw of [false, true]) for (const variant of ['callback-growth', 'publication-growth', 'failed-observation']) {
  test(`native Sankey memory lifecycle: ${raw ? 'raw' : 'source'}/${variant}`, async ({page}, info) => {
    test.setTimeout(120000);
    await page.goto('/examples/web/');
    const report = await page.evaluate(async ({raw, variant}) => {
      const {loadRenderer} = await import('/dist/package-web/package/full.js');
      const control = await loadRenderer();
      const {enableCapturedSankeyForTesting, capturedSankeyStorage} = await import('/dist/package-web/package/captured-owner.js');
      enableCapturedSankeyForTesting();
      const candidate = await loadRenderer();
      const {default: initialize} = await import('/dist/package-web/package/initialize.js');
      const {memory} = await initialize();
      const bitsView = new DataView(new ArrayBuffer(8));
      const bits = value => {bitsView.setFloat64(0, value); return bitsView.getBigUint64(0).toString(16).padStart(16, '0');};
      const output = value => ({...value, width: bits(value.width), height: bits(value.height)});
      const errorView = value => ({kind: typeof value, name: value?.name, message: value?.message, code: value?.code});
      const normal = {now: 1704067200000, measurer: {width: (text, size) => text.length * size * .6}};
      const rows = {};
      try {
        for (const [name, engine] of [['control', control], ['candidate', candidate]]) {
          const model = engine.parse('sankey\nA,B,5\nB,C,3');
          const other = engine.parse('sankey\nX,Y,2');
          const draw = options => raw ? engine.renderRaw('sankey', model, {}, options) : engine.render(model, options);
          const result = {}, extra = {a: 1, b: '\ud800A\udc00', zero: -0};
          extra.self = extra;
          let armed = false, growing = false, grows = 0;
          const grow = () => {
            const before = memory.buffer;
            memory.grow(1);
            if (memory.buffer === before) throw new Error('Memory growth did not refresh its buffer');
            grows++;
          };
          const nested = () => {
            try {draw(normal);} catch (error) {result.sameDocument = errorView(error);}
            const added = other.findOrCreateNode('Z');
            other.addLink(other.lookup.get('Y'), added, 1);
            result.nestedAlias = other.lookup.get('Z') === added && other.links.at(-1).target === added;
            result.nestedOutput = output(engine.render(other, normal));
          };
          const proxy = new Proxy(extra, {defineProperty(target, key, descriptor) {
            if (armed && !growing) {
              armed = false; growing = true;
              try {grow(); nested();} finally {growing = false;}
            }
            return Reflect.defineProperty(target, key, descriptor);
          }});
          model.extra = proxy;
          try {
            result.initial = output(draw(normal));
            result.storage = capturedSankeyStorage(model);
            if (variant === 'failed-observation') {
              // The root interns this suffix before its queued child throws.
              // Subsequent calls must retain ID history while discarding W.
              for (const sentinel of [{original: true}, undefined, Symbol('original')]) {
                model.newHistory = `suffix-${grows}-\ud800\udc00\udfff`;
                let throwing = true;
                model.bad = new Proxy({}, {ownKeys() {
                  if (throwing) {grow(); throw sentinel;}
                  return [];
                }});
                let caught = false;
                try {draw(normal);} catch (error) {
                  caught = true;
                  if (!Object.is(error, sentinel)) throw new Error('Observation replaced the original throw');
                }
                if (!caught) throw new Error('Observation did not visit the throwing object');
                // Detached historical objects remain observed by public policy.
                throwing = false;
                delete model.bad;
                result.repaired = output(draw(normal));
              }
              const added = model.findOrCreateNode('D');
              model.addLink(model.lookup.get('C'), added, 2);
              result.helperAlias = model.lookup.get('D') === added && model.links.at(-1).target === added;
            } else {
              let fired = false;
              result.operation = output(draw({...normal, measurer: {width(text, size) {
                if (!fired) {
                  fired = true;
                  extra.a = 99; extra.b = 'callback mutation'; extra.added = {};
                  if (variant === 'callback-growth') {grow(); nested();}
                  else armed = true;
                }
                return text.length * size * .6;
              }}}));
              if (!fired || grows !== 1) throw new Error('Expected memory growth was not exercised');
            }
            armed = false;
            result.grows = grows;
            result.state = {root: model.extra === proxy, cycle: extra.self === extra, a: extra.a, b: extra.b,
              zero: bits(extra.zero), added: Object.hasOwn(extra, 'added'), nodes: model.nodes.map(node => [node.id, node.index]),
              links: model.links.map(link => [link.source.id, link.target.id, bits(link.value)]), history: model.newHistory};
            result.recovery = output(draw(normal));
          } finally {armed = false; engine.dispose(model); engine.dispose(other);}
          rows[name] = result;
        }
        return rows;
      } finally {control.close(); candidate.close();}
    }, {raw, variant});
    await info.attach('native-memory-lifecycle.json', {body: JSON.stringify(report, null, 2), contentType: 'application/json'});
    expect(report.control.storage).toBeUndefined();
    expect(report.candidate.storage).toBe('native');
    delete report.control.storage; delete report.candidate.storage;
    expect(report.candidate).toEqual(report.control);
    expect(report.candidate.grows).toBe(variant === 'failed-observation' ? 3 : 1);
  });
}
