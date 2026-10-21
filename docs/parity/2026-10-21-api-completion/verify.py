#!/usr/bin/env python3
"""Verify lossless archived bytes and recorded outcomes, without Git or packages."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import tarfile

CHUNK = 1024 * 1024


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(CHUNK), b''):
            h.update(chunk)
    return h.hexdigest()


def safe_path(value):
    p = PurePosixPath(value)
    if p.is_absolute() or '..' in p.parts or not p.parts:
        raise ValueError('Unsafe archived path: ' + value)
    return p


def require(condition, message):
    if not condition:
        raise ValueError(message)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--restore', type=Path, help='Restore every original byte stream beneath a NEW directory')
    parser.add_argument('--excluded-root', type=Path, help='Optionally verify excluded generated artifacts against their original repository root')
    args = parser.parse_args()
    root = Path(__file__).resolve().parent
    manifest_bytes = (root/'manifest.json').read_bytes()
    recorded = (root/'manifest.sha256').read_text().split()[0]
    require(hashlib.sha256(manifest_bytes).hexdigest() == recorded, 'Manifest checksum mismatch')
    m = json.loads(manifest_bytes)
    require(m['format'] == 'pele-api-evidence-v1', 'Unknown archive version')
    files = m['files']
    excluded = m.get('excludedBuildArtifacts', {})
    require(len(excluded) == m.get('excludedCount', 0), 'Excluded artifact count mismatch')
    require(sum(v['bytes'] for v in excluded.values()) == m.get('excludedBytes', 0), 'Excluded artifact byte count mismatch')
    require(not (files.keys() & excluded.keys()), 'Archived/excluded path collision')
    for name, item in excluded.items():
        safe_path(name)
        require(re.fullmatch('[0-9a-f]{64}', item['sha256']) is not None, 'Invalid excluded artifact hash')
        if args.excluded_root:
            original = args.excluded_root / name
            require(original.stat().st_size == item['bytes'] and digest(original) == item['sha256'], 'Excluded artifact changed: '+name)
    for name in files:
        safe_path(name)
    require(len(files) == m['fileCount'], 'File count mismatch')
    require(sum(item['bytes'] for item in files.values()) == m['rawBytes'], 'Raw byte count mismatch')
    objects = {}
    for name, item in files.items():
        require(re.fullmatch('[0-9a-f]{64}', item['sha256']) is not None, 'Invalid content hash')
        require(isinstance(item['bytes'], int) and item['bytes'] >= 0, 'Invalid byte count')
        previous = objects.setdefault(item['sha256'], {'bytes': item['bytes'], 'paths': []})
        require(previous['bytes'] == item['bytes'], 'Inconsistent content identity')
        previous['paths'].append(name)
    require(len(objects) == m['uniqueObjects'], 'Unique object count mismatch')
    require(sum(x['bytes'] for x in objects.values()) == m['uniqueRawBytes'], 'Unique byte count mismatch')
    for item in [m['archive'], m['checkpoint']] + [dict(v, path=k) for k,v in m['tools'].items()]:
        path = root / str(safe_path(item['path']))
        require(path.stat().st_size == item['bytes'] and digest(path) == item['sha256'], 'Bundle checksum mismatch: '+str(path))
    checkpoint = json.loads((root/m['checkpoint']['path']).read_bytes())
    # Read only records referenced by declared verification checks. Unknown or
    # pending outcomes are not promoted to passes by this verifier.
    wanted = {check['evidence'] for check in checkpoint['checks']}
    wanted.update(check['evidence'] for check in checkpoint.get('pending', []) if check.get('evidence'))
    require(wanted <= files.keys(), 'A declared evidence file is absent')
    captured = {}
    restored = args.restore.resolve() if args.restore else None
    if restored:
        require(not restored.exists(), 'Restore destination must not exist')
        restored.mkdir(parents=True)
    seen = set()
    with tarfile.open(root/m['archive']['path'], mode='r|gz') as tar:
        for member in tar:
            require(member.isfile() and member.name.startswith('objects/'), 'Unexpected archive member')
            sha = member.name.removeprefix('objects/')
            require(sha in objects and sha not in seen, 'Unexpected/duplicate content object')
            require(member.size == objects[sha]['bytes'], 'Tar size mismatch')
            seen.add(sha)
            names = objects[sha]['paths']
            outputs = []
            if restored:
                for name in names:
                    destination = restored / str(safe_path(name))
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    outputs.append(destination.open('xb'))
            keep = bool(wanted.intersection(names))
            saved = bytearray() if keep else None
            h = hashlib.sha256()
            try:
                stream = tar.extractfile(member)
                for chunk in iter(lambda: stream.read(CHUNK), b''):
                    h.update(chunk)
                    if keep:
                        saved.extend(chunk)
                    for output in outputs:
                        output.write(chunk)
            finally:
                for output in outputs:
                    output.close()
            require(h.hexdigest() == sha, 'Decoded content checksum mismatch')
            if keep:
                for name in wanted.intersection(names):
                    captured[name] = bytes(saved)
    require(seen == objects.keys(), 'Missing content objects')
    for check in checkpoint['checks']:
        text = captured[check['evidence']].decode('utf-8')
        if check['kind'] == 'contains':
            require(check['value'] in text, 'Missing recorded result: '+check['id'])
        elif check['kind'] == 'json_equals':
            value = json.loads(text)
            for key in check.get('path', []):
                value = value[key]
            require(value == check['value'], 'Recorded result mismatch: '+check['id'])
        else:
            raise ValueError('Unknown result-check kind: '+check['kind'])
    print(f"Verified {len(files)} original paths, {len(objects)} distinct byte streams, {len(checkpoint['checks'])} evidence checks.")
    print('Checkpoint status: '+checkpoint['status']+'. Pending records are not passes; see checkpoint.json.')


if __name__ == '__main__':
    main()
