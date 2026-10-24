export default async function(a) {
  const p = a.page;
  await a.nav('/setup');
  await a.caption('④ 渐进式配置：四步完成一次声音创作');
  await a.click(p.getByRole('heading', { name: '龙安洋', exact: true }));
  await a.click(p.locator('button').filter({ hasText: '下一步' }));
  await a.sleep(1000);
  const traditional = p.locator('button').filter({ hasText: '传统参数' });
  if (await traditional.count()) await a.click(traditional);
  const slider = p.locator('input[type=range]').nth(1);
  await a.click(slider); await slider.press('ArrowLeft');
  await a.shot('05-setup-params');
  await a.click(p.locator('button').filter({ hasText: '下一步' }));
  await a.type(p.locator('textarea').first(), '你好，这里是耳朵。四个简单步骤，就能让创意被听见。');
  await a.shot('05-setup-text');
  await a.click(p.locator('button').filter({ hasText: '生成声音' }));
  await a.audio();
  await a.caption('④ 生成结果：配置回顾、试听与分享入口');
  const play = p.locator('button').filter({ has: p.locator('i.fa-play') }).last();
  if (await play.count()) await a.click(play);
  await a.sleep(2500);
  await a.shot('05-setup-result');
  await a.sleep(1600);
}
