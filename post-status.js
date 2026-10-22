const https = require('https');
const u = new URL(process.env.PAPERCLIP_API_URL.replace(/\/$/, '').replace(/\/api$/, '') + '/api/issues/' + process.env.PAPERCLIP_TASK_ID + '/heartbeat-context');
const opts = { hostname: u.hostname, port: u.port || 3100, path: u.pathname, method: 'GET', headers: { Authorization: 'Bearer ' + process.env.PAPERCLIP_API_KEY, 'X-Paperclip-Run-Id': process.env.PAPERCLIP_RUN_ID } };
const req = https.request(opts, (res) => { let d=''; res.on('data',c=>d+=c); res.on('end',()=>{ console.log(d); }); });
req.on('error',e=>console.error('ERR',e.message));
req.end();