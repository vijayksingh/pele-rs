// Public feature scenarios shared verbatim by Node and browser runners.
// Expected results come from the pinned Pele package in the same JS engine.
export const ORACLE_SHA256 = 'd1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
export const SOURCE = '---\ntitle: fa:fa-user Original\n---\nsankey\nA,B,5\nB,C,3\nA,C,1';
const failures = ['none', 'object', 'symbol', 'undefined'];
const mutations = ['currentId', 'laterId', 'value', 'endpoint', 'title', 'lookup', 'combined'];
export const cases = [];
function add(group, resource, mutation, failure = 'none', extra = {}) {
  cases.push({id: `${group}/${resource}/${mutation}/${failure}`, group, resource, mutation, failure, ...extra});
}
for (const resource of ['width', 'icon']) {
  for (const mutation of mutations) {
    for (const failure of failures) add(`basic-${resource}`, resource, mutation, failure);
    add('responsive', resource, mutation, 'none', {responsive: true});
  }
  for (const failure of failures) {
    add('detached', resource, 'detached', failure);
    add('same-owner', resource, 'sameCreate', failure);
    add('same-owner', resource, 'sameAddLink', failure);
    add('other-owner', resource, 'otherCreate', failure);
  }
  for (const mutation of ['sameExisting', 'partialCreate', 'partialAddLink']) add('same-owner', resource, mutation);
  for (const mutation of ['deleteCurrentId', 'laterIdNumber', 'replaceNodes', 'accessibility', 'deleteTitle']) add('live-fields', resource, mutation);
  for (const mutation of ['nestedSame', 'nestedOther']) add('nested-render', resource, mutation);
}
export const groups = [...new Set(cases.map(row => row.group))];

const view = new DataView(new ArrayBuffer(8));
function bits(value) { view.setFloat64(0, value); return view.getBigUint64(0).toString(16).padStart(16, '0'); }
function describeError(value, engineErrors) {
  // Generic Error.line/column describe engine implementation stack locations,
  // not Pele source diagnostics. Preserve them raw per lane, outside semantic
  // comparison. PeleError line/column/snippet remain part of the strict result.
  const engineError = value instanceof Error && value.name !== 'PeleError';
  if (engineError) engineErrors.push({name: value.name, message: value.message,
    sourceURL: value.sourceURL, fileName: value.fileName, line: value.line,
    column: value.column, lineNumber: value.lineNumber, columnNumber: value.columnNumber,
    stack: value.stack});
  return {kind: typeof value, name: value?.name, message: value?.message, code: value?.code,
    type: value?.type, line: engineError ? undefined : value?.line,
    column: engineError ? undefined : value?.column, snippet: value?.snippet,
    symbol: typeof value === 'symbol' ? value.description : undefined};
}
function snapshot(roots, retained) {
  const ids = new Map(), objects = [], nodes = [];
  const key = value => typeof value === 'symbol' ? ['symbol', Symbol.keyFor(value), value.description] : ['string', value];
  const atom = value => {
    if (value === null) return ['null'];
    if (typeof value === 'number') return ['number', bits(value)];
    if (typeof value === 'undefined') return ['undefined'];
    if (typeof value === 'symbol') return key(value);
    if (typeof value === 'function') return ['function', value.length];
    if (typeof value !== 'object') return [typeof value, value];
    if (!ids.has(value)) { ids.set(value, objects.length); objects.push(value); }
    return ['ref', ids.get(value)];
  };
  const root = roots.map(atom);
  for (let index = 0; index < objects.length; index++) {
    const object = objects[index], proto = Object.getPrototypeOf(object);
    const kind = Array.isArray(object) ? 'array' : object instanceof Map ? 'map' : object instanceof Set ? 'set' : object instanceof Date ? 'date' : 'object';
    const properties = Reflect.ownKeys(object).map(name => {
      const descriptor = Object.getOwnPropertyDescriptor(object, name);
      return [key(name), descriptor.enumerable, descriptor.configurable,
        Object.hasOwn(descriptor, 'value') ? ['data', descriptor.writable, atom(descriptor.value)] : ['accessor', atom(descriptor.get), atom(descriptor.set)]];
    });
    nodes.push({kind, prototype: proto === null ? 'null' : proto === Object.prototype ? 'Object' : kind === 'object' ? 'model' : kind,
      properties, ...(kind === 'map' ? {entries: [...object].map(([k, v]) => [atom(k), atom(v)])} :
        kind === 'set' ? {values: [...object].map(atom)} : kind === 'date' ? {milliseconds: bits(object.getTime())} : {})});
  }
  if (retained) retained.push(...objects);
  return {root, nodes};
}
function output(value) { return snapshot([value]); }

