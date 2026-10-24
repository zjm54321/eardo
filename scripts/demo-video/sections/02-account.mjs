export default async function(a) {
  const p = a.page;
  await a.nav('/register');
  await a.caption('② 创建账号：手机号注册并登录');
  await a.type(p.getByPlaceholder('请输入手机号', { exact: true }), a.account.phone);
  await a.type(p.getByPlaceholder('设置密码', { exact: true }), a.account.password);
  await a.type(p.getByPlaceholder('请再次输入密码', { exact: true }), a.account.password);
  await a.shot('02-register');
  await a.click(p.getByRole('button', { name: '注册账号', exact: true }));
  await p.waitForURL('**/login', { timeout: 20000 }); await a.sleep(3000);
  await a.type(p.locator('input').first(), a.account.phone);
  await a.type(p.locator('input[type=password]'), a.account.password);
  await a.click(p.getByRole('button', { name: '立即登录', exact: true }));
  await a.sleep(3500);
  await a.caption('② 登录成功：右上角出现个人头像');
  await a.shot('02-login');
  await a.sleep(1800);
}
