// MiniRoblox Admin dashboard — TypeScript frontend (Google login + roles).
interface Player {
  name: string;
  coins: number;
  online: boolean;
  x: number;
  y: number;
  z: number;
}
interface Status {
  online_count: number;
  world_blocks: number;
  total_players: number;
  total_coins: number;
  players: Player[];
}
interface Me {
  email: string;
  role: string;
}

type Tag = keyof HTMLElementTagNameMap;
function el<K extends Tag>(tag: K, cls?: string, text?: string): HTMLElementTagNameMap[K] {
  const n = document.createElement(tag);
  if (cls) n.className = cls;
  if (text !== undefined) n.textContent = text;
  return n;
}
const fmt = (n: number): string => n.toLocaleString('en-US');

let me: Me | null = null;
const canCoins = () => !!me && ['admin', 'manager', 'sales'].includes(me.role);
const canDbOps = () => !!me && ['admin', 'manager'].includes(me.role);
const canRestore = () => !!me && me.role === 'admin';
const canRoles = () => !!me && me.role === 'admin';

function setMsg(text: string, ok?: boolean): void {
  const m = document.getElementById('msg');
  if (!m) return;
  m.textContent = text;
  m.className = 'msg ' + (ok === undefined ? '' : ok ? 'ok' : 'err');
}

async function getMe(): Promise<Me | null> {
  try {
    const r = await fetch('/api/me');
    if (!r.ok) return null;
    return (await r.json()) as Me;
  } catch {
    return null;
  }
}

async function getStatus(): Promise<Status> {
  const r = await fetch('/api/status', { cache: 'no-store' });
  if (!r.ok) throw new Error(`HTTP ${r.status}`);
  return r.json() as Promise<Status>;
}

// ---------- login screen ----------
function showLogin(): void {
  const app = document.getElementById('app')!;
  app.replaceChildren();
  const box = el('div', 'login-box');
  box.appendChild(el('div', 'logo', 'M'));
  box.appendChild(el('h1', undefined, 'MiniRoblox Admin'));
  box.appendChild(el('p', 'sub', 'ต้องล็อกอินด้วยบัญชี Google เพื่อเข้าใช้งาน'));
  const btn = el('a', 'btn primary', 'ล็อกอินด้วย Google');
  (btn as HTMLAnchorElement).href = '/login';
  box.appendChild(btn);
  app.appendChild(box);
}

