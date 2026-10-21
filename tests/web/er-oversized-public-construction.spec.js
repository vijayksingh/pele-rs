import {test, expect} from '@playwright/test';
import {writeFile} from 'node:fs/promises';

test('ER oversized public collections preserve complete models and retained helper identity', async ({page}, testInfo) => {
  test.setTimeout(120_000);
  await page.goto('/examples/web/');
  const report = await page.evaluate(async () => {
    const {loadRenderer} = await import('/dist/package-web/package/full.js');
    const oracle = await import('/dist/evidence/pele-oracle.js');
    const engine = await loadRenderer(), count = 7000, rows = [];
    const options = {maxEdges: 20_000, limit: 1_000_000};
    const fixtures = [
      {
        name: 'many-bare-entities',
        source: 'erDiagram\n' + Array.from({length: count}, (_, i) => `E${i}`).join('\n'),
        firstKey: 'E0', lastKey: `E${count - 1}`, entities: count, relationships: 0,
      },
      {
        name: 'many-repeated-relationships',
        source: 'erDiagram\n' + 'A ||--o{ B : owns\n'.repeat(count),
        firstKey: 'A', lastKey: 'B', entities: 2, relationships: count,
      },
      {
        // One relationship Ref costs five compact bytes. This crosses the
        // bounded dense-array event while the preceding 7,000 case fits it.
        name: 'many-repeated-relationships-beyond-dense-frame',
        source: 'erDiagram\n' + 'A ||--o{ B : owns\n'.repeat(13_106),
        firstKey: 'A', lastKey: 'B', entities: 2, relationships: 13_106,
      },
    ];

    // Complete reachable data graph, with identity IDs rather than flattened
    // copies. Own order/presence/descriptors, cycles and IEEE bits stay visible.
    const signature = roots => {
      const ids = new Map(), pending = [], nodes = [], numeric = new DataView(new ArrayBuffer(8));
      const standard = [Object.prototype, Array.prototype, Map.prototype, Set.prototype, Date.prototype];
      const diagnostics = new Set(['stack', 'fileName', 'lineNumber', 'columnNumber', 'sourceURL']);
      const number = value => {
        numeric.setFloat64(0, value);
        return numeric.getBigUint64(0).toString(16).padStart(16, '0');
      };
      const atom = value => {
        if (value === undefined) return ['undefined'];
        if (value === null) return ['null'];
        if (typeof value === 'number') return ['number', number(value)];
        if (typeof value === 'bigint') return ['bigint', value.toString()];
        if (typeof value === 'symbol') return ['symbol', Symbol.keyFor(value), value.description];
        // Function source/name can differ across bundled implementations; public
        // arity and own metadata shape are their existing compatibility contract.
        if (typeof value === 'function') return ['function', value.length, Reflect.ownKeys(value).filter(key => key !== 'name')];
        if (typeof value !== 'object') return [typeof value, value];
        if (!ids.has(value)) { ids.set(value, pending.length); pending.push(value); }
        return ['ref', ids.get(value)];
      };
      const key = value => typeof value === 'symbol' ? atom(value) : value;
      const descriptor = d => [
        d.enumerable, d.configurable,
        Object.hasOwn(d, 'value') ? ['data', d.writable, atom(d.value)] : ['accessor', atom(d.get), atom(d.set)],
      ];
      const root = roots.map(atom);
      for (let index = 0; index < pending.length; index++) {
        const value = pending[index], prototype = Object.getPrototypeOf(value);
        const kind = Array.isArray(value) ? 'Array' : value instanceof Map ? 'Map'
          : value instanceof Set ? 'Set' : value instanceof Date ? 'Date'
          : value instanceof Error ? `Error:${value.name}` : prototype === null ? 'NullObject' : 'Object';
        const members = prototype && !standard.includes(prototype) && !(value instanceof Error)
          ? Reflect.ownKeys(prototype).filter(name => name !== 'constructor')
            .map(name => [key(name), ...descriptor(Object.getOwnPropertyDescriptor(prototype, name))])
          : [];
        // Stack/source locations describe this test's call site, not model data.
        // Library fields such as code/type/line/column/snippet remain exact.
        const properties = Reflect.ownKeys(value)
          .filter(name => !(value instanceof Error && diagnostics.has(name)))
          .map(name => [key(name), ...descriptor(Object.getOwnPropertyDescriptor(value, name))]);
        const content = kind === 'Map' ? [...value].map(([k, v]) => [atom(k), atom(v)])
          : kind === 'Set' ? [...value].map(atom) : kind === 'Date' ? number(value.getTime()) : null;
        nodes.push([kind, members, properties, content]);
      }
      return {text: JSON.stringify([root, nodes]), nodes: nodes.length};
    };
    const outcome = run => {
      try { return {ok: true, value: run()}; }
      catch (error) { return {ok: false, error}; }
    };

    for (const fixture of fixtures) {
      const failures = [], lanes = {}, row = {name: fixture.name, sourceBytes: new TextEncoder().encode(fixture.source).length};
      const check = (condition, stage) => { if (!condition) failures.push(stage); };
      const compare = (stage, left, right) => {
        if (left === right) return;
        failures.push(stage);
        let offset = 0;
        while (offset < left.length && offset < right.length && left[offset] === right[offset]) offset++;
        (row.differences ??= []).push({
          stage, offset, jsLength: left.length, wasmLength: right.length,
          js: left.slice(Math.max(0, offset - 40), offset + 120),
          wasm: right.slice(Math.max(0, offset - 40), offset + 120),
        });
      };
      try {
        for (const [name, renderer] of [['js', oracle], ['wasm', engine]]) {
          const parsed = outcome(() => renderer.parse(fixture.source, options));
          lanes[name] = {renderer, parsed};
          check(parsed.ok, `${name}: parse succeeds`);
          if (!parsed.ok) continue;
          const model = parsed.value, entities = model.entities, relationships = model.relationships;
          const entries = [...entities], first = entities.get(fixture.firstKey), last = entities.get(fixture.lastKey);
          const firstRelationship = relationships[0], lastRelationship = relationships.at(-1);
          const prototype = Object.getPrototypeOf(model), addEntity = model.addEntity;
          const attributes = first.attributes, addAttributes = model.addAttributes, addRelationship = model.addRelationship;
          Object.assign(lanes[name], {model, entities, relationships, first, last, firstRelationship, lastRelationship, prototype, addEntity, attributes, addAttributes, addRelationship});
          check(model.type === 'er', `${name}: ER type`);
          check(entities.size === fixture.entities && relationships.length === fixture.relationships, `${name}: collection counts`);
          check(entries[0][0] === fixture.firstKey && entries.at(-1)[0] === fixture.lastKey, `${name}: first/last Map membership`);
          check(entries[0][1] === first && entries.at(-1)[1] === last && first !== last, `${name}: first/last entity identity`);
          if (fixture.relationships) {
            check(firstRelationship !== lastRelationship, `${name}: distinct first/last relationships`);
            for (const relationship of [firstRelationship, lastRelationship]) {
              check(relationship.entityA === first.id && relationship.entityB === last.id && relationship.roleA === 'owns', `${name}: relationship endpoints and role`);
            }
          }
          lanes[name].initial = signature([model, entities, relationships, first, last, firstRelationship, lastRelationship]);
          lanes[name].membership = {
            entities: entities.size, relationships: relationships.length,
            first: [entries[0][0], first.id], last: [entries.at(-1)[0], last.id],
            firstRelationship: firstRelationship && [firstRelationship.entityA, firstRelationship.roleA, firstRelationship.entityB],
            lastRelationship: lastRelationship && [lastRelationship.entityA, lastRelationship.roleA, lastRelationship.entityB],
          };
        }

        check(signature([lanes.js.parsed.ok ? 'parsed' : lanes.js.parsed]).text === signature([lanes.wasm.parsed.ok ? 'parsed' : lanes.wasm.parsed]).text, 'parse outcome/type/error equality');
        if (!lanes.js.parsed.ok || !lanes.wasm.parsed.ok) {
          row.parse = {
            js: signature([lanes.js.parsed.ok ? 'parsed' : lanes.js.parsed]).text,
            wasm: signature([lanes.wasm.parsed.ok ? 'parsed' : lanes.wasm.parsed]).text,
          };
          continue;
        }
        compare('complete initial model signature', lanes.js.initial.text, lanes.wasm.initial.text);
        row.initialNodes = {js: lanes.js.initial.nodes, wasm: lanes.wasm.initial.nodes};
        row.membership = {js: lanes.js.membership, wasm: lanes.wasm.membership};
        // Release the large initial strings before the retained-operation phase.
        delete lanes.js.initial; delete lanes.wasm.initial;

        for (const name of ['js', 'wasm']) {
          const lane = lanes[name], {model, entities, relationships, first, last, attributes} = lane;
          const extension = {model, first, last, numbers: [NaN, -0, 0, Infinity, -Infinity], missing: undefined, raw: '\ud800A\udc00'};
          extension.self = extension;
          extension.error = new TypeError('retained constructor fixture');
          extension.error.code = 'fixture'; extension.error.numeric = -0;
          extension.errorAlias = extension.error;
          Object.defineProperty(extension, 'hiddenAlias', {value: extension.numbers, enumerable: false, writable: false, configurable: true});
          model.fixtureExtension = extension;
          const attribute = {type: 'string', name: 'retained_id', keys: ['PK'], comment: 'added after oversized parse'};
          const spec = {cardA: 'ONLY_ONE', relType: 'IDENTIFYING', cardB: 'ZERO_OR_MORE'};
          const results = [
            outcome(() => model.addEntity(fixture.firstKey)),
            outcome(() => model.addEntity(fixture.lastKey, 'Last retained alias')),
            outcome(() => model.addAttributes(fixture.firstKey, [attribute])),
            outcome(() => model.addRelationship(fixture.firstKey, 'added after parse', fixture.lastKey, spec)),
          ];
          check(results.every(result => result.ok), `${name}: all retained helpers succeed`);
          check(results[0].value === first && results[1].value === last, `${name}: helper entity return aliases`);
          check(results[2].value === undefined && results[3].value === undefined, `${name}: helper undefined returns`);
          check(model.entities === entities && model.relationships === relationships, `${name}: collection identities stable`);
          check(entities.get(fixture.firstKey) === first && entities.get(fixture.lastKey) === last, `${name}: entity identities stable`);
          check(first.attributes === attributes && attributes.at(-1) === attribute, `${name}: attribute array and argument identity`);
          check(relationships.length === fixture.relationships + 1 && relationships.at(-1).relSpec === spec, `${name}: appended relationship and spec alias`);
          check(entities.size === fixture.entities && last.alias === 'Last retained alias', `${name}: existing-entity helper preserves membership`);
          if (fixture.relationships) {
            check(relationships[0] === lane.firstRelationship && relationships[fixture.relationships - 1] === lane.lastRelationship, `${name}: original first/last relationship identities`);
          }
          check(Object.getPrototypeOf(model) === lane.prototype && model.addEntity === lane.addEntity && model.addAttributes === lane.addAttributes && model.addRelationship === lane.addRelationship, `${name}: prototype and helper identity`);
          check(model.fixtureExtension === extension && extension.self === extension && extension.model === model && extension.hiddenAlias === extension.numbers && extension.errorAlias === extension.error, `${name}: cycles and extra aliases`);
          check(Object.is(extension.numbers[1], -0) && Number.isNaN(extension.numbers[0]) && Object.is(extension.error.numeric, -0), `${name}: numeric values preserved`);
          lane.after = signature([model, results, entities, relationships, first, last, attributes, lane.firstRelationship, lane.lastRelationship, attribute, spec, extension]);
        }
        compare('complete post-helper model/return/alias/error/descriptor signature', lanes.js.after.text, lanes.wasm.after.text);
        row.afterNodes = {js: lanes.js.after.nodes, wasm: lanes.wasm.after.nodes};
      } finally {
        const model = lanes.wasm?.parsed.ok && lanes.wasm.parsed.value;
        if (model) engine.dispose(model);
        row.failures = failures; rows.push(row);
      }
    }
    return rows;
  });

  const path = testInfo.outputPath('er-oversized-public-construction.json');
  await writeFile(path, JSON.stringify(report, null, 2));
  await testInfo.attach('er-oversized-public-construction.json', {path, contentType: 'application/json'});
  expect(report).toHaveLength(3);
  expect(report[0].sourceBytes).toBeLessThan(64 * 1024);
  expect(report[1].sourceBytes).toBeGreaterThan(64 * 1024);
  expect(report[2].sourceBytes).toBeGreaterThan(64 * 1024);
  expect(report[2].sourceBytes).toBeLessThan(1_000_000);
  expect(report.map(({name, failures}) => ({name, failures})), JSON.stringify(report)).toEqual([
    {name: 'many-bare-entities', failures: []},
    {name: 'many-repeated-relationships', failures: []},
    {name: 'many-repeated-relationships-beyond-dense-frame', failures: []},
  ]);
});
