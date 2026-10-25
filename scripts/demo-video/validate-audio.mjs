import fs from 'node:fs';
import { spawnSync } from 'node:child_process';
const OUT=process.env.DEMO_OUT||'/tmp/opencode/demo-video', V=`${OUT}/validation`;
fs.mkdirSync(V,{recursive:true});
const read=n=>JSON.parse(fs.readFileSync(`${OUT}/${n}`,'utf8'));
const tool=(name,args,binary=false)=>{
  const r=spawnSync('timeout',['90',name,...args],{encoding:binary?undefined:'utf8',maxBuffer:64*1024*1024});
  if(r.status!==0)throw Error(`${name}: ${r.stderr}`);
  return binary?r.stdout:{out:r.stdout,log:r.stderr};
};
const ff=args=>tool('ffmpeg',['-hide_banner','-y',...args]);
const probe=JSON.parse(tool('ffprobe',['-v','error','-show_streams','-show_format','-of','json',`${OUT}/eardo-demo.mp4`]).out);
const video=probe.streams.find(s=>s.codec_type==='video'),audio=probe.streams.find(s=>s.codec_type==='audio');
if(!video||!audio||video.codec_name!=='h264'||audio.codec_name!=='aac'||audio.sample_rate!=='48000'||audio.channels!==2)throw Error('Incorrect streams');
if(Math.abs(Number(video.duration)-Number(audio.duration))>.05)throw Error('Audio/video duration mismatch');
fs.writeFileSync(`${V}/ffprobe.json`,JSON.stringify(probe,null,2));
const measurements={};
for(const [name,file] of [['website','audio/site.wav'],['music-bed','audio/music-bed.wav'],['mix','eardo-demo.mp4']]){
  const {log}=ff(['-i',`${OUT}/${file}`,'-af','ebur128=peak=true','-f','null','-']);
  fs.writeFileSync(`${V}/${name}-ebur128.log`,log);
  const summary=log.slice(log.lastIndexOf('Summary:'));
  measurements[name]={summary,integrated:Number(summary.match(/I:\s*(-?[\d.]+)/)?.[1])};
}
if(Math.abs(measurements.website.integrated+16)>1||Math.abs(measurements['music-bed'].integrated+28)>1)throw Error('Loudness targets missed');
const events=read('media-events.json'), timeline=read('playback-timeline.json'), bookmarks=read('playback-bookmarks.json'), clock=read('frame-clock.json');
const cases=[];
const volume=(file,start,duration)=>{
  const {log}=ff(['-ss',String(start),'-t',String(duration),'-i',`${OUT}/${file}`,'-af','volumedetect','-f','null','-']);
  return {mean:Number(log.match(/mean_volume:\s*(-?[\d.]+)/)?.[1]),max:Number(log.match(/max_volume:\s*(-?[\d.]+)/)?.[1])};
};
for(const label of ['generate','setup','filter-applied']){
  const before=bookmarks.find(b=>b.label===label&&b.phase==='before-click'),after=bookmarks.find(b=>b.label===label&&b.phase==='after-click');
  const clip=timeline.find(c=>c.start>=before.time&&c.start<=after.time+.1);
  if(!clip?.button)throw Error(`Missing playback/button for ${label}`);
  const start=Math.max(0,clip.start-.7), windowStart=clip.start+.4, windowDuration=Math.min(2,clip.end-windowStart-.1);
  ff(['-ss',String(start),'-i',`${OUT}/eardo-demo.mp4`,'-t','5','-c:v','libx264','-preset','veryfast','-crf','22','-c:a','aac','-b:a','192k',`${V}/playback-${label}-5s.mp4`]);
  const result={label,start:clip.start,end:clip.end,sampleStart:start,
    finalClip:volume('eardo-demo.mp4',windowStart,windowDuration),
    speech:volume('audio/site.wav',windowStart,windowDuration),
    musicUnducked:volume('audio/music-bed.wav',windowStart,windowDuration),
    musicDucked:volume('audio/music-ducked.wav',windowStart,windowDuration)};
  result.duckDb=result.musicDucked.mean-result.musicUnducked.mean;
  if(!(result.speech.mean>-40&&result.duckDb<-13&&result.duckDb>-17))throw Error(`Audio/ducking missing: ${JSON.stringify(result)}`);
  const center=Math.floor(clip.start*15), frames=[];
  const box=clip.button, x=Math.max(0,Math.round(box.x+box.width/2-9)),y=Math.max(0,Math.round(box.y+box.height/2-9));
  for(let index=center-3;index<=center+5;index++){
    const time=index/15;
    ff(['-ss',String(time),'-i',`${OUT}/eardo-demo.mp4`,'-frames:v','1','-vf',`crop=96:96:${Math.max(0,Math.round(box.x+box.width/2-48))}:${Math.max(0,Math.round(box.y+box.height/2-48))},scale=192:192`,`${V}/sync-${label}-${index}.png`]);
    const pixels=tool('ffmpeg',['-v','error','-ss',String(time),'-i',`${OUT}/eardo-demo.mp4`,'-frames:v','1','-vf',`crop=18:18:${x}:${y}`,'-pix_fmt','rgb24','-f','rawvideo','pipe:1'],true);
    const mask=[];for(let i=0;i<pixels.length;i+=3)mask.push(pixels[i]>220&&pixels[i+1]>220&&pixels[i+2]>190?1:0);
    frames.push({index,time,mask});
  }
  const distance=(a,b)=>a.reduce((s,value,i)=>s+(value!==b[i]?1:0),0);
  const play=frames[0].mask,pause=frames.at(-1).mask;
  if(distance(play,pause)<4)throw Error(`No visual play→pause transition: ${label}`);
  const visual=frames.find(f=>distance(f.mask,pause)<distance(f.mask,play));
  // Detect actual waveform onset, not merely the intended filter delay.
  const raw=tool('ffmpeg',['-v','error','-ss',String(clip.start-.2),'-t','1','-i',`${OUT}/audio/site.wav`,'-ac','1','-ar','48000','-f','f32le','pipe:1'],true);
  let offset=0;
  for(let base=0;base<raw.length/4;base+=240){let energy=0,n=0;for(let j=base;j<Math.min(base+240,raw.length/4);j++){const v=raw.readFloatLE(j*4);energy+=v*v;n++;}if(Math.sqrt(energy/n)>10**(-48/20)){offset=base/48000;break;}}
  const audibleOnset=clip.start-.2+offset;
  result.sync={visualPauseFrame:visual.index,visualTime:visual.time,placedAudioStart:clip.start,audibleOnset,
    deltaMs:(visual.time-audibleOnset)*1000,
    frames:frames.map(({mask,...f})=>({...f,playDistance:distance(mask,play),pauseDistance:distance(mask,pause)})),
    sourceFrame:clock.find(f=>f.frame===visual.index)};
  cases.push(result);
  // Save the evidence before asserting, so a failed sync check is diagnosable.
  fs.writeFileSync(`${V}/sync-and-volume.json`,JSON.stringify({measurements,cases},null,2));
  if(Math.abs(result.sync.deltaMs)>150)throw Error(`Sync exceeds 150ms: ${JSON.stringify(result.sync)}`);
}
const idleStart=20;
ff(['-ss',String(idleStart),'-i',`${OUT}/eardo-demo.mp4`,'-t','5','-c:v','libx264','-preset','veryfast','-crf','22','-c:a','aac','-b:a','192k',`${V}/idle-5s.mp4`]);
const idle={start:idleStart,duration:5,finalClip:volume('eardo-demo.mp4',idleStart,5),music:volume('audio/music-ducked.wav',idleStart,5),speech:volume('audio/site.wav',idleStart,5)};
if(!(idle.music.mean>-40&&idle.speech.max<-80))throw Error('Idle music missing or stray site audio');
fs.writeFileSync(`${V}/sync-and-volume.json`,JSON.stringify({measurements,cases,idle,passed:true},null,2));
console.log(JSON.stringify({duration:probe.format.duration,measurements,cases:cases.map(c=>({label:c.label,start:c.start,duckDb:c.duckDb,syncMs:c.sync.deltaMs})),idle},null,2));
