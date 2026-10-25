// Deterministic, original synthesized music + the exact recorded website audio.
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
const OUT=process.env.DEMO_OUT||'/tmp/opencode/demo-video';
const read=name=>JSON.parse(fs.readFileSync(`${OUT}/${name}`,'utf8'));
const write=(name,value)=>fs.writeFileSync(`${OUT}/${name}`,JSON.stringify(value,null,2));
const run=(args,seconds=120)=>{
  const r=spawnSync('timeout',[String(seconds),'ffmpeg','-hide_banner','-y',...args],{encoding:'utf8',maxBuffer:16*1024*1024});
  if(r.status!==0)throw Error(`ffmpeg failed (${r.status}): ${r.stderr}`);
  return r.stderr;
};
const stats=text=>{const m=text.match(/\{\s*"input_i"[\s\S]*?\}/);if(!m)throw Error(`Missing loudnorm stats: ${text}`);return JSON.parse(m[0]);};
const measurements={};
function normalize(input,output,target,label){
  const a=stats(run(['-i',input,'-af',`loudnorm=I=${target}:TP=-1.5:LRA=11:print_format=json`,'-f','null','-']));
  if(!Number.isFinite(Number(a.input_i)))throw Error(`No audible audio in ${input}`);
  const filter=`loudnorm=I=${target}:TP=-1.5:LRA=11:measured_I=${a.input_i}:measured_TP=${a.input_tp}:measured_LRA=${a.input_lra}:measured_thresh=${a.input_thresh}:offset=${a.target_offset}:linear=true:print_format=json,aresample=48000`;
  const b=stats(run(['-i',input,'-af',filter,'-ac','2','-c:a','pcm_f32le',output]));
  measurements[label]={source:a,normalization:b,target};
}
const manifest=read('manifest.json'), duration=manifest.reduce((s,c)=>s+c.duration,0);
const events=read('media-events.json').sort((a,b)=>a.videoTime-b.videoTime);
const assets=read('audio-assets.json');
const active=new Map(), contextStates=new Map(), intervals=[];
const stops=new Set(['pause','ended','waiting','seeking','emptied','pagehide','context-suspended','context-closed','segment-end']);
function close(id,event){
  const start=active.get(id);if(!start)return;
  active.delete(id);
  const end=Math.min(duration,event.videoTime);
  if(end-start.videoTime>.025)intervals.push({id,src:start.src,start:start.videoTime,end,
    mediaOffset:start.mediaTime||0,rate:start.rate||1,volume:start.muted?0:(start.volume??1),
    mediaEnd:event.kind!=='seeking'&&Number.isFinite(event.mediaTime)?event.mediaTime:null,
    section:start.section,startEvent:start.kind,stopEvent:event.kind,
    button:events.find(e=>e.id===id&&e.kind==='visual-playing'&&e.videoTime>=start.videoTime&&e.videoTime<end)?.button||start.button});
}
for(const e of events){
  if(!e.id)continue;
  if(e.kind==='context-link'){
    contextStates.set(e.id,e.contextState);
    if(e.contextState!=='running')close(e.id,e);
    continue;
  }
  if(e.kind.startsWith('context-'))contextStates.set(e.id,e.kind.slice('context-'.length));
  if(stops.has(e.kind)){close(e.id,e);continue;}
  if(e.kind==='ratechange'||e.kind==='volumechange')close(e.id,e);
  const starts=e.kind==='playing'||e.kind==='context-running'||e.kind==='segment-start'||e.kind==='seeked'||e.kind==='ratechange'||e.kind==='volumechange';
  if(starts&&!e.paused&&!e.ended&&e.src&&!active.has(e.id)&&(!contextStates.has(e.id)||contextStates.get(e.id)==='running'))active.set(e.id,e);
}
for(const id of active.keys())close(id,{videoTime:duration,kind:'video-end'});
intervals.sort((a,b)=>a.start-b.start);
if(!intervals.length)throw Error('No website audio playback was recorded');
write('playback-timeline.json',intervals);
fs.mkdirSync(`${OUT}/audio/normalized`,{recursive:true});
const normalized=new Map();
for(const interval of intervals){
  if(normalized.has(interval.src))continue;
  const asset=assets.find(a=>a.src===interval.src);if(!asset)throw Error(`Missing source: ${interval.src}`);
  const output=`${OUT}/audio/normalized/${path.basename(asset.file)}.wav`;
  normalize(`${OUT}/${asset.file}`,output,-16,asset.file);normalized.set(interval.src,output);
}
// Place every playback, including autoplay, at its actual encoded-frame clock time.
const filters=[], sitePCM=Buffer.alloc(Math.ceil(duration*48000)*8);
intervals.forEach((clip,i)=>{
  const length=clip.end-clip.start;
  const span=clip.mediaEnd!==null&&clip.mediaEnd>clip.mediaOffset?clip.mediaEnd-clip.mediaOffset:length*clip.rate;
  // Match the measured media-clock span to the encoded-frame span, including
  // small scheduler/encoder clock differences, without duplicating samples at resume.
  const speed=span/length,tempo=Math.abs(speed-1)<.00001?'':`,atempo=${speed}`;
  clip.sourceSpan=span;clip.effectiveTempo=speed;
  const filter=`atrim=start=${clip.mediaOffset}:duration=${span},asetpts=PTS-STARTPTS${tempo},atrim=duration=${length},apad=whole_dur=${length},volume=${clip.volume}`;
  const decoded=spawnSync('timeout',['60','ffmpeg','-hide_banner','-loglevel','error','-i',normalized.get(clip.src),'-af',filter,'-ar','48000','-ac','2','-f','f32le','pipe:1'],{maxBuffer:128*1024*1024});
  if(decoded.status!==0)throw Error(`Clip decode failed: ${decoded.stderr}`);
  const offset=Math.round(clip.start*48000)*8;
  for(let p=0;p<decoded.stdout.length&&offset+p<sitePCM.length;p+=4)sitePCM.writeFloatLE(sitePCM.readFloatLE(offset+p)+decoded.stdout.readFloatLE(p),offset+p);
  filters.push(`playback ${i}: ${filter}; place at stereo sample ${Math.round(clip.start*48000)} (equivalent to sample-accurate adelay)`);
});
fs.writeFileSync(`${OUT}/audio/site-filter.txt`,filters.join(';\n'));
write('playback-timeline.json',intervals);
// Place decoded samples directly; avoids FFmpeg 9's early EOF with multiple
// long-leading-silence adelay inputs feeding amix.
const siteHeader=Buffer.alloc(44);siteHeader.write('RIFF',0);siteHeader.writeUInt32LE(36+sitePCM.length,4);siteHeader.write('WAVEfmt ',8);
siteHeader.writeUInt32LE(16,16);siteHeader.writeUInt16LE(3,20);siteHeader.writeUInt16LE(2,22);siteHeader.writeUInt32LE(48000,24);
siteHeader.writeUInt32LE(48000*8,28);siteHeader.writeUInt16LE(8,32);siteHeader.writeUInt16LE(32,34);siteHeader.write('data',36);siteHeader.writeUInt32LE(sitePCM.length,40);
const siteFile=fs.openSync(`${OUT}/audio/site-raw.wav`,'w');fs.writeSync(siteFile,siteHeader);fs.writeSync(siteFile,sitePCM);fs.closeSync(siteFile);
normalize(`${OUT}/audio/site-raw.wav`,`${OUT}/audio/site.wav`,-16,'website-stem');

