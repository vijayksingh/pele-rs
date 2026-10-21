from pathlib import Path
import json,gzip,hashlib
root=Path(__file__).parent
index=json.loads((root/'index.json').read_text())
for path,expected in index['artifacts'].items():
    p=root/path
    raw=p.read_bytes() if p.exists() else gzip.decompress(Path(str(p)+'.gz').read_bytes())
    assert hashlib.sha256(raw).hexdigest()==expected,path
print('Verified',len(index['artifacts']),'original artifacts')
