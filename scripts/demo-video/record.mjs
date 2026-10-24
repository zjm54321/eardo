// Staging-only real browser recording. See README.md.
import fs from 'node:fs';
import path from 'node:path';
import http from 'node:http';
import { spawn, execFileSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';
const OUT = process.env.DEMO_OUT || '/tmp/opencode/demo-video';
const BASE = 'https://dev.eardoai.com';
const { chromium } = await import(pathToFileURL(`${OUT}/node_modules/playwright-core/index.mjs`));
fs.mkdirSync(`${OUT}/clips`, { recursive: true });
fs.mkdirSync(`${OUT}/frames`, { recursive: true });
const sleep = ms => new Promise(r => setTimeout(r, ms));
// Do not launch a browser until a cache-busted deployment is live.
for (let i = 0; ; i++) {
  const html = await (await fetch(`${BASE}/generate`)).text();
  const release = html.match(/\/pkg\/eardo\.[A-Za-z0-9_-]*\.js/);
  if (release) { fs.writeFileSync(`${OUT}/release.txt`, release[0]); break; }
  if (i === 39) throw Error('预发布部署等待超过20分钟');
  await sleep(30000);
}
const chrome = spawn('timeout', ['1800', 'google-chrome', '--headless=new', '--no-sandbox',
  '--disable-gpu', '--hide-scrollbars', '--autoplay-policy=no-user-gesture-required',
  '--remote-debugging-port=9451', '--window-size=1440,900',
  `--user-data-dir=${OUT}/profile-${Date.now()}`, 'about:blank'], { stdio: 'ignore' });
let browser;
for (let i = 0; i < 60; i++) {
  try { browser = await chromium.connectOverCDP('http://127.0.0.1:9451'); break; } catch { await sleep(250); }
}
if (!browser) throw Error('Chrome did not start');
const context = browser.contexts()[0];
await context.grantPermissions(['clipboard-read', 'clipboard-write'], { origin: BASE });
const page = context.pages()[0];
await page.setViewportSize({ width: 1440, height: 900 });
page.setDefaultTimeout(10000);
// Prevent accidental writes/requests to production, including inherited absolute URLs.
await context.route('**/*', route => {
  const host = new URL(route.request().url()).hostname;
  return /^(www\.)?eardoai\.com$/.test(host) ? route.abort('blockedbyclient') : route.continue();
});
const logs = [];
page.on('console', m => { if (m.type() === 'error') logs.push({ time: new Date().toISOString(), url: page.url(), type: 'console', text: m.text() }); });
page.on('pageerror', e => logs.push({ time: new Date().toISOString(), url: page.url(), type: 'exception', text: e.message }));
page.on('response', async r => { if (r.status() >= 400) logs.push({ time: new Date().toISOString(), url: r.url(), status: r.status(), text: (await r.body().then(b => b.toString('utf8')).catch(() => '')).slice(0, 1500) }); });
page.on('download', async d => { await d.saveAs(`${OUT}/${d.suggestedFilename()}`); });
await context.addInitScript(() => {
  const init = () => {
    if (document.getElementById('demo-cursor')) return;
    const style = document.createElement('style');
    style.textContent = `html{scroll-behavior:smooth}#demo-cursor{position:fixed;left:0;top:0;z-index:2147483647;pointer-events:none;width:18px;height:18px;border:2px solid #ed8b20;border-radius:50%;background:#ffb84244;transform:translate(-50%,-50%);box-shadow:0 0 0 4px #ffb84222}#demo-caption{position:fixed;top:82px;left:50%;transform:translateX(-50%);z-index:2147483646;pointer-events:none;background:rgba(25,35,45,.91);color:#fff;padding:11px 25px;border-radius:30px;font:600 17px/1.4 sans-serif;box-shadow:0 4px 20px #0002;white-space:nowrap}.demo-ripple{position:fixed;border:3px solid #fa9c2a;border-radius:100%;width:22px;height:22px;pointer-events:none;z-index:2147483647;animation:demo-pop .65s ease-out forwards}@keyframes demo-pop{from{transform:translate(-50%,-50%) scale(.5);opacity:1}to{transform:translate(-50%,-50%) scale(3);opacity:0}}`;
    document.head.append(style);
    const cursor = document.createElement('div'); cursor.id = 'demo-cursor'; document.body.append(cursor);
    document.addEventListener('mousemove', e => { cursor.style.left = e.clientX + 'px'; cursor.style.top = e.clientY + 'px'; });
    document.addEventListener('mousedown', e => { const r = document.createElement('div'); r.className = 'demo-ripple'; r.style.left = e.clientX + 'px'; r.style.top = e.clientY + 'px'; document.body.append(r); setTimeout(() => r.remove(), 700); });
  };
  document.readyState === 'loading' ? document.addEventListener('DOMContentLoaded', init) : init();
});
const cdp = await context.newCDPSession(page);
let latest, encoder, ticker, frames = 0;
cdp.on('Page.screencastFrame', async event => {
  latest = Buffer.from(event.data, 'base64');
  await cdp.send('Page.screencastFrameAck', { sessionId: event.sessionId }).catch(() => {});
});
await cdp.send('Page.startScreencast', { format: 'jpeg', quality: 88, maxWidth: 1440, maxHeight: 900, everyNthFrame: 1 });
const manifest = [];
const account = { phone: process.env.DEMO_PHONE || `139000${String(Math.floor(Math.random() * 100000)).padStart(5, '0')}`, password: 'Demo12345!' };
fs.writeFileSync(`${OUT}/account.json`, JSON.stringify(account, null, 2));
const api = {
  page, context, cdp, account, OUT, sleep,
  async nav(route) { await page.goto(BASE + route, { waitUntil: 'domcontentloaded' }); await sleep(3300); },
  async caption(text) { await page.evaluate(text => { let el = document.getElementById('demo-caption'); if (!el) { el = document.createElement('div'); el.id = 'demo-caption'; document.body.append(el); } el.textContent = text; }, text); await sleep(1300); },
  async click(locator) {
    await locator.waitFor({ state: 'visible' });
    await locator.evaluate(el => el.scrollIntoView({ behavior: 'smooth', block: 'center' })); await sleep(650);
    const box = await locator.boundingBox();
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2, { steps: 16 });
    await sleep(180); await locator.click(); await sleep(1150);
  },
  async type(locator, text) { await api.click(locator); await locator.fill(''); await locator.pressSequentially(text, { delay: 65 }); await sleep(700); },
  async scroll(y) { await page.mouse.move(1325, 720, { steps: 10 }); await page.mouse.wheel(0, y); await sleep(1500); },
  async shot(name) { await page.screenshot({ path: `${OUT}/frames/live-${name}.png` }); },
  async audio() { await page.waitForSelector('audio[src]', { state: 'attached', timeout: 65000 }); await sleep(1200); },
  async begin(name) {
    if (encoder) throw Error('Recording already active');
    latest = Buffer.from((await cdp.send('Page.captureScreenshot', { format: 'jpeg', quality: 88 })).data, 'base64');
    frames = 0;
    encoder = spawn('timeout', ['600', 'ffmpeg', '-hide_banner', '-loglevel', 'warning', '-y', '-f', 'image2pipe', '-vcodec', 'mjpeg', '-framerate', '15', '-i', 'pipe:0', '-an', '-vf', 'scale=in_range=pc:out_range=tv,format=yuv420p', '-color_range', 'tv', '-c:v', 'libx264', '-preset', 'veryfast', '-crf', '20', '-pix_fmt', 'yuv420p', '-movflags', '+faststart', `${OUT}/clips/${name}.mp4`], { stdio: ['pipe', 'ignore', fs.openSync(`${OUT}/ffmpeg.log`, 'a')] });
    encoder.stdin.on('error', e => logs.push({ type: 'encoder', text: e.message }));
    ticker = setInterval(() => { if (latest && encoder && !encoder.stdin.writableNeedDrain) { encoder.stdin.write(latest); frames++; } }, 1000 / 15);
    api.currentName = name;
  },
  async end() {
    if (!encoder) return;
    clearInterval(ticker); const proc = encoder; encoder = null;
    proc.stdin.end(); await new Promise(r => proc.on('exit', r));
    const duration = frames / 15;
    manifest.push({ name: api.currentName, duration, start: manifest.reduce((s, c) => s + c.duration, 0) });
    fs.writeFileSync(`${OUT}/manifest.json`, JSON.stringify(manifest, null, 2));
    fs.writeFileSync(`${OUT}/browser-errors.json`, JSON.stringify(logs, null, 2));
    console.log(`CLIP ${api.currentName} ${duration.toFixed(1)}s`);
  },
  async finish() {
    await api.end();
    fs.writeFileSync(`${OUT}/concat.txt`, manifest.map(c => `file '${OUT}/clips/${c.name}.mp4'`).join('\n'));
    execFileSync('timeout', ['120', 'ffmpeg', '-hide_banner', '-loglevel', 'warning', '-y', '-f', 'concat', '-safe', '0', '-i', `${OUT}/concat.txt`, '-c', 'copy', '-movflags', '+faststart', `${OUT}/eardo-demo.mp4`]);
    await browser.close(); chrome.kill();
  }
};
const directory = path.dirname(new URL(import.meta.url).pathname);
if (process.argv.includes('--serve')) {
  http.createServer(async (req, res) => {
    let text = ''; for await (const chunk of req) text += chunk;
    try {
      const result = await new (Object.getPrototypeOf(async function(){}).constructor)('a', text)(api);
      res.end(JSON.stringify({ ok: true, result }));
    } catch (e) { res.end(JSON.stringify({ ok: false, error: e.stack })); }
  }).listen(9452, '127.0.0.1', () => console.log('CONTROLLER_READY 9452'));
} else {
  try {
    for (const file of fs.readdirSync(`${directory}/sections`).filter(f => f.endsWith('.mjs')).sort()) {
      await api.begin(file.replace('.mjs', ''));
      await (await import(pathToFileURL(`${directory}/sections/${file}`))).default(api);
      await api.end();
    }
    await api.finish();
  } catch (e) { await api.end(); console.error(e); await browser.close(); chrome.kill(); process.exitCode = 1; }
}
