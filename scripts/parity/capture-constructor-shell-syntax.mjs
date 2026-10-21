// Capture constructor syntax only at authoring time. Runtime never reads source.
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const load = file => import(pathToFileURL(path.resolve(file)));
const fixturePath = process.argv[2] ?? 'fixtures/public-constructor-reflection.json';
const fixture = JSON.parse(fs.readFileSync(fixturePath));
const corpus = JSON.parse(fs.readFileSync('fixtures/pele/diagram-entries.json'));
if (sha(fs.readFileSync('node_modules/pele/dist/pele.js')) !== fixture.oracleSHA256) throw new Error('Changed pinned oracle');
const mono = await load('node_modules/pele/dist/pele.js');
const proof = [];
for (const row of fixture.rows) {
  const item = corpus.cases.find(item => item.family === row.family);
  const source = row.prototype === 2 ? 'classDiagram\nclass A {\n+method() String\n}' : item.source;
  const entry = (await load(`node_modules/pele/dist/esm/diagrams/${row.family}.js`)).default;
  for (const [lane, model] of [['monolith', mono.parse(source)], ['module', entry.parse(source, item.config, item.title ?? undefined, item.lineOffset, {maxEdges: undefined})]]) {
    const value = row.prototype === 2 ? model.classes.get('A').methods[0] : model;
    const constructor = Object.getPrototypeOf(value).constructor;
    if (constructor.name !== row[lane].name || constructor.length !== row[lane].length) throw new Error('Changed pinned reflection');
    const sourceText = Function.prototype.toString.call(constructor);
    // This pinned minified artifact spells all constructor declarations this way.
    // Full source hashes are retained in the optional authoring proof.
    const implicitConstructor = !/\bconstructor\s*\(/.test(sourceText);
    if (implicitConstructor && constructor.length !== 0) throw new Error('Unexpected implicit constructor arity');
    row[lane].implicitConstructor = implicitConstructor;
    proof.push({family: row.family, prototype: row.prototype, lane, name: constructor.name, implicitConstructor, sourceText, sourceSha256: sha(sourceText)});
  }
}
fixture.syntaxCaptureScriptSHA256 = sha(fs.readFileSync(new URL(import.meta.url)));
fs.writeFileSync(fixturePath, JSON.stringify(fixture, null, 2) + '\n');
if (process.argv[3]) fs.writeFileSync(process.argv[3], JSON.stringify({oracleSHA256: fixture.oracleSHA256, rows: proof}, null, 2) + '\n');