// ---------- static shell ----------
function buildShell(): void {
  const app = document.getElementById('app')!;
  app.replaceChildren();

  const header = el('header');
  const brand = el('div', 'brand');
  brand.appendChild(el('div', 'logo', 'M'));
  const titleBox = el('div');
  titleBox.appendChild(el('h1', undefined, 'MiniRoblox Admin'));
  titleBox.appendChild(el('div', 'sub', 'server console · 45.91.134.37'));
  brand.appendChild(titleBox);

  const right = el('div', 'conn');
  const roleBadge = el('span', 'role-badge');
  roleBadge.id = 'role-badge';
  roleBadge.textContent = me?.role ?? '';
  const logout = el('a', 'btn ghost', 'ออก');
  (logout as HTMLAnchorElement).href = '/logout';
  right.appendChild(roleBadge);
  right.appendChild(logout);
  header.appendChild(brand);
  header.appendChild(right);
  app.appendChild(header);

  const stats = el('div', 'stats');
  stats.id = 'stats';
  app.appendChild(stats);

  const grid = el('div', 'grid');

  // ผู้เล่น panel
  const playersPanel = el('div', 'panel');
  playersPanel.appendChild(el('h2', undefined, 'ผู้เล่น'));
  const playersBody = el('div', 'body');
  playersBody.id = 'players-body';
  playersPanel.appendChild(playersBody);
  grid.appendChild(playersPanel);

  // ฐานข้อมูล panel (เฉพาะ admin/manager)
  if (canDbOps()) {
    const dbPanel = el('div', 'panel');
    dbPanel.appendChild(el('h2', undefined, 'ฐานข้อมูล'));
    const dbActions = el('div', 'db-actions');
    const row1 = el('div', 'row');
    const backupBtn = el('button', 'btn ghost', 'สำรองข้อมูล');
    const repairBtn = el('button', 'btn ghost', 'ซ่อม (VACUUM)');
    row1.appendChild(backupBtn);
    row1.appendChild(repairBtn);
    dbActions.appendChild(row1);
    if (canRestore()) {
      const row2 = el('div', 'row');
      const restoreBtn = el('button', 'btn primary', 'กู้คืนจากไฟล์');
      const fileInput = el('input') as HTMLInputElement;
      fileInput.type = 'file';
      fileInput.accept = '.db,.sqlite';
      row2.appendChild(restoreBtn);
      row2.appendChild(fileInput);
      dbActions.appendChild(row2);
      restoreBtn.addEventListener('click', () => fileInput.click());
      fileInput.addEventListener('change', async () => {
        const f = fileInput.files?.[0];
        if (!f) return;
        setMsg(`กำลังกู้คืนจาก ${f.name}...`);
        const t = await fetch('/restore', { method: 'POST', body: f }).then((r) => r.text());
        setMsg(t, t.startsWith('restore OK'));
      });
    }
    const msg = el('div', 'msg');
    msg.id = 'msg';
    dbPanel.appendChild(dbActions);
    dbPanel.appendChild(msg);
    grid.appendChild(dbPanel);
    backupBtn.addEventListener('click', () => {
      window.location.href = '/backup';
    });
    repairBtn.addEventListener('click', async () => {
      setMsg('กำลังซ่อมฐานข้อมูล...');
      const t = await fetch('/repair', { method: 'POST' }).then((r) => r.text());
      setMsg(t, t.includes('integrity_check'));
    });
  }

  // จัดการ role (เฉพาะ admin)
  if (canRoles()) {
    const rolesPanel = el('div', 'panel');
    rolesPanel.appendChild(el('h2', undefined, 'สิทธิ์ผู้ใช้ (roles)'));
    const rolesBody = el('div', 'body');
    rolesBody.id = 'roles-body';
    const form = el('div', 'role-form');
    const emailInput = el('input') as HTMLInputElement;
    emailInput.placeholder = 'อีเมลผู้ใช้';
    const select = el('select') as HTMLSelectElement;
    for (const r of ['admin', 'manager', 'sales', 'user']) {
      const o = el('option') as HTMLOptionElement;
      o.value = r;
      o.textContent = r;
      select.appendChild(o);
    }
    const setBtn = el('button', 'btn', 'ตั้ง role');
    form.appendChild(emailInput);
    form.appendChild(select);
    form.appendChild(setBtn);
    rolesBody.appendChild(form);
    const list = el('div', 'roles-list');
    list.id = 'roles-list';
    rolesBody.appendChild(list);
    rolesPanel.appendChild(rolesBody);
    grid.appendChild(rolesPanel);
    setBtn.addEventListener('click', async () => {
      const email = emailInput.value.trim();
      if (!email) return;
      const t = await fetch('/api/roles', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, role: select.value }),
      }).then((r) => r.text());
      setMsg(t, true);
      renderRoles();
    });
  }

  app.appendChild(grid);
}

// ---------- dynamic render ----------
function statCard(label: string, value: number, unit: string, accent = false): HTMLElement {
  const c = el('div', 'card' + (accent ? ' accent' : ''));
  c.appendChild(el('div', 'label', label));
  const v = el('div', 'value', fmt(value));
  v.appendChild(el('span', 'unit', unit));
  c.appendChild(v);
  return c;
}

function renderStats(s: Status): void {
  const box = document.getElementById('stats')!;
  box.replaceChildren(
    statCard('ออนไลน์', s.online_count, 'คน'),
    statCard('โลก', s.world_blocks, 'บล็อก'),
    statCard('ผู้เล่นทั้งหมด', s.total_players, 'คน'),
    statCard('เหรียญรวม', s.total_coins, 'coins', true),
  );
}