export function runCase(api, spec, dispose = () => {}) {
  let model, other;
  const result = {cleanup: [], engineErrors: []};
  const error = value => describeError(value, result.engineErrors);
  try {
    model = api.parse(SOURCE);
    if (spec.mutation === 'otherCreate' || spec.mutation === 'nestedOther') other = api.parse(SOURCE);
    const nodes = model.nodes, links = model.links, lookup = model.lookup, first = nodes[0], second = nodes[1];
    const prototype = Object.getPrototypeOf(model), borrowedFind = model.findOrCreateNode;
    let extra, detached, helperResult;
    if (spec.mutation === 'detached') {
      extra = {n: 1}; extra.self = extra; detached = [first]; model.extra = extra; model.detached = detached;
    }
    if (spec.mutation === 'partialCreate') Object.defineProperty(nodes, 'length', {writable: false});
    if (spec.mutation === 'partialAddLink') Object.defineProperty(links, 'length', {writable: false});
    const sentinel = spec.failure === 'object' ? {original: 'callback'} : spec.failure === 'symbol' ? Symbol('callback') : undefined;
    let fired = false;
    const roots = () => [model, nodes, links, lookup, first, second, extra, detached, other, helperResult, sentinel];
    const retained = []; snapshot(roots(), retained);
    const state = () => ({graph: snapshot([...roots(), ...retained]), identity: {
      nodes: model.nodes === nodes, links: model.links === links, lookup: model.lookup === lookup,
      first: model.nodes[0] === first, second: model.nodes[1] === second,
      prototype: Object.getPrototypeOf(model) === prototype,
      endpoints: model.links.every(link => model.nodes.includes(link.source) && model.nodes.includes(link.target)),
      extraCycle: extra ? extra.self === extra : null,
      detachedAlias: detached ? detached[0] === second : null,
      helperLookupAlias: helperResult === undefined ? null : helperResult === (other ?? model).lookup.get(spec.mutation === 'sameExisting' ? 'A' : 'X'),
    }});
    result.initial = state();
    function draw(target, mutating) {
      const trace = [], events = [];
      const row = {trace, events, threw: false, original: false};
      function action() {
        if (!mutating || fired) return;
        fired = true; events.push(['entered', spec.mutation]);
        const long = spec.responsive ? 'Changed B with a deliberately long label that expands drawing bounds' : 'Changed B';
        switch (spec.mutation) {
          case 'currentId': model.nodes[0].id = 'Changed A'; break;
          case 'laterId': model.nodes[1].id = long; break;
          case 'value': model.links[0].value = 99; break;
          case 'endpoint': model.links[0].target = model.nodes[2]; break;
          case 'title': model.title = 'Changed title'; break;
          case 'lookup': model.lookup.clear(); break;
          case 'deleteCurrentId': delete first.id; break;
          case 'laterIdNumber': second.id = 17; break;
          case 'replaceNodes': model.nodes = nodes.map(node => ({...node, id: `Replacement ${node.id}`})); break;
          case 'accessibility': model.accTitle = 'Changed accessible title'; model.accDescr = 'Changed accessible description'; break;
          case 'deleteTitle': delete model.title; break;
          case 'combined':
            model.nodes[0].id = 'Changed A'; model.nodes[1].id = long; model.links[0].value = 99;
            model.links[0].target = model.nodes[2]; model.title = 'Changed title'; model.lookup.clear(); break;
          case 'detached':
            extra.n = 99; extra.added = first; delete model.extra; delete model.detached;
            detached[0] = second; first.id = 'Changed A'; second.id = 'Changed B'; lookup.clear();
            Object.defineProperty(first, 'callbackField', {value: extra, writable: false, enumerable: false, configurable: true}); break;
          case 'sameCreate': case 'sameExisting': case 'partialCreate': case 'otherCreate': {
            const owner = spec.mutation === 'otherCreate' ? other : model;
            try {
              helperResult = spec.mutation === 'otherCreate' ? borrowedFind.call(owner, 'X') : borrowedFind.call(model, spec.mutation === 'sameExisting' ? 'A' : 'X');
              events.push(['helper-return', helperResult === owner.lookup.get(spec.mutation === 'sameExisting' ? 'A' : 'X')]);
            } catch (value) { events.push(['helper-throw', error(value)]); throw value; }
            break;
          }
          case 'sameAddLink': case 'partialAddLink':
            try { const value = model.addLink(first, nodes[2], 2); events.push(['helper-return', snapshot([value])]); }
            catch (value) { events.push(['helper-throw', error(value)]); throw value; } break;
          case 'nestedSame': case 'nestedOther':
            events.push(['nested', draw(spec.mutation === 'nestedOther' ? other : model, false)]); break;
          default: throw Error('Unknown callback scenario');
        }
        events.push(['mutation-complete']);
        if (spec.failure !== 'none') throw sentinel;
      }
      try {
        const value = api.render(target, {...(spec.responsive ? {maxWidth: 220} : {}),
          measurer: {width(text, size, style) {
            trace.push(['width', text, bits(size), style]); if (spec.resource === 'width') action(); return text.length * size * .6;
          }}, icons(name) { trace.push(['icon', name]); if (spec.resource === 'icon') action(); return '<path d="M0 0L1 1"/>'; }});
        row.output = output(value);
      } catch (value) {
        row.threw = true; row.original = spec.failure !== 'none' && Object.is(value, sentinel);
        row.error = row.original ? {sentinel: spec.failure} : error(value);
      }
      return row;
    }
    result.first = draw(model, true); result.fired = fired; result.after = state();
    result.next = draw(model, false); result.afterNext = state();
    if (spec.group === 'live-fields') {
      // Observe the unmodified next-call result before repair. Invalid live
      // values may fail at different phases; only the oracle decides that.
      first.id = 'A'; second.id = 'B'; model.nodes = nodes; model.title = 'fa:fa-user Original';
      result.repaired = draw(model, false); result.afterRepair = state();
    } else if (spec.mutation === 'partialCreate') {
      const cached = lookup.get('X');
      helperResult = model.findOrCreateNode('X');
      result.partialRetry = {same: helperResult === cached, orphan: !nodes.includes(cached)};
      model.nodes = [...nodes, cached]; model.addLink(second, cached, 2);
      result.repaired = draw(model, false); result.afterRepair = state();
    } else if (spec.mutation === 'partialAddLink') {
      model.links = [...links]; model.addLink(first, nodes[2], 2);
      result.repaired = draw(model, false); result.afterRepair = state();
    } else {
      result.helperAfter = {same: borrowedFind.call(model, first.id) === first};
      result.afterHelper = state(); result.final = draw(model, false); result.afterFinal = state();
    }
  } catch (value) { result.fatal = error(value); }
  finally {
    for (const value of [other, model]) if (value) {
      try { dispose(value); } catch (failure) { result.cleanup.push(error(failure)); }
    }
  }
  return JSON.parse(JSON.stringify(result));
}

