export default async function(a) {
  const p = a.page;
  await a.nav('/');
  await a.caption('① 从灵感开始：欢迎页与场景创作');
  await a.sleep(2000);
  await a.click(p.getByRole('link', { name: '进入声音工作台' }));
  await a.sleep(3000);
  await a.caption('① 工作台：选择使用场景');
  await a.shot('01-home');
  await a.click(p.getByRole('link').filter({ hasText: '汇报 / 演讲' }).first());
  await a.sleep(3000);
  await a.caption('① 表达助手：选择受众、时长与输入方式');
  await a.click(p.getByRole('button', { name: '1分钟', exact: true }));
  await a.click(p.locator('button').filter({ hasText: '一句话想法' }));
  await a.type(p.locator('textarea').first(), '介绍耳朵平台，让每个人都能轻松创作自己的声音。');
  await a.shot('01-scene');
  await a.click(p.getByRole('link', { name: '用这段生成声音' }));
  await a.sleep(3000);
  await a.caption('① 声音气质：清晰讲解、轻快口播与情绪配音');
  await a.click(p.getByRole('button').filter({ hasText: '轻快口播' }).first());
  await a.shot('01-style');
  await a.scroll(490);
  await a.sleep(1800);
}
