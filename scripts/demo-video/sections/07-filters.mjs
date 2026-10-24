export default async function(a) {
  const p = a.page;
  await a.nav('/filters');
  await a.caption('⑥ 声音滤镜：发现、搜索、收藏与一键应用');
  await a.sleep(1500); await a.scroll(350);
  await a.type(p.getByPlaceholder('搜索滤镜（名称、作者、标签、描述）'), '耳朵演示');
  const card = p.locator('div.bg-white.rounded-xl.p-6').filter({ hasText: '耳朵演示 · 温暖讲述' }).first();
  await a.click(card.locator('button').filter({ has: p.locator('i.fa-heart') }));
  await a.click(card.locator('button').filter({ has: p.locator('i.fa-comment') }));
  await a.type(p.getByPlaceholder('添加评论...'), '已收藏，准备应用到下一段作品。');
  await a.click(p.locator('button').filter({ hasText: /^发送$/ }));
  await a.sleep(1500);
  await a.shot('07-filter-comment');
  await a.click(p.locator('div.fixed.inset-0 button').filter({ has: p.locator('i.fa-xmark') }));
  await a.shot('07-filter');
  await a.click(card.locator('button').filter({ hasText: /^使用$/ }));
  await a.sleep(3300);
  await a.caption('⑥ 应用已保存的滤镜参数，创作新的内容');
  await a.type(p.locator('textarea').first(), '好声音可以分享，创作的灵感也可以传递。欢迎体验耳朵。');
  await a.click(p.locator('button').filter({ hasText: '生成音频' }));
  await a.audio();
  await p.locator('canvas').evaluate(el => el.scrollIntoView({ behavior: 'smooth', block: 'center' }));
  await a.sleep(1800);
  const play = p.locator('button').filter({ has: p.locator('i.fa-play') }).last();
  if (await play.count()) await a.click(play);
  await a.sleep(2000);
  await a.shot('07-applied');
}
