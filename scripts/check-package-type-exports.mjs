import {mkdtempSync, mkdirSync, copyFileSync, symlinkSync, rmSync, readFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {resolve, dirname, join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const packageRoot = resolve(process.argv[2] ?? join(root, 'dist/package-web/package'));
const fixture = resolve(process.argv[3] ?? join(root, 'tests/types/public-package-exports.mts'));
const compiler = resolve(process.argv[4] ?? join(root, 'node_modules/typescript/bin/tsc'));
const manifest = JSON.parse(readFileSync(join(packageRoot, 'package.json'), 'utf8'));
if (manifest.name !== '@unlocalhosted/pele-rs') throw new Error('Unexpected package identity');
for (const entry of ['.', './full', './lazy', './core', './registry']) {
  if (!manifest.exports[entry]?.types) throw new Error(`Missing public types entry: ${entry}`);
}
const temp = mkdtempSync(join(tmpdir(), 'pele-package-types-'));
try {
  mkdirSync(join(temp, 'node_modules/@unlocalhosted'), {recursive:true});
  symlinkSync(packageRoot, join(temp, 'node_modules/@unlocalhosted/pele-rs'), 'dir');
  copyFileSync(fixture, join(temp, 'consumer.mts'));
  const result = spawnSync(process.execPath, [compiler, '--noEmit', '--strict', '--target', 'ES2022',
    '--module', 'NodeNext', '--moduleResolution', 'NodeNext', '--lib', 'ES2022,DOM',
    join(temp, 'consumer.mts')], {encoding:'utf8'});
  if (result.stdout) process.stdout.write(result.stdout);
  if (result.stderr) process.stderr.write(result.stderr);
  if (result.error) throw result.error;
  if (result.status !== 0) process.exitCode = result.status ?? 1;
  else console.log('PASS: package root/full/lazy/core/registry public type catalog and 33 model names');
} finally {
  rmSync(temp, {recursive:true, force:true});
}
