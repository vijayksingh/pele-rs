#!/usr/bin/env python3
"""Package transport only. Never launches browser, timing harness, or build."""
import argparse,hashlib,json,pathlib,shlex,subprocess,tarfile
from pathlib import Path

ROOT=Path(__file__).resolve().parent
ARCH=ROOT.parent
REMOTE='/home/admin/pele-perf.jL9wzP'
SSH=['ssh','-T','-o','RemoteCommand=none','-o','RequestTTY=no','-o','ConnectTimeout=10','vm-agent']
SCP=['scp','-o','RemoteCommand=none','-o','RequestTTY=no','-o','ConnectTimeout=10']
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()

def frozen_inputs():
    manifest=json.loads((ROOT/'manifest.json').read_text())
    assert manifest.get('candidateSource'),'Candidate pins unset; wait for parent frozen build'
    package=ARCH/'disposal-receipt-runtime/candidate-package'
    freeze=ARCH/'disposal-receipt-runtime/package.json'
    assert sha(freeze)==manifest['candidateManifestSHA256'],'Parent package manifest changed'
    frozen=json.loads(freeze.read_text())
    assert frozen['commit']==manifest['candidateSource']
    actual={str(p.relative_to(package)):sha(p) for p in sorted(package.rglob('*')) if p.is_file()}
    assert actual==frozen['package'] and len(actual)==117
    assert actual['pkg/pele_bg.wasm']==manifest['candidateWasm']
    files=ROOT/'files/scripts/perf'
    for name,digest in manifest['files'].items():assert sha(files/name)==digest,'Runner input drift: '+name
    return manifest,package,freeze

def pack():
    manifest,package,freeze=frozen_inputs()
    target=ROOT/'runtime/transfer.tar.gz';target.parent.mkdir(exist_ok=True)
    assert not target.exists(),'Transfer archive already exists; preserve frozen bundle'
    entries=[]
    for folder,relative in [(package,'parse-architecture/disposal-receipt-runtime/candidate-package'),(ROOT/'files','parse-architecture/disposal-receipt-runner/files')]:
        for p in sorted(folder.rglob('*')):
            assert not p.is_symlink(),'No symlink inputs'
            if p.is_file():entries.append((p,relative+'/'+str(p.relative_to(folder))))
    entries.extend([(freeze,'parse-architecture/disposal-receipt-runtime/package.json'),(ROOT/'manifest.json','parse-architecture/disposal-receipt-runner/manifest.json')])
    with tarfile.open(target,'w:gz',format=tarfile.PAX_FORMAT) as archive:
        for p,name in entries:
            assert '/._' not in name and '__MACOSX' not in name
            info=archive.gettarinfo(str(p),arcname=name);info.uid=info.gid=0;info.uname=info.gname='';info.pax_headers={}
            with p.open('rb') as source:archive.addfile(info,source)
    record={'candidateSource':manifest['candidateSource'],'candidateWasm':manifest['candidateWasm'],'archiveSHA256':sha(target),'remoteRoot':REMOTE,'files':{name:sha(p) for p,name in entries},'containsBrowserOrBuild':False}
    (ROOT/'runtime/transfer.json').write_text(json.dumps(record,indent=2)+'\n')
    print(target)

def transfer():
    frozen_inputs()
    archive=ROOT/'runtime/transfer.tar.gz';record=json.loads((ROOT/'runtime/transfer.json').read_text())
    assert sha(archive)==record['archiveSHA256']
    remote_archive=REMOTE+'/disposal-receipt-transfer.tar.gz'
    subprocess.run(SCP+[str(archive),'vm-agent:'+remote_archive],check=True)
    # Remote files are private immutable run inputs. Refuse overwriting different bytes.
    script='''import tarfile,hashlib,pathlib,sys
root=pathlib.Path(sys.argv[1]);archive=pathlib.Path(sys.argv[2]);expected=sys.argv[3]
assert hashlib.sha256(archive.read_bytes()).hexdigest()==expected
with tarfile.open(archive,'r:gz') as t:
 for m in t.getmembers():
  assert m.isfile() and not pathlib.PurePosixPath(m.name).is_absolute() and '..' not in pathlib.PurePosixPath(m.name).parts
  dst=root/m.name;data=t.extractfile(m).read()
  if dst.exists():assert dst.read_bytes()==data, 'Different existing immutable input: '+str(dst)
with tarfile.open(archive,'r:gz') as t:
 for m in t.getmembers():
  dst=root/m.name
  if not dst.exists():dst.parent.mkdir(parents=True,exist_ok=True);dst.write_bytes(t.extractfile(m).read())
print('Transferred immutable inputs; no runtime job started')
'''
    command='python3 -c '+shlex.quote(script)+' '+shlex.join([REMOTE,remote_archive,record['archiveSHA256']])
    subprocess.run(SSH+[command],check=True)

def fetch(phase):
    # Authorized only after that specific run ends; no process probing or harness invocation.
    destination=ROOT/'runtime'/('fetch-'+phase)
    assert not destination.exists(),'Fetched evidence already exists; do not overwrite'
    destination.mkdir()
    relative='parse-architecture/disposal-receipt-runner'
    remote_archive=REMOTE+'/disposal-receipt-'+phase+'-evidence.tar.gz'
    command='cd '+shlex.quote(REMOTE+'/'+relative)+' && tar -czf '+shlex.quote(remote_archive)+' evidence files manifest.json'
    subprocess.run(SSH+[command],check=True)
    archive=destination/'executed-evidence.tar.gz'
    subprocess.run(SCP+['vm-agent:'+remote_archive,str(archive)],check=True)
    with tarfile.open(archive,'r:gz') as t:
        for m in t.getmembers():
            assert not pathlib.PurePosixPath(m.name).is_absolute() and '..' not in pathlib.PurePosixPath(m.name).parts and (m.isfile() or m.isdir())
        t.extractall(destination,filter='data')
    artifacts={str(p.relative_to(destination)):sha(p) for p in sorted(destination.rglob('*')) if p.is_file()}
    (destination/'fetch-manifest.json').write_text(json.dumps({'phase':phase,'remoteRoot':REMOTE,'artifacts':artifacts,'verification':'Fetched only; run/probe/package semantics must be checked separately'},indent=2)+'\n')
    print(destination)

parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('action',choices=['pack','transfer','fetch'])
parser.add_argument('--phase',choices=['first-use','warm'])
parser.add_argument('--parent-go',action='store_true',help='Required for transfer/fetch; parent must explicitly authorize this action')
a=parser.parse_args()
if a.action=='pack':pack()
elif a.action=='transfer':
    assert a.parent_go,'No transfer authorization supplied';transfer()
else:
    assert a.parent_go and a.phase,'Fetch needs completed phase and parent authorization';fetch(a.phase)
