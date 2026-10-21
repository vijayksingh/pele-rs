// Matched comparison of two real Swift API consumers. No builds or guard bypass.
process.env.TZ = 'UTC';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { spawn, execFileSync } from 'node:child_process';
import { createInterface } from 'node:readline';
import { cpus, loadavg, platform, arch, totalmem, freemem } from 'node:os';
import { join, dirname, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import assert from 'node:assert/strict';
import { packageDir, repository, inputs, sha, guardQuiet, systemProcesses } from './build.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const args = process.argv.slice(2);
const arg = (name, fallback) => {
    const index = args.indexOf(name);
    if (index < 0) return fallback;
    assert(index + 1 < args.length && !args[index + 1].startsWith('--'), `Missing value for ${name}`);
    return args[index + 1];
};
const root = repository(), timed = args.includes('--quiet-run');
const controlDirectory = resolve(arg('--control-dir', join(root, 'dist/perf-control/swift')));
const candidateManifestPath = resolve(arg('--candidate-manifest', join(packageDir, '.build/swift-build-manifest.json')));
const controlManifestPath = join(controlDirectory, 'swift-build-manifest.json');
const knownStages = ['parse', 'render-source', 'render-retained', 'parseMutable', 'snapshot', 'apply', 'render', 'lifetime'];
const stages = arg('--stages', knownStages.join(',')).split(',');
assert(stages.length && new Set(stages).size === stages.length && stages.every(stage => knownStages.includes(stage)), 'Unknown or duplicate stage');
const trials = 15, warmupMs = 1000, targetBatchMs = 12;
const ids = arg('--cases', 'gantt/medium,sequence/small').split(',').filter(Boolean);
assert(ids.length && new Set(ids).size === ids.length, 'Cases must be a nonempty unique list');
const { cases, revision: oracleRevision, now } = await import(pathToFileURL(join(root, 'scripts/perf/cases.mjs')));
const selected = cases().filter(workload => ids.includes(workload.id));
assert(ids.every(id => selected.some(workload => workload.id === id)), 'Unknown explicit case');
const bundle = join(root, 'node_modules/pele/dist/pele.js');
const oracleSha256 = sha(await readFile(bundle));
assert.equal(oracleSha256, 'd1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d', 'Pinned development oracle differs');
const upstream = await import(pathToFileURL(bundle));
Date.now = () => now;

async function loadBuild(manifestPath, binaryOverride) {
    const manifestBytes = await readFile(manifestPath);
    const manifest = JSON.parse(manifestBytes);
    assert.equal(manifest.schemaVersion, 1, 'Unsupported Swift build manifest');
    assert(manifest.sourceInputSha256 && manifest.inputHashes && manifest.binarySha256, 'Incomplete build manifest');
    const binary = binaryOverride ?? manifest.binary;
    assert(binary && binary !== '', 'Missing consumer executable');
    const binarySha256 = sha(await readFile(binary));
    assert.equal(binarySha256, manifest.binarySha256, `Executable differs from build manifest: ${binary}`);
    return { manifestPath, manifestSha256: sha(manifestBytes), binary, binarySha256, manifest };
}
const builds = {
    // Frozen manifest deliberately keeps its original build path/provenance.
    control: await loadBuild(controlManifestPath, join(controlDirectory, 'PeleSwiftPerf')),
    candidate: await loadBuild(candidateManifestPath)
};
assert.notEqual(resolve(builds.control.binary), resolve(builds.candidate.binary), 'Control and candidate executable must be distinct paths');
assert.equal(builds.control.manifest.architecture, builds.candidate.manifest.architecture, 'Architecture differs');
assert.equal(builds.control.manifest.swift, builds.candidate.manifest.swift, 'Swift compiler differs');
assert.equal(builds.control.manifest.sdk, builds.candidate.manifest.sdk, 'SDK differs');
const sharedInputs = Object.keys(builds.control.manifest.inputHashes).filter(key => key.startsWith('consumer/'));
sharedInputs.push('root/scripts/perf/cases.mjs');
for (const key of sharedInputs) {
    assert(builds.control.manifest.inputHashes[key], `Missing control input ${key}`);
    assert.equal(builds.control.manifest.inputHashes[key], builds.candidate.manifest.inputHashes[key], `Unmatched consumer/workload input ${key}`);
}
async function validateFrozenInputs() {
    const current = await inputs(root);
    assert.equal(current.sha256, builds.candidate.manifest.sourceInputSha256, 'Candidate source/SDK changed since build');
    for (const [side, build] of Object.entries(builds)) {
        assert.equal(sha(await readFile(build.manifestPath)), build.manifestSha256, `${side} build manifest changed`);
        assert.equal(sha(await readFile(build.binary)), build.binarySha256, `${side} executable changed`);
    }
    assert.equal(sha(await readFile(bundle)), oracleSha256, 'Pinned oracle changed during comparison');
}
await validateFrozenInputs();
const comparatorSha256 = sha(await readFile(fileURLToPath(import.meta.url)));
const inputKeys = [...new Set([...Object.keys(builds.control.manifest.inputHashes), ...Object.keys(builds.candidate.manifest.inputHashes)])].sort();
const changedInputs = inputKeys.filter(key => builds.control.manifest.inputHashes[key] !== builds.candidate.manifest.inputHashes[key])
    .map(key => ({ path: key, control: builds.control.manifest.inputHashes[key] ?? null, candidate: builds.candidate.manifest.inputHashes[key] ?? null }));
const bits = value => { const bytes = Buffer.alloc(8); bytes.writeDoubleBE(value); return bytes.toString('hex'); };
const summary = result => ({ type: result.type, widthBits: bits(result.width), heightBits: bits(result.height), svgSha256: sha(Buffer.from(result.svg)), svgBytes: Buffer.byteLength(result.svg) });
const resources = () => ({ measurer: { width: (text, size, style) => text.length * size * ((style & 1) ? 0.71 : 0.6) } });
const maxBatch = workload => Math.max(1, Math.min(128, Math.floor(25_000 / workload.units)));
const quantile = (values, probability) => {
    const sorted = [...values].sort((a, b) => a - b);
    return sorted[Math.max(0, Math.ceil(sorted.length * probability) - 1)];
};
function statistics(samples) {
    const median = quantile(samples, .5);
    return { median, mad: quantile(samples.map(value => Math.abs(value - median)), .5), min: Math.min(...samples), max: Math.max(...samples), sampleP95: quantile(samples, .95) };
}
const system = () => ({ loadAverage: loadavg(), freeMemory: freemem(), processes: systemProcesses().filter(process => process.cpuPercent >= 10 || /^(cargo|rustc|swiftc|swift-build|swift-frontend|xcodebuild|xctest|ld)$/.test(process.name)).sort((a, b) => b.cpuPercent - a.cpuPercent).slice(0, 20) });
const pairedOrder = index => index % 2 === 0 ? ['control', 'candidate'] : ['candidate', 'control'];

class SwiftConsumer {
    constructor(side, binary) {
        this.side = side; this.pending = []; this.dead = false;
        this.child = spawn(binary, [], { cwd: root, env: { ...process.env, TZ: 'UTC', PELE_PERF_ALLOW_TIMING: timed ? '1' : '0' }, stdio: ['pipe', 'pipe', 'inherit'] });
        this.lines = createInterface({ input: this.child.stdout });
        this.lines.on('line', line => {
            const pending = this.pending.shift();
            if (!pending) { this.child.kill(); this.fail(Error(`${side}: unexpected Swift protocol line`)); return; }
            clearTimeout(pending.timer);
            try {
                const response = JSON.parse(line);
                if (response.ok === true) pending.resolve(response.value);
                else pending.reject(Error(`${side}: ${response.error ?? 'invalid Swift response'}`));
            } catch (error) { pending.reject(error); }
        });
        this.child.on('error', error => this.fail(error));
        this.child.on('exit', (code, signal) => this.fail(Error(`${side}: Swift consumer exited ${code ?? signal}`)));
        this.child.stdin.on('error', error => this.fail(error));
    }
    fail(error) {
        this.dead = true;
        for (const pending of this.pending) { clearTimeout(pending.timer); pending.reject(error); }
        this.pending = [];
    }
    send(request) {
        if (this.dead) return Promise.reject(Error(`${this.side}: consumer closed`));
        assert.equal(this.pending.length, 0, 'Only one in-flight request per consumer');
        return new Promise((resolve, reject) => {
            const timer = setTimeout(() => { this.child.kill(); this.fail(Error(`${this.side}: Swift request exceeded 60 seconds`)); }, 60_000);
            this.pending.push({ resolve, reject, timer });
            this.child.stdin.write(JSON.stringify(request) + '\n');
        });
    }
    close() { this.child.stdin.end(); this.child.kill(); this.lines.close(); }
}
const outputPath = resolve(arg('--output', join(root, 'dist/evidence', timed ? 'swift-comparison.json' : 'swift-comparison-correctness.json')));
const report = {
    schemaVersion: 1,
    mode: timed ? 'matched-local-swift-api-timing' : 'matched-swift-api-correctness-only',
    startedAt: new Date().toISOString(),
    revision: execFileSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).trim(),
    worktree: execFileSync('git', ['status', '--porcelain'], { cwd: root, encoding: 'utf8' }).trim().split('\n').filter(Boolean),
    comparatorSha256, builds, changedInputs, oracleRevision, oracleSha256,
    node: process.version, platform: platform(), arch: arch(), cpu: cpus()[0]?.model, logicalCPUs: cpus().length, totalMemory: totalmem(),
    settings: { trials, warmupMsPerBinaryPerStage: warmupMs, warmupMaximumBatch: 128, targetBatchMs, stages, selectedCases: selected.map(workload => workload.id), fixedClock: now, timeZone: 'UTC', resources: 'UTF16 length × size × (bold?.71:.6)', matchedCounts: true, order: 'Alternate control/candidate first by stage row and trial; 15 pairs give 8/7 per row, opposite imbalance next row', sequentialTiming: true, freshRetainedStatePerBatch: true },
    limitations: [
        `Frozen control source ${builds.control.manifest.revision}; candidate build source ${builds.candidate.manifest.revision}. This measures the combined binary differences listed below, not isolated eager-projection causality.`,
        'Candidate complete source/SDK aggregate is verified against current files. Historical control uses frozen binary and original manifest hashes; historical source tree is not reconstructed or claimed current.',
        'All changed input hashes are recorded. Mutable Gantt improvements may include direct native construction; unrelated stages are controls, not a subtraction estimator.',
        'Per-operation autorelease pool drain is inside timer. Process startup, NDJSON, verification rendering/hashing, retained setup/warm rendering, AppKit/DOM and GPU work are excluded.',
        'Existing eight-stage consumer covers immutable parse and retained/source render, plus mutable APIs; it has no isolated model-access/decodeModel timing. Public differential integration checks remain required.',
        'Exact output means SVG SHA256/bytes and IEEE dimension bits against pinned JS; checksum/arena fields also match between binaries. This does not replace complete public model/Unicode/error parity tests.',
        'No allocation/live-memory measurement, no JS speed ratio, no release or JS parity claim. Medians and paired ratios are descriptive 15-trial observations on a shared host.',
        'Unrelated xctest processes are recorded, not stopped or used to block the run. Shared guardQuiet rejects build and Pele test processes before each timed request; other host load is recorded.'
    ],
    systemBefore: timed ? (guardQuiet(), system()) : system(), cases: [], rows: [], completed: false
};
const consumers = {};
async function persist() { await mkdir(dirname(outputPath), { recursive: true }); await writeFile(outputPath, JSON.stringify(report, null, 2) + '\n'); }
function checkVerification(value, expected, caseID) {
    assert.equal(value.caseID, caseID);
    assert.equal(value.outputs?.length, 5, 'Unexpected verification output count');
    for (const output of value.outputs) assert.deepEqual(output, expected, `${caseID}: exact oracle output`);
}
function checkMeasurement(value, expected, workload, stage, count, timing) {
    assert.equal(value.caseID, workload.id); assert.equal(value.stage, stage); assert.equal(value.count, count);
    assert.deepEqual(value.output, expected, `${workload.id} ${stage}: exact oracle output`);
    assert.equal(typeof value.checksum, 'string');
    if (timing) assert(Number.isSafeInteger(value.elapsedNs) && value.elapsedNs > 0, 'Invalid elapsed time');
    else assert(value.elapsedNs == null, 'Correctness mode unexpectedly timed');
}
function checkPair(values, label) {
    const observable = value => Object.fromEntries(Object.entries(value).filter(([key]) => key !== 'elapsedNs'));
    assert.deepEqual(observable(values.control), observable(values.candidate), `${label}: consumer observable parity`);
}
async function exercisePair(workload, expected, stage, count, timing, order) {
    const values = {}, guards = {};
    for (const side of order) {
        if (timing) guards[side] = { at: new Date().toISOString(), externalLoad: guardQuiet() };
        values[side] = await consumers[side].send({ op: timing ? 'measure' : 'exercise', caseID: workload.id, stage, count });
        checkMeasurement(values[side], expected, workload, stage, count, timing);
    }
    checkPair(values, `${workload.id} ${stage}`);
    return { order, values, ...(timing ? { guards } : {}) };
}
try {
    for (const side of ['control', 'candidate']) consumers[side] = new SwiftConsumer(side, builds[side].binary);
    let rowIndex = 0;
    for (const workload of selected) {
        const expected = summary(upstream.render(workload.source, { ...workload.options, ...resources() }));
        const before = {};
        for (const side of pairedOrder(rowIndex)) {
            before[side] = await consumers[side].send({ op: 'prepare', workload });
            checkVerification(before[side], expected, workload.id);
        }
        assert.deepEqual(before.control, before.candidate, 'Prepare observable parity');
        const caseRecord = { ...workload, sourceSha256: sha(Buffer.from(workload.source)), sourceBytes: Buffer.byteLength(workload.source), expected, before };
        report.cases.push(caseRecord);
        for (const stage of stages) {
            const row = { case: workload.id, stage, before: await exercisePair(workload, expected, stage, 1, false, pairedOrder(rowIndex)), raw: [] };
            report.rows.push(row);
            if (timed) {
                const warmup = { control: { operations: 0, elapsedNs: 0 }, candidate: { operations: 0, elapsedNs: 0 }, pairs: 0 };
                // Both binaries receive identical warmup operation counts. Stop
                // only after each side meets the full one-second target.
                while (warmup.pairs < 6 || warmup.control.elapsedNs < warmupMs * 1e6 || warmup.candidate.elapsedNs < warmupMs * 1e6) {
                    assert(warmup.pairs < 10_000, 'Warmup failed to converge');
                    const count = warmup.pairs < 6 ? 1 : maxBatch(workload);
                    const pair = await exercisePair(workload, expected, stage, count, true, pairedOrder(rowIndex + warmup.pairs));
                    for (const side of ['control', 'candidate']) { warmup[side].operations += count; warmup[side].elapsedNs += pair.values[side].elapsedNs; }
                    warmup.pairs++;
                }
                row.warmup = warmup;
                let count = 1;
                row.calibration = [];
                while (true) {
                    const pair = await exercisePair(workload, expected, stage, count, true, pairedOrder(rowIndex + row.calibration.length));
                    row.calibration.push(pair);
                    // Bound slower side near target; faster side uses exactly
                    // the same count, so API batching never differs by binary.
                    if (count >= maxBatch(workload) || Math.max(pair.values.control.elapsedNs, pair.values.candidate.elapsedNs) >= targetBatchMs * 1e6) break;
                    count = Math.min(maxBatch(workload), count * 2);
                }
                row.count = count;
                for (let trial = 0; trial < trials; trial++) {
                    row.raw.push({ trial, ...await exercisePair(workload, expected, stage, count, true, pairedOrder(rowIndex + trial)) });
                }
                const controlMs = row.raw.map(pair => pair.values.control.elapsedNs / count / 1e6);
                const candidateMs = row.raw.map(pair => pair.values.candidate.elapsedNs / count / 1e6);
                const ratios = candidateMs.map((value, index) => value / controlMs[index]);
                row.statistics = {
                    controlMs: statistics(controlMs), candidateMs: statistics(candidateMs), pairedCandidateOverControl: statistics(ratios),
                    ratioOfMedians: quantile(candidateMs, .5) / quantile(controlMs, .5),
                    candidateFasterPairs: ratios.filter(value => value < 1).length,
                    pairedPercentChanges: ratios.map(value => (value - 1) * 100), controlMs, candidateMs, ratios
                };
            }
            row.after = await exercisePair(workload, expected, stage, row.count ?? 1, false, pairedOrder(rowIndex + 1));
            rowIndex++;
            await persist();
            console.log(`${timed ? 'measured' : 'verified'} ${workload.id} ${stage}${row.statistics ? ` ${row.statistics.candidateFasterPairs}/${trials} faster; paired median ${((row.statistics.pairedCandidateOverControl.median - 1) * 100).toFixed(2)}%` : ''}`);
        }
        caseRecord.after = {};
        for (const side of pairedOrder(rowIndex)) {
            caseRecord.after[side] = await consumers[side].send({ op: 'verify', caseID: workload.id });
            checkVerification(caseRecord.after[side], expected, workload.id);
        }
        assert.deepEqual(caseRecord.after.control, caseRecord.after.candidate, 'Post-run observable parity');
        for (const side of ['control', 'candidate']) assert.deepEqual(caseRecord.after[side], before[side], `${side}: before/after parity`);
    }
    await validateFrozenInputs();
    assert.equal(sha(await readFile(fileURLToPath(import.meta.url))), comparatorSha256, 'Comparator changed during comparison');
    if (timed) guardQuiet();
    report.orderCounts = report.rows.flatMap(row => row.raw).reduce((counts, pair) => { counts[pair.order[0]]++; return counts; }, { control: 0, candidate: 0 });
    if (timed) assert.equal(Math.abs(report.orderCounts.control - report.orderCounts.candidate), report.rows.length % 2, 'Global pair order imbalance');
    report.systemAfter = system(); report.completedAt = new Date().toISOString(); report.completed = true;
    await persist(); console.log(outputPath);
} catch (error) {
    report.failedAt = new Date().toISOString(); report.failure = String(error);
    await persist(); throw error;
} finally {
    for (const consumer of Object.values(consumers)) consumer.close();
}