// Original composition, 80 BPM, C–G–Am–F (I–V–vi–IV), warm sine pad + soft arpeggio.
// Entirely synthesized here: no recordings, samples, third-party tracks or attribution obligations.
const sr=48000, count=Math.ceil(duration*sr), file=fs.openSync(`${OUT}/audio/music-original.wav`,'w');
const header=Buffer.alloc(44);header.write('RIFF',0);header.writeUInt32LE(36+count*8,4);header.write('WAVEfmt ',8);
header.writeUInt32LE(16,16);header.writeUInt16LE(3,20);header.writeUInt16LE(2,22);header.writeUInt32LE(sr,24);
header.writeUInt32LE(sr*8,28);header.writeUInt16LE(8,32);header.writeUInt16LE(32,34);header.write('data',36);header.writeUInt32LE(count*8,40);fs.writeSync(file,header);
const chords=[[48,52,55],[43,47,50],[45,48,52],[41,45,48]], frequency=m=>440*2**((m-69)/12);
const frequencies=chords.map(c=>c.map(frequency));
for(let base=0;base<count;base+=sr){
  const size=Math.min(sr,count-base), buffer=Buffer.allocUnsafe(size*8);
  for(let j=0;j<size;j++){
    const t=(base+j)/sr, chord=Math.floor(t/6)%4, local=t%6;
    const blend=Math.min(1,local/.55), previous=(chord+3)%4;
    let left=0,right=0;
    for(let n=0;n<3;n++){
      for(const [index,gain] of [[previous,1-blend],[chord,blend]]){
        const f=frequencies[index][n], phase=2*Math.PI*f*t;
        left+=gain*(Math.sin(phase)*.045+Math.sin(phase*2+.18)*.009);
        right+=gain*(Math.sin(phase+.08)*.045+Math.sin(phase*2+.26)*.009);
      }
    }
    const age=t%.75,note=[0,1,2,1][Math.floor(t/.75)%4], f=frequencies[chord][note]*2;
    const arp=(1-Math.exp(-age/.035))*Math.exp(-age/.28)*.018;
    left+=Math.sin(2*Math.PI*f*t)*arp;right+=Math.sin(2*Math.PI*f*t+.15)*arp;
    buffer.writeFloatLE(left,j*8);buffer.writeFloatLE(right,j*8+4);
  }
  fs.writeSync(file,buffer);
}
fs.closeSync(file);
normalize(`${OUT}/audio/music-original.wav`,`${OUT}/audio/music-bed.wav`,-28,'music-bed');
const windows=[];
for(const clip of intervals.filter(c=>c.volume>0)){
  const last=windows.at(-1);
  if(last&&clip.start-last.end<.7)last.end=Math.max(last.end,clip.end);
  else windows.push({start:clip.start,end:clip.end});
}
const duck=windows.map(w=>`(1-0.82*max(0,min(1,min((t-${Math.max(0,w.start-.1)})/0.1,(${w.end+.6}-t)/0.6))))`).join('*')||'1';
const mix=`[0:a]volume='${duck}':eval=frame,afade=t=in:st=0:d=2.5,afade=t=out:st=${Math.max(0,duration-4)}:d=4[bg];[bg]asplit=2[bgout][bgmix];[1:a][bgmix]amix=inputs=2:normalize=0:duration=longest,atrim=duration=${duration}[mix]`;
fs.writeFileSync(`${OUT}/audio/mix-filter.txt`,mix);
run(['-i',`${OUT}/audio/music-bed.wav`,'-i',`${OUT}/audio/site.wav`,'-filter_complex',mix,
  '-map','[bgout]','-c:a','pcm_f32le',`${OUT}/audio/music-ducked.wav`,
  '-map','[mix]','-ar','48000','-ac','2','-c:a','pcm_f32le',`${OUT}/audio/soundtrack.wav`]);
