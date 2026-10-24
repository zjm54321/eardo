export default async function(a) {
  const p = a.page;
  await a.nav('/help');
  await a.caption('⑧ 帮助中心：展开常见问题与参数说明');
  const faq = p.locator('button').filter({ has: p.locator('h3') });
  for (let i = 0; i < await faq.count(); i++) {
    await a.click(faq.nth(i));
    await a.sleep(800);
    if (i === 1) await a.shot('09-help');
  }
  await a.click(p.locator('a[href="/home"]').filter({ hasText: '返回首页' }));
  await a.sleep(3000);
  await a.caption('⑨ 退出登录：安全结束这次声音创作之旅');
  await a.click(p.locator('header button').first());
  await a.click(p.locator('button').filter({ hasText: /^退出登录$/ }));
  await a.sleep(3500);
  await a.caption('⑨ 已退出：登录入口恢复，演示完成');
  await p.locator('header').getByText('登录', { exact: true }).waitFor();
  await a.shot('09-logout');
  await a.sleep(2500);
}
