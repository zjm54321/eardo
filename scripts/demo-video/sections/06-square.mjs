export default async function(a) {
  const p = a.page;
  await a.nav('/voice');
  await a.caption('⑤ 声音广场：浏览、搜索、播放、点赞与评论');
  const next = p.locator('button').filter({ has: p.locator('i.fa-chevron-right') }).first();
  if (await next.count()) await a.click(next);
  await a.scroll(430);
  await a.type(p.getByPlaceholder('搜索标题、作者或描述'), '耳朵演示');
  const card = p.locator('div.bg-white.rounded-xl.p-6').filter({ hasText: '耳朵演示 · 第一段声音' }).first();
  await a.click(card.locator('button').filter({ has: p.locator('i.fa-play') }));
  await a.sleep(1800);
  const heart = card.locator('button').filter({ has: p.locator('i.fa-heart') });
  await a.click(heart); await a.click(heart);
  await a.click(card.locator('button').filter({ has: p.locator('i.fa-comment') }));
  await a.type(p.getByPlaceholder('写下你的评论...'), '声音自然又温暖，第一次创作成功！');
  await a.click(p.locator('button').filter({ hasText: '发表评论' }));
  await p.getByText('声音自然又温暖，第一次创作成功！', { exact: true }).waitFor({ timeout: 20000 });
  await a.shot('06-comment');
  await a.sleep(1300);
  await a.click(p.getByRole('button', { name: '×', exact: true }));
  await a.click(p.getByRole('button', { name: '清除', exact: true }));
  await a.shot('06-square');
  await a.sleep(1200);
}