run(['-i',`${OUT}/eardo-demo-picture.mp4`,'-i',`${OUT}/audio/soundtrack.wav`,'-map','0:v:0','-map','1:a:0',
  '-c:v','copy','-c:a','aac','-b:a','192k','-ar','48000','-ac','2','-t',String(duration),'-movflags','+faststart',`${OUT}/eardo-demo.mp4`]);
measurements.finalMix=stats(run(['-i',`${OUT}/eardo-demo.mp4`,'-af','loudnorm=I=-16:TP=-1.5:LRA=11:print_format=json','-f','null','-']));
write('loudness.json',measurements);
write('music-source.json',{source:'Original procedural synthesis in scripts/demo-video/mix.mjs',license:'Original generated audio; no external audio, samples or licensed recordings used.',bpm:80,progression:'C–G–Am–F (I–V–vi–IV)',targetBedLUFS:-28,duckAmplitude:.18,duckAttackSeconds:.1,duckReleaseSeconds:.6,fadeInSeconds:2.5,fadeOutSeconds:4});
console.log(JSON.stringify({duration,playbacks:intervals.length,websiteLUFS:measurements['website-stem'].normalization.output_i,musicLUFS:measurements['music-bed'].normalization.output_i,finalLUFS:measurements.finalMix.input_i},null,2));
