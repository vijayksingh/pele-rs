import {test, expect} from '@playwright/test';

test.skip(process.env.PELE_NATIVE_CANONICAL_SANKEY !== '1', 'Requires the native canonical WASM build');

test('native Sankey publication preserves long UTF16 labels through source prepared and raw operations', async ({page}, info) => {
  await page.goto('/examples/web/');
  const report = await page.evaluate(async () => {
    const {loadRenderer} = await import('/dist/package-web/package/full.js');
    const control = await loadRenderer();
    const {enableCapturedSankeyForTesting, capturedSankeyStorage} = await import('/dist/package-web/package/captured-owner.js');
    enableCapturedSankeyForTesting();
    const candidate = await loadRenderer();
    // Several native UTF16 pages, including astral pairs and combining marks.
    // Pele's Sankey CSV labels are ASCII; Unicode belongs in the title/raw model.
    const label = 'A'.repeat(2466), title = '🙂'.repeat(1200) + 'e\u0301'.repeat(33);
    const body = `sankey\n"${label}",B,5\nB,C,3`;
    const source = `---\ntitle: ${title}\n---\n${body}`;
    const normal = {now: 1704067200000, measurer: {width: (text, size) => text.length * size * .6}};
    const bitsView = new DataView(new ArrayBuffer(8));
    const bits = value => {bitsView.setFloat64(0, value); return bitsView.getBigUint64(0).toString(16);};
    const output = value => ({...value, width: bits(value.width), height: bits(value.height)});
    const rows = [];
    try {
      for (const lane of ['source', 'prepared', 'raw']) {
        const row = {lane};
        for (const [name, engine] of [['control', control], ['candidate', candidate]]) {
          const config = {};
          const nodes = [{id: label, index: 0}, {id: 'B', index: 1}, {id: 'C', index: 2}];
          const model = lane === 'source' ? engine.parse(source)
            : lane === 'prepared' ? engine.parsePrepared('sankey', body, config, title, 0, undefined)
            : {type: 'sankey', nodes, links: [{source: nodes[0], target: nodes[1], value: 5},
              {source: nodes[1], target: nodes[2], value: 3}], title, lookup: new Map(nodes.map(node => [node.id, node]))};
          const draw = options => lane === 'source' ? engine.render(model, options) : engine.renderRaw('sankey', model, config, options);
          try {
            const initial = output(draw(normal));
            const extra = {text: '\ud800' + title + '\udfff', zero: -0};
            extra.self = extra; model.extra = extra;
            let fired = false;
            const edited = output(draw({...normal, measurer: {width(text, size) {
              if (!fired) {fired = true; extra.text = 'callback edit'; delete model.extra;}
              return text.length * size * .6;
            }}}));
            row[name] = {initial, edited, recovery: output(draw(normal)), fired,
              label: model.nodes[0].id, title: model.title, alias: model.lookup.get(label) === model.nodes[0],
              state: {root: model.extra === extra, cycle: extra.self === extra, text: extra.text, zero: bits(extra.zero)},
              storage: capturedSankeyStorage(model)};
          } finally {engine.dispose(model);}
        }
        rows.push(row);
      }
      return rows;
    } finally {control.close(); candidate.close();}
  });
  await info.attach('native-long-unicode.json', {body: JSON.stringify(report, null, 2), contentType: 'application/json'});
  for (const row of report) {
    expect(row.control.storage).toBeUndefined(); expect(row.candidate.storage).toBe('native');
    delete row.control.storage; delete row.candidate.storage;
    expect(row.candidate, row.lane).toEqual(row.control);
    expect(row.candidate.fired).toBe(true);
    expect(row.candidate.alias).toBe(true);
    expect(row.candidate.state.root).toBe(true);
  }
});