export function compareCase(spec, oracle, candidate) {
  // Nonvacuity checks are independent of native output. Nothing assumes native
  // rollback, eager importer behavior, or a guessed callback phase.
  const problems = [];
  if (oracle.fatal) problems.push('oracle fixture failed');
  if (!oracle.fired || !candidate.fired) problems.push('callback not reached');
  if (spec.failure !== 'none' && (!oracle.first?.original || !candidate.first?.original)) problems.push('original callback throw not preserved');
  if (spec.failure === 'none' && spec.group !== 'live-fields' && !spec.mutation.startsWith('partial') && oracle.first?.threw) problems.push('oracle success scenario threw');
  if (spec.mutation.startsWith('partial') && !oracle.first?.threw) problems.push('oracle readonly helper did not fail');
  if (spec.group !== 'live-fields' && oracle.next?.threw) problems.push('oracle recovery draw failed');
  if (oracle.repaired?.threw || oracle.final?.threw) problems.push('oracle final repaired draw failed');
  if (spec.mutation === 'partialCreate' && (!oracle.partialRetry?.same || !oracle.partialRetry?.orphan)) problems.push('oracle partial helper alias not established');
  if (['sameCreate', 'sameExisting', 'otherCreate'].includes(spec.mutation) && !oracle.first?.events.some(event => event[0] === 'helper-return' && event[1] === true)) problems.push('oracle helper return alias not established');
  if (spec.mutation === 'nestedSame' || spec.mutation === 'nestedOther') {
    const nested = oracle.first?.events.find(event => event[0] === 'nested');
    if (!nested || nested[1].threw || !nested[1].output) problems.push('oracle nested render did not succeed');
  }
  if (oracle.cleanup.length || candidate.cleanup.length) problems.push('cleanup failed');
  // Exclude only our top-level diagnostic recorder, never caller graph fields.
  const {engineErrors: candidateEngineErrors, ...candidateSemantic} = candidate;
  const {engineErrors: oracleEngineErrors, ...oracleSemantic} = oracle;
  if (JSON.stringify(candidateSemantic) !== JSON.stringify(oracleSemantic)) problems.push('public result differs from pinned Pele');
  return problems;
}
