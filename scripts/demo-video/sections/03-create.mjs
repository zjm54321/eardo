export default async function(a) {
  const p = a.page;
  await a.nav('/generate');
  await a.caption('③ 声音生成：选声线、全屏编辑与表达助手');
  await a.type(p.locator('textarea').first(), '欢迎来到耳朵。在这里，文字可以变成温暖的声音。');
  await a.click(p.getByTitle('全屏编辑', { exact: true }));
  await a.sleep(1600);
  await a.click(p.getByTitle('退出全屏', { exact: true }));
  await a.click(p.getByTitle('表达助手', { exact: true }));
  await a.click(p.getByRole('button', { name: '1分钟', exact: true }));
  await a.click(p.getByRole('button', { name: '口播改写', exact: true }));
  await p.getByText('生成成功', { exact: true }).waitFor({ timeout: 65000 });
  await a.click(p.getByRole('button', { name: '原始文本', exact: true }));
  await a.click(p.getByRole('button', { name: '助手文本', exact: true }));
  await a.shot('03-assistant');
  await a.click(p.getByRole('button', { name: '一键填入', exact: true }));
  await a.type(p.locator('textarea').first(), '欢迎来到耳朵。选择喜欢的声线，让每一句话都充满表现力。');
  await a.click(p.locator('.voice-option').filter({ hasText: '甜茶' }));
  await a.click(p.locator('.voice-option').filter({ hasText: '龙安洋' }).first());
  await a.caption('③ 参数与 AI 语言控制：音高、语速、音量');
  await a.click(p.locator('#parameter-control button').filter({ hasText: '传统参数' }));
  const sliders = p.locator('#parameter-control input[type=range]');
  for (const [i, presses] of [[0, 8], [1, -7], [2, 10]]) {
    await a.click(sliders.nth(i));
    for (let n = 0; n < Math.abs(presses); n++) { await sliders.nth(i).press(presses > 0 ? 'ArrowRight' : 'ArrowLeft'); await a.sleep(70); }
  }
  await a.shot('03-params');
  await a.click(p.locator('#parameter-control button').filter({ hasText: '语言控制' }));
  await a.type(p.getByPlaceholder('例如：用温柔的语气，语速稍微慢一点，像在讲故事一样...'), '温暖亲切，语速舒缓，像朋友一样自然地讲述。');
  await a.click(p.getByTitle('AI 优化指令', { exact: true }));
  await p.getByTitle('AI 优化指令', { exact: true }).waitFor({ state: 'visible' });
  await p.waitForFunction(() => !document.querySelector('button[title="AI 优化指令"]').disabled, undefined, { timeout: 65000 });
  await a.sleep(1600);
  await a.shot('03-instruction');
}
