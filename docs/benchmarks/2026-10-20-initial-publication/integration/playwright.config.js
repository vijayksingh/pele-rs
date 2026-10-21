import {defineConfig} from '@playwright/test';
export default defineConfig({
 testDir:'.',workers:1,timeout:30000,
 use:{baseURL:'http://127.0.0.1:5191',timezoneId:'UTC'},
 projects:[
  {name:'chromium',use:{browserName:'chromium',channel:'chrome'}},
  {name:'firefox',use:{browserName:'firefox'}},
  {name:'webkit',use:{browserName:'webkit'}},
 ],
 webServer:{command:'node scripts/serve.mjs',cwd:new URL('../../../../',import.meta.url).pathname,url:'http://127.0.0.1:5191/examples/web/',reuseExistingServer:true},
 outputDir:'../../../../dist/evidence/rejected-owned-publication-tests',reporter:'list',
});
