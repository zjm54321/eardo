// Staging-only real browser recording. See README.md.
import fs from 'node:fs';
import path from 'node:path';
import http from 'node:http';
import { spawn, execFileSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';
import { createHash } from 'node:crypto';
const OUT = process.env.DEMO_OUT || '/tmp/opencode/demo-video';
const BASE = 'https://dev.eardoai.com';
const FPS = 15;
const resume = process.argv.find(arg=>arg.startsWith('--resume='))?.slice('--resume='.length);
const load = (name,fallback) => { try { return JSON.parse(fs.readFileSync(`${OUT}/${name}`,'utf8')); } catch { return fallback; } };
const previous = resume ? load('manifest.json',[]).filter(section=>section.name<resume) : [];
const resumeTime = previous.reduce((sum,section)=>sum+section.duration,0);
const { chromium } = await import(pathToFileURL(`${OUT}/node_modules/playwright-core/index.mjs`));
fs.mkdirSync(`${OUT}/clips`, { recursive: true });
fs.mkdirSync(`${OUT}/frames`, { recursive: true });
fs.mkdirSync(`${OUT}/audio`, { recursive: true });
const sleep = ms => new Promise(r => setTimeout(r, ms));
// Do not launch a browser until a cache-busted deployment is live.
for (let i = 0; ; i++) {
  // curl respects the WSL/NixOS HTTPS proxy variables (Node 22 fetch does not).
  let html = '';
  try { html = execFileSync('timeout', ['25','curl','--max-time','20','--fail','-sS',`${BASE}/generate`], {encoding:'utf8'}); } catch {}
  const release = html.match(/\/pkg\/eardo\.[A-Za-z0-9_-]*\.js/);
  if (release) { fs.writeFileSync(`${OUT}/release.txt`, release[0]); break; }
  if (i === 39) throw Error('预发布部署等待超过20分钟');
  await sleep(30000);
}
const profileDir = process.env.DEMO_PROFILE || `${OUT}/profile-${Date.now()}`;
if(resume&&!process.env.DEMO_PROFILE)throw Error('--resume requires DEMO_PROFILE from record-state.json');
fs.writeFileSync(`${OUT}/record-state.json`,JSON.stringify({profileDir,resume:resume||null},null,2));
const chrome = spawn('timeout', ['1800', 'google-chrome', '--headless=new', '--no-sandbox',
  '--disable-gpu', '--hide-scrollbars', '--autoplay-policy=no-user-gesture-required',
  '--remote-debugging-port=9451', '--window-size=1440,900',
  `--user-data-dir=${profileDir}`, 'about:blank'], { stdio: 'ignore' });
let browser;
for (let i = 0; i < 60; i++) {
  try { browser = await chromium.connectOverCDP('http://127.0.0.1:9451'); break; } catch { await sleep(250); }
}
if (!browser) throw Error('Chrome did not start');
const context = browser.contexts()[0];
await context.grantPermissions(['clipboard-read', 'clipboard-write'], { origin: BASE });
const page = context.pages()[0];
await page.setViewportSize({ width: 1440, height: 900 });
page.setDefaultTimeout(45000);
page.setDefaultNavigationTimeout(45000);
const logs = resume ? load('browser-errors.json',[]) : [];
page.on('console', m => { if (m.type() === 'error') logs.push({ time: new Date().toISOString(), url: page.url(), type: 'console', text: m.text() }); });
page.on('pageerror', e => logs.push({ time: new Date().toISOString(), url: page.url(), type: 'exception', text: e.message }));
page.on('response', async r => { if (r.status() >= 400) logs.push({ time: new Date().toISOString(), url: r.url(), status: r.status(), text: (await r.body().then(b => b.toString('utf8')).catch(() => '')).slice(0, 1500) }); });
page.on('download', async d => { await d.saveAs(`${OUT}/${d.suggestedFilename()}`); });
const mediaEvents = resume ? load('media-events.json',[]).filter(e=>e.videoTime<resumeTime) : [];
const audioAssets = new Map((resume ? load('audio-assets.json',[]) : []).map(a=>[a.src,a]));
const assetTasks = [], bookmarks = resume ? load('playback-bookmarks.json',[]).filter(b=>b.time<resumeTime) : [];
const frameClock = resume ? load('frame-clock.json',[]).filter(f=>f.videoTime<resumeTime) : [];
let segmentStart = 0;
await page.exposeBinding('__demoMediaEvent', async (_source, event) => {
  if (!encoder) return;
  const videoTime = segmentStart + Math.max(0, frames / FPS - (Date.now() - event.wallMs) / 1000);
  mediaEvents.push({ ...event, videoTime, section: api.currentName, frame: frames });
  if (event.src && !audioAssets.has(event.src)) {
    const url = new URL(event.src, BASE);
    if (url.origin !== BASE) throw Error(`Audio source outside staging: ${url}`);
    const file = `audio/${createHash('sha256').update(url.href).digest('hex').slice(0, 16)}.audio`;
    const asset = { src: url.href, file }; audioAssets.set(event.src, asset);
    assetTasks.push((async () => {
      const cookies = (await context.cookies(url.href)).map(c=>`${c.name}=${c.value}`).join('; ');
      asset.contentType = await new Promise((resolve,reject)=>{
        const child=spawn('timeout',['60','curl','--fail','--max-time','45','--silent','--show-error','-H',`Cookie: ${cookies}`,
          '--output',`${OUT}/${file}`,'--write-out','%{content_type}',url.href]);
        let output='',error='';child.stdout.on('data',b=>output+=b);child.stderr.on('data',b=>error+=b);
        child.on('exit',code=>code===0?resolve(output):reject(Error(`Audio download ${code}: ${error}`)));
      });
    })().catch(error=>{asset.error=error.message;}));
  }
});
await context.addInitScript(() => {
  window.__demoWasmReady = false;
  for (const method of ['instantiate','instantiateStreaming']) {
    const original = WebAssembly[method];
    if (original) WebAssembly[method] = function(...args) {
      return original.apply(this,args).then(result=>{window.__demoWasmReady=true;return result;});
    };
  }
  let next = 0;
  const ids = new WeakMap(), media = new Set(), links = new Map();
  const id = element => { if (!ids.has(element)) ids.set(element, `${performance.timeOrigin}-${++next}`); return ids.get(element); };
  const snapshot = element => {
    let parent = element.parentElement, button;
    while (parent && !button) { button = parent.querySelector('button:has(i.fa-play),button:has(i.fa-pause)'); if (!button) parent = parent.parentElement; }
    const r = button?.getBoundingClientRect();
    return { id: id(element), src: element.currentSrc || element.src, mediaTime: element.currentTime,
      rate: element.playbackRate, volume: element.volume, muted: element.muted, paused: element.paused,
      ended: element.ended, duration: Number.isFinite(element.duration) ? element.duration : null,
      button: r ? { x:r.x, y:r.y, width:r.width, height:r.height, state:button.querySelector('i.fa-pause')?'pause':'play' } : null };
  };
  const emit = (kind, data = {}) => window.__demoMediaEvent({ kind, wallMs: Date.now(), url: location.href, ...data }).catch(console.error);
  for (const kind of ['play','playing','pause','ended','seeking','seeked','ratechange','volumechange','waiting','emptied']) {
    document.addEventListener(kind, event => {
      if (!(event.target instanceof HTMLMediaElement)) return;
      media.add(event.target); emit(kind, snapshot(event.target));
      if (kind === 'playing') requestAnimationFrame(() => requestAnimationFrame(() => emit('visual-playing', snapshot(event.target))));
    }, true);
  }
  addEventListener('pagehide', () => { for (const element of media) emit('pagehide', snapshot(element)); });
  window.__demoCaptureMedia = kind => Promise.all([...media].map(element => emit(kind, snapshot(element))));
  const original = AudioContext.prototype.createMediaElementSource;
  AudioContext.prototype.createMediaElementSource = function(element) {
    const source = original.call(this, element); media.add(element);
    if (!links.has(this)) {
      links.set(this, new Set());
      this.addEventListener('statechange', () => { for (const item of links.get(this)) emit(`context-${this.state}`, snapshot(item)); });
    }
    links.get(this).add(element); emit('context-link', { ...snapshot(element), contextState:this.state }); return source;
  };
});
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
// Block production at the network layer without disabling staging's normal browser cache.
await cdp.send('Network.enable');
await cdp.send('Network.setBlockedURLs',{urls:['*://eardoai.com/*','*://www.eardoai.com/*']});
let latest, latestTimestamp, encoder, ticker, frames = 0;
cdp.on('Page.screencastFrame', async event => {
  latest = Buffer.from(event.data, 'base64');
  latestTimestamp = event.metadata.timestamp;
  await cdp.send('Page.screencastFrameAck', { sessionId: event.sessionId }).catch(() => {});
});
await cdp.send('Page.startScreencast', { format: 'jpeg', quality: 88, maxWidth: 1440, maxHeight: 900, everyNthFrame: 1 });
const manifest = previous;
const account = resume ? load('account.json',null) : { phone: process.env.DEMO_PHONE || `139000${String(Math.floor(Math.random() * 100000)).padStart(5, '0')}`, password: 'Demo12345!' };
if(!account)throw Error('Missing account.json for resume');
account.workTitle = `耳朵演示 · 第一段声音（${account.phone.slice(-4)}）`;
account.filterTitle = `耳朵演示 · 温暖讲述（${account.phone.slice(-4)}）`;
fs.writeFileSync(`${OUT}/account.json`, JSON.stringify(account, null, 2));
const api = {
  page, context, cdp, account, OUT, sleep,
  async nav(route) {
    const link=page.locator(`header a[href="${route}"]`).first();
    if(page.url().startsWith(BASE)&&await link.isVisible()) {
      const box=await link.boundingBox();await page.mouse.move(box.x+box.width/2,box.y+box.height/2,{steps:10});
      await link.click();await page.waitForURL(BASE+route);
    } else await page.goto(BASE + route, { waitUntil: 'domcontentloaded' });
    await page.waitForFunction(()=>window.__demoWasmReady===true,undefined,{timeout:60000});await sleep(3300);
  },
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
  async playbackDemo(label, scope = page) {
    const pause = scope.locator('button').filter({ has: page.locator('i.fa-pause') }).last();
    if (await pause.count()) { await pause.click(); await sleep(850); }
    const play = scope.locator('button').filter({ has: page.locator('i.fa-play') }).last();
    await play.waitFor();
    bookmarks.push({ label, phase:'before-click', time:segmentStart + frames/FPS });
    await api.click(play);
    bookmarks.push({ label, phase:'after-click', time:segmentStart + frames/FPS });
    await sleep(2200); await api.shot(`playing-${label}`); await sleep(1000);
    // Let short clips reach their natural end: a slow move towards a disappearing
    // pause icon can otherwise race ended and wait for a button that no longer exists.
    await page.waitForFunction(()=>[...document.querySelectorAll('audio,video')].every(el=>el.paused||el.ended),undefined,{timeout:45000});
  },
  async begin(name) {
    if (encoder) throw Error('Recording already active');
    latest = Buffer.from((await cdp.send('Page.captureScreenshot', { format: 'jpeg', quality: 88 })).data, 'base64');
    frames = 0;
    segmentStart = manifest.reduce((sum, section) => sum + section.duration, 0);
    encoder = spawn('timeout', ['600', 'ffmpeg', '-hide_banner', '-loglevel', 'warning', '-y', '-f', 'image2pipe', '-vcodec', 'mjpeg', '-framerate', '15', '-i', 'pipe:0', '-an', '-vf', 'scale=in_range=pc:out_range=tv,format=yuv420p', '-color_range', 'tv', '-c:v', 'libx264', '-preset', 'veryfast', '-crf', '20', '-pix_fmt', 'yuv420p', '-movflags', '+faststart', `${OUT}/clips/${name}.mp4`], { stdio: ['pipe', 'ignore', fs.openSync(`${OUT}/ffmpeg.log`, 'a')] });
    encoder.stdin.on('error', e => logs.push({ type: 'encoder', text: e.message }));
    const clockStart = performance.now();
    ticker = setInterval(() => {
      const due = Math.floor((performance.now()-clockStart)*FPS/1000);
      while (frames < due && latest && encoder && !encoder.stdin.writableNeedDrain) {
        frameClock.push({ frame:Math.round(segmentStart*FPS)+frames, videoTime:segmentStart+frames/FPS, sourceTimestamp:latestTimestamp });
        encoder.stdin.write(latest); frames++;
      }
    }, 10);
    api.currentName = name;
    await page.evaluate(() => window.__demoCaptureMedia?.('segment-start')).catch(()=>{});
  },
  async end() {
    if (!encoder) return;
    await page.evaluate(() => window.__demoCaptureMedia?.('segment-end')).catch(()=>{});
    clearInterval(ticker); const proc = encoder; encoder = null;
    proc.stdin.end(); await new Promise(r => proc.on('exit', r));
    const duration = frames / 15;
    manifest.push({ name: api.currentName, duration, start: manifest.reduce((s, c) => s + c.duration, 0) });
    fs.writeFileSync(`${OUT}/manifest.json`, JSON.stringify(manifest, null, 2));
    fs.writeFileSync(`${OUT}/browser-errors.json`, JSON.stringify(logs, null, 2));
    fs.writeFileSync(`${OUT}/media-events.json`, JSON.stringify(mediaEvents, null, 2));
    fs.writeFileSync(`${OUT}/audio-assets.json`, JSON.stringify([...audioAssets.values()], null, 2));
    fs.writeFileSync(`${OUT}/playback-bookmarks.json`, JSON.stringify(bookmarks, null, 2));
    fs.writeFileSync(`${OUT}/frame-clock.json`, JSON.stringify(frameClock, null, 2));
    console.log(`CLIP ${api.currentName} ${duration.toFixed(1)}s`);
  },
  async finish() {
    await api.end();
    fs.writeFileSync(`${OUT}/concat.txt`, manifest.map(c => `file '${OUT}/clips/${c.name}.mp4'`).join('\n'));
    execFileSync('timeout', ['120', 'ffmpeg', '-hide_banner', '-loglevel', 'warning', '-y', '-f', 'concat', '-safe', '0', '-i', `${OUT}/concat.txt`, '-c', 'copy', '-movflags', '+faststart', `${OUT}/eardo-demo-picture.mp4`]);
    await Promise.all(assetTasks);
    const failed=[...audioAssets.values()].filter(a=>a.error);if(failed.length)throw Error(JSON.stringify(failed));
    await browser.close(); chrome.kill();
    fs.writeFileSync(`${OUT}/audio-assets.json`, JSON.stringify([...audioAssets.values()], null, 2));
    execFileSync('timeout', ['600', process.execPath, `${directory}/mix.mjs`], { stdio:'inherit', env:{...process.env,DEMO_OUT:OUT} });
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
    for (const file of fs.readdirSync(`${directory}/sections`).filter(f => f.endsWith('.mjs')&&(!resume||f.replace('.mjs','')>=resume)).sort()) {
      await api.begin(file.replace('.mjs', ''));
      await (await import(pathToFileURL(`${directory}/sections/${file}`))).default(api);
      await api.end();
    }
    await api.finish();
  } catch (e) { await api.end(); console.error(e); await browser.close(); chrome.kill(); process.exitCode = 1; }
}
