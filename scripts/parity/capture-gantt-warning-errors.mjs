import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
const sha=b=>createHash('sha256').update(b).digest('hex'),pin='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
if(sha(fs.readFileSync('node_modules/pele/dist/pele.js'))!==pin)throw Error('Oracle changed');
// Calendar dates in this fixture are UTC, matching the browser feature.
process.env.TZ='UTC';
const support=process.argv[2]??'tests/web/support/gantt-warning-errors.js',output=process.argv[3]??'fixtures/public-gantt-warning-errors.json';
const {runGanttWarningErrors}=await import(pathToFileURL(path.resolve(support))),entry=(await import(pathToFileURL(path.resolve('node_modules/pele/dist/esm/diagrams/gantt.js')))).default;
const cases=runGanttWarningErrors(entry);fs.writeFileSync(output,JSON.stringify({oracleSHA256:pin,driverSHA256:sha(fs.readFileSync(support)),cases},null,2)+'\n');
console.log(JSON.stringify({cases:cases.length,firstErrors:cases.filter(r=>r.first.error?.name==='TypeError').length,recoveryErrors:cases.filter(r=>r.recovery.error).length}));