function renderPlayers(s: Status): void {
  const body = document.getElementById('players-body')!;
  body.replaceChildren();
  if (s.players.length === 0) {
    body.appendChild(el('div', 'empty', 'ยังไม่มีผู้เล่น'));
    return;
  }
  const table = el('table');
  const thead = el('thead');
  const hr = el('tr');
  for (const h of ['ผู้เล่น', 'เหรียญ', 'สถานะ', canCoins() ? '' : '']) hr.appendChild(el('th', undefined, h));
  thead.appendChild(hr);
  table.appendChild(thead);

  const tbody = el('tbody');
  const players = [...s.players].sort(
    (a, b) => (b.online ? 1 : 0) - (a.online ? 1 : 0) || b.coins - a.coins,
  );
  for (const p of players) {
    const tr = el('tr');
    const nameTd = el('td');
    nameTd.appendChild(el('div', 'name', p.name));
    nameTd.appendChild(el('div', 'email-hint', `(${p.x.toFixed(1)}, ${p.y.toFixed(1)}, ${p.z.toFixed(1)})`));
    tr.appendChild(nameTd);

    const coinsTd = el('td');
    coinsTd.appendChild(el('span', 'coins', fmt(p.coins)));
    tr.appendChild(coinsTd);

    const statusTd = el('td');
    const st = el('span', 'status ' + (p.online ? 'on' : 'off'));
    st.appendChild(el('span', 'dot-sm'));
    st.appendChild(document.createTextNode(p.online ? 'ออนไลน์' : 'ออฟไลน์'));
    statusTd.appendChild(st);
    tr.appendChild(statusTd);

    if (canCoins()) {
      const actTd = el('td');
      const act = el('div', 'coin-actions');
      const minus = el('button', 'coin-btn minus', '-');
      const plus = el('button', 'coin-btn', '+');
      minus.title = 'ลด 10 เหรียญ';
      plus.title = 'เพิ่ม 10 เหรียญ';
      minus.addEventListener('click', () => adjustCoins(p.name, -10));
      plus.addEventListener('click', () => adjustCoins(p.name, 10));
      act.appendChild(minus);
      act.appendChild(plus);
      actTd.appendChild(act);
      tr.appendChild(actTd);
    }
    tbody.appendChild(tr);
  }
  table.appendChild(tbody);
  body.appendChild(table);
}

async function adjustCoins(name: string, delta: number): Promise<void> {
  setMsg(`ปรับเหรียญ ${name} ${delta > 0 ? '+' : ''}${delta}...`);
  try {
    const r = await fetch('/api/coins', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name, delta }),
    });
    const t = await r.text();
    setMsg(t, r.ok);
    await refresh();
  } catch (e) {
    setMsg(String(e), false);
  }
}

async function renderRoles(): Promise<void> {
  const list = document.getElementById('roles-list');
  if (!list) return;
  list.replaceChildren();
  const r = await fetch('/api/roles');
  if (!r.ok) return;
  const roles: [string, string][] = await r.json();
  if (roles.length === 0) {
    list.appendChild(el('div', 'empty', 'ยังไม่มีการตั้ง role (ค่าเริ่มต้น: user)'));
    return;
  }
  for (const [email, role] of roles) {
    const row = el('div', 'role-row');
    row.appendChild(el('span', 'name', email));
    row.appendChild(el('span', 'role-badge', role));
    list.appendChild(row);
  }
}

async function refresh(): Promise<void> {
  try {
    const s = await getStatus();
    renderStats(s);
    renderPlayers(s);
  } catch {
    /* ถอยเงียบ ๆ */
  }
  if (canRoles()) void renderRoles();
}

// ---------- boot ----------
(async () => {
  me = await getMe();
  if (!me) {
    showLogin();
    return;
  }
  buildShell();
  void refresh();
  setInterval(() => void refresh(), 5000);
})();
