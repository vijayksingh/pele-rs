import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {gunzipSync} from 'node:zlib';
const file = name => new URL(name, import.meta.url);
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const json = async name => JSON.parse(await readFile(file(name), 'utf8'));
const index = await json('index.json');
for (const [name, hash] of Object.entries(index)) assert.equal(sha(await readFile(file(name))), hash, name);
const source = await json('source-freeze.json'), packed = await json('package-freeze.json'), after = await json('after-full-browser.json');
assert.equal(after.commit, '00d2411954794f7c19ea42eaab0fcd1f816977cf');
assert.equal(after.selectorDefault, false);
assert.equal(after.sourceInputCount, 408);
assert.equal(after.gitSourceInputCount, 408);
assert.deepEqual([...after.gitMappedInputs].sort(), Object.keys(source.hashes).sort());
assert.deepEqual(after.generatedSourceInputs, []);
assert(after.sourceUnchanged && after.packageUnchanged && packed.sourceUnchanged);
assert.equal(Object.keys(packed.hashes).length, 112);
assert.deepEqual(after.packageHashes, packed.hashes);
assert.equal(after.wasmSHA256, packed.hashes['pkg/pele_bg.wasm']);
assert.equal(after.wasmSHA256, 'fbaa003fc95224a27cd8f7a49964a4a271b85f9a1dfaac4c44a3ecc4bed3dee2');
const log = gunzipSync(await readFile(file('full-web.log.gz')));
assert.equal(sha(log), after.fullBrowser.logSHA256);
const text = log.toString();
assert.match(text, /450 passed \(13\.3m\)/);
const rows = [...text.matchAll(/^\s*✓\s+\d+ \[(chromium|firefox|webkit)\]/gm)];
assert.equal(rows.length, 450);
for (const browser of ['chromium', 'firefox', 'webkit']) assert.equal(rows.filter(row => row[1] === browser).length, 150);
assert.equal(after.fullBrowser.existingTests + after.fullBrowser.selectedBackendTests, 450);
assert.equal(after.fullBrowser.failed, 0);
assert.equal(after.fullBrowser.skipped, 0);
let gitVerified = false;
const option = process.argv.indexOf('--repo');
if (option >= 0) {
  const keys = after.gitMappedInputs;
  const bytes = execFileSync('git', ['cat-file', '--batch'], {
    cwd: process.argv[option + 1], input: keys.map(path => `${after.commit}:${path}\n`).join(''), maxBuffer: 100 * 1024 * 1024,
  });
  let offset = 0;
  for (const path of keys) {
    const end = bytes.indexOf(10, offset), header = bytes.subarray(offset, end).toString().split(' ');
    assert.equal(header[1], 'blob');
    const length = Number(header[2]);
    assert.equal(sha(bytes.subarray(end + 1, end + 1 + length)), source.hashes[path], path);
    offset = end + length + 2;
  }
  gitVerified = true;
}
console.log(JSON.stringify({verified: true, passed: 450, packageFiles: 112, sourceFiles: 408, gitVerified, performanceAccepted: false}, null, 2));
