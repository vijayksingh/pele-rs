// Appended to constructor-feature.js by the test; uses its lossless graph observer.
export const ganttReferenceCaseIds = [
  'known-only', 'shadowed-known', 'shadowed-warn', 'warn-false', 'normal',
  'first-get-throw', 'second-get-throw', 'second-not-callable', 'keyword-throw',
  'keyword-reentry', 'warn-reentry', 'warn-throw',
];

export function runGanttReferences(entry, dispose) {
  const seed = entry.parse('gantt\n dateFormat YYYY-MM-DD\n Task :t, 2026-01-01, 1d', {}, undefined, 0, {});
  const C = seed.constructor;
  if (dispose) dispose(seed);
  return ganttReferenceCaseIds.map(id => {
    let model;
    const trace = [], sentinel = {id}, options = {};
    let reads = 0;
    function callback(message) {
      trace.push(['call', this === options, Array.from({length: message.length}, (_, i) => message.charCodeAt(i))]);
      if (id === 'warn-reentry') {
        model.addTask('Resolved', ':missing,2024-01-01,1d');
        model.tasks[0].task = 'Changed';
      }
      if (id === 'warn-throw') throw sentinel;
    }
    Object.defineProperty(options, 'warn', {configurable: true, get() {
      trace.push(['get', ++reads]);
      if ((id === 'first-get-throw' && reads === 1) || (id === 'second-get-throw' && reads === 2)) throw sentinel;
      return id === 'warn-false' ? false : id === 'second-not-callable' && reads === 2 ? 0 : callback;
    }});
    try {
      model = new C(options);
      model.addTask('Alpha', ':a,2024-01-01,1d');
      model.addTask('Beta', ':b,2024-01-02,1d');
      const originalA = model.tasks[0];
      const keyword = {toString() {
        trace.push(['keyword']);
        if (id === 'keyword-throw') throw sentinel;
        if (id === 'keyword-reentry') {
          model.addSection('Nested');
          Object.defineProperty(options, 'warn', {configurable: true, value: () => {throw Error('replaced callback called');}});
        }
        return 'after\ud800';
      }};
      if (id === 'shadowed-known' || id === 'shadowed-warn') model.warn = null;
      const list = id === 'known-only' || id === 'shadowed-known' ? 'a b a' : ' a missing b missing  a ';
      let result, failure = null;
      try { result = model.references(keyword, list); }
      catch (e) { failure = e === sentinel ? 'original' : error(e); }
      const first = {
        graph: graph(model, C.prototype, result, options), trace: trace.slice(),
        error: failure, returnedAlias: result ? result[0] === originalA : null,
      };
      Object.defineProperty(options, 'warn', {configurable: true, value: undefined});
      if (id === 'shadowed-known' || id === 'shadowed-warn') delete model.warn;
      const recovered = model.references('after', list);
      return {id, list, first, recovery: {
        graph: graph(model, C.prototype, recovered, options), returnedAlias: recovered[0] === originalA,
      }};
    } catch (e) {
      // Keep later scenarios and original diagnostics even when one lane fails.
      return {id, fatal: error(e), trace: trace.slice(), original: e === sentinel};
    } finally { if (dispose && model) dispose(model); }
  });
}
