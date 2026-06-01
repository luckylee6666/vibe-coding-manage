let invoke;
try {
  invoke = window.__TAURI__.core.invoke;
} catch (e) {
  document.body.innerHTML = '<div style="padding:40px;color:red;font-size:16px;">Tauri API 未加载，请用 <code>pnpm tauri dev</code> 启动</div>';
  throw e;
}

let projects = [];
let servers = [];
let currentEditId = null;
let pendingConfirm = null;
let activeGroup = 'all';
let currentServerEditId = null;

const $ = id => document.getElementById(id);

const el = {
  list: $('project-list'),
  empty: $('empty-state'),
  count: $('project-count'),
  countAll: $('count-all'),
  search: $('search-input'),
  addBtn: $('add-btn'),
  exportBtn: $('export-btn'),
  modal: $('modal-overlay'),
  modalTitle: $('modal-title'),
  modalClose: $('modal-close'),
  form: $('project-form'),
  id: $('project-id'),
  name: $('project-name'),
  path: $('project-path'),
  url: $('project-url'),
  machine: $('project-machine'),
  serverSelectWrap: $('server-select-wrap'),
  serverSelect: $('project-server'),
  manageServerBtn: $('manage-server-btn'),
  group: $('project-group'),
  groupList: $('group-list'),
  groupSuggestions: $('group-suggestions'),
  desc: $('project-desc'),
  browse: $('browse-btn'),
  cancel: $('cancel-btn'),
  submit: $('submit-btn'),
  confirm: $('confirm-overlay'),
  confirmClose: $('confirm-close'),
  deleteName: $('delete-project-name'),
  confirmCancel: $('confirm-cancel'),
  confirmDelete: $('confirm-delete'),
  toasts: $('toast-container'),
  serverModal: $('server-modal-overlay'),
  serverModalTitle: $('server-modal-title'),
  serverModalClose: $('server-modal-close'),
  serverForm: $('server-form'),
  serverId: $('server-id'),
  serverName: $('server-name'),
  serverHost: $('server-host'),
  serverPort: $('server-port'),
  serverUser: $('server-user'),
  serverAuthType: $('server-auth-type'),
  serverNote: $('server-note'),
  serverCancelBtn: $('server-cancel-btn'),
  serverSubmitBtn: $('server-submit-btn'),
  serverListOverlay: $('server-list-overlay'),
  serverListClose: $('server-list-close'),
  addServerBtn: $('add-server-btn'),
  serverList: $('server-list'),
  serverEmpty: $('server-empty'),
  scanBtn: $('scan-btn'),
  scanModal: $('scan-modal-overlay'),
  scanModalTitle: $('scan-modal-title'),
  scanModalClose: $('scan-modal-close'),
  scanStatus: $('scan-status'),
  scanList: $('scan-list'),
  scanEmpty: $('scan-empty'),
  scanCancelBtn: $('scan-cancel-btn'),
  scanImportBtn: $('scan-import-btn'),
};

async function init() {
  await load();
  bind();
}

async function load() {
  try {
    projects = await invoke('get_projects');
    servers = await invoke('get_servers');
    renderGroups();
    render(projects);
    el.count.textContent = projects.length;
    el.countAll.textContent = projects.length;
  } catch (e) {
    console.error('加载失败:', e);
    msg('加载失败: ' + (e.message || e), 'error');
  }
}

function getGroups() {
  const groups = {};
  projects.forEach(p => {
    const g = p.group || '未分组';
    groups[g] = (groups[g] || 0) + 1;
  });
  const sorted = {};
  Object.keys(groups)
    .sort((a, b) => a === '未分组' ? 1 : b === '未分组' ? -1 : a.localeCompare(b))
    .forEach(k => sorted[k] = groups[k]);
  return sorted;
}

function renderGroups() {
  const groups = getGroups();
  const entries = Object.entries(groups);
  
  el.groupList.innerHTML = entries.map(([name, count]) => {
    const isActive = activeGroup === name;
    const groupProjects = projects.filter(p => (p.group || '未分组') === name);
    return `
    <div class="menu-group-item ${isActive ? 'expanded' : ''}" data-group="${esc(name)}">
      <a class="menu-item ${isActive ? 'active' : ''}" href="#" data-group="${esc(name)}">
        <svg class="group-arrow" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M9 18l6-6-6-6"/>
        </svg>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M3.75 9.776c.112-.017.227-.026.344-.026h15.812c.117 0 .232.009.344.026m-16.5 0a2.25 2.25 0 00-1.883 2.542l.857 6a2.25 2.25 0 002.227 1.932H19.05a2.25 2.25 0 002.227-1.932l.857-6a2.25 2.25 0 00-1.883-2.542m-16.5 0V6A2.25 2.25 0 016 3.75h3.879a1.5 1.5 0 011.06.44l2.122 2.12a1.5 1.5 0 001.06.44H18A2.25 2.25 0 0120.25 9v.776"/>
        </svg>
        <span>${esc(name)}</span>
        <span class="menu-badge">${count}</span>
      </a>
      <div class="group-children">
        ${groupProjects.map(p => `
          <a class="menu-child-item" href="#" data-id="${p.id}">
            <span>${esc(p.name)}</span>
          </a>
        `).join('')}
      </div>
    </div>
    `;
  }).join('');

  el.groupSuggestions.innerHTML = Object.keys(groups)
    .filter(g => g !== '未分组')
    .map(g => `<option value="${esc(g)}">`)
    .join('');

  el.groupList.querySelectorAll('.menu-group-item > .menu-item').forEach(item => {
    item.onclick = (e) => {
      e.preventDefault();
      const groupName = item.dataset.group;
      const groupEl = item.parentElement;
      if (activeGroup === groupName) {
        groupEl.classList.toggle('expanded');
      } else {
        activeGroup = groupName;
        document.querySelectorAll('.sider .menu-item').forEach(i => i.classList.remove('active'));
        document.querySelectorAll('.sider .menu-group-item').forEach(g => g.classList.remove('expanded'));
        item.classList.add('active');
        groupEl.classList.add('expanded');
        filterAndRender();
      }
    };
  });

  el.groupList.querySelectorAll('.menu-child-item').forEach(child => {
    child.onclick = (e) => {
      e.preventDefault();
      const id = child.dataset.id;
      const card = document.querySelector(`.project-card[data-id="${id}"]`);
      if (card) {
        card.scrollIntoView({ behavior: 'smooth', block: 'center' });
        card.classList.add('highlight');
        setTimeout(() => card.classList.remove('highlight'), 1500);
      }
    };
  });
}

function filterAndRender() {
  const q = el.search.value.toLowerCase();
  let filtered = projects;

  if (activeGroup !== 'all') {
    filtered = filtered.filter(p => (p.group || '未分组') === activeGroup);
  }

  if (q) {
    filtered = filtered.filter(p =>
      p.name.toLowerCase().includes(q) ||
      p.description.toLowerCase().includes(q) ||
      p.localPath.toLowerCase().includes(q) ||
      p.remoteUrl.toLowerCase().includes(q)
    );
  }

  render(filtered);
}

function getServerName(serverId) {
  const s = servers.find(x => x.id === serverId);
  return s ? s.name : '';
}

function render(list) {
  if (!list.length) {
    el.empty.style.display = 'flex';
    el.list.style.display = 'none';
    return;
  }
  el.empty.style.display = 'none';
  el.list.style.display = 'flex';

  el.list.innerHTML = list.map(p => `
    <div class="project-card" data-id="${p.id}">
      <div class="card-row">
        <div class="card-main">
          <div class="card-title">
            ${esc(p.name)}
            ${p.group ? `<span class="card-group">${esc(p.group)}</span>` : ''}
          </div>
          <div class="card-info">
            <div class="info-item">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M2.25 12.75V12A2.25 2.25 0 014.5 9.75h15A2.25 2.25 0 0121.75 12v.75m-8.69-6.44l-2.12-2.12a1.5 1.5 0 00-1.061-.44H4.5A2.25 2.25 0 002.25 6v12a2.25 2.25 0 002.25 2.25h15A2.25 2.25 0 0021.75 18V9a2.25 2.25 0 00-2.25-2.25h-5.379a1.5 1.5 0 01-1.06-.44z"/></svg>
              <span title="${esc(p.localPath)}">${esc(short(p.localPath))}</span>
            </div>
            ${p.remoteUrl ? `<div class="info-item">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M13.19 8.688a4.5 4.5 0 011.242 7.244l-4.5 4.5a4.5 4.5 0 01-6.364-6.364l1.757-1.757m13.35-.622l1.757-1.757a4.5 4.5 0 00-6.364-6.364l-4.5 4.5a4.5 4.5 0 001.242 7.244"/></svg>
              <a href="${esc(p.remoteUrl)}" target="_blank">${esc(repo(p.remoteUrl))}</a>
            </div>` : ''}
          </div>
          ${p.description ? `<div class="card-desc">${esc(p.description)}</div>` : ''}
          <div class="card-tags">
            <span class="tag ${tagCls(p.machine)}">${tagLabel(p.machine)}</span>
            ${p.machine === 'server' && p.serverId ? `<span class="tag tag-server">${esc(getServerName(p.serverId))}</span>` : ''}
          </div>
        </div>
        <div class="card-actions">
          <button class="action-btn terminal-btn" title="打开终端">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M6.75 7.5l3 2.25-3 2.25m4.5 0h3M3.375 3h17.25c.621 0 1.125.504 1.125 1.125v15.75c0 .621-.504 1.125-1.125 1.125H3.375c-.621 0-1.125-.504-1.125-1.125V4.125C2.25 3.504 2.754 3 3.375 3z"/></svg>
          </button>
          <button class="action-btn edit-btn" title="编辑">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M16.862 4.487l1.687-1.688a1.875 1.875 0 112.652 2.652L10.582 16.07a4.5 4.5 0 01-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 011.13-1.897l8.932-8.931zm0 0L19.5 7.125M18 14v4.75A2.25 2.25 0 0115.75 21H5.25A2.25 2.25 0 013 18.75V8.25A2.25 2.25 0 015.25 6H10"/></svg>
          </button>
          <button class="action-btn danger del-btn" title="删除">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M14.74 9l-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 01-2.244 2.077H8.084a2.25 2.25 0 01-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 00-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 013.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 00-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 00-7.5 0"/></svg>
          </button>
        </div>
      </div>
    </div>
  `).join('');

  el.list.querySelectorAll('.project-card').forEach(card => {
    const id = card.dataset.id;
    const p = projects.find(x => x.id === id);
    if (!p) return;
    card.querySelector('.terminal-btn').onclick = () => openTerminal(p);
    card.querySelector('.edit-btn').onclick = () => openModal(p);
    card.querySelector('.del-btn').onclick = () => del(p.id, p.name);
  });
}

function bind() {
  el.addBtn.onclick = () => openModal();
  el.exportBtn.onclick = exportExcel;
  el.modalClose.onclick = closeModal;
  el.cancel.onclick = closeModal;
  el.modal.onclick = e => { if (e.target === el.modal) closeModal(); };
  el.form.onsubmit = submit;
  el.submit.type = 'button';
  el.submit.onclick = () => submit(new Event('submit'));
  el.browse.onclick = browse;
  el.confirmClose.onclick = closeDel;
  el.confirmCancel.onclick = closeDel;
  el.confirm.onclick = e => { if (e.target === el.confirm) closeDel(); };
  el.confirmDelete.onclick = doDelete;
  el.search.oninput = () => filterAndRender();

  document.querySelector('[data-group="all"]').onclick = (e) => {
    e.preventDefault();
    activeGroup = 'all';
    document.querySelectorAll('.sider .menu-item').forEach(i => i.classList.remove('active'));
    e.currentTarget.classList.add('active');
    filterAndRender();
  };

  document.onkeydown = e => { if (e.key === 'Escape') { closeModal(); closeDel(); closeServerModal(); closeServerList(); } };

  // 运行环境切换时显示/隐藏服务器选择
  el.machine.onchange = () => {
    el.serverSelectWrap.style.display = el.machine.value === 'server' ? '' : 'none';
    if (el.machine.value === 'server') renderServerOptions();
  };

  // 服务器管理按钮
  el.manageServerBtn.onclick = () => openServerList();

  // 服务器弹窗事件
  el.serverModalClose.onclick = closeServerModal;
  el.serverCancelBtn.onclick = closeServerModal;
  el.serverModal.onclick = e => { if (e.target === el.serverModal) closeServerModal(); };
  el.serverForm.onsubmit = submitServer;
  el.serverSubmitBtn.type = 'button';
  el.serverSubmitBtn.onclick = () => submitServer(new Event('submit'));

  // 服务器列表弹窗事件
  el.serverListClose.onclick = closeServerList;
  el.serverListOverlay.onclick = e => { if (e.target === el.serverListOverlay) closeServerList(); };
  el.addServerBtn.onclick = () => { closeServerList(); openServerModal(); };

  // 侧边栏服务器管理入口
  $('server-manage-entry').onclick = (e) => {
    e.preventDefault();
    openServerList();
  };

  // 扫描导入
  el.scanBtn.onclick = startScan;
  el.scanModalClose.onclick = closeScanModal;
  el.scanCancelBtn.onclick = closeScanModal;
  el.scanModal.onclick = e => { if (e.target === el.scanModal) closeScanModal(); };
  el.scanImportBtn.onclick = importScanned;

  // 内置终端
  termEl.fab.onclick = () => { sessions.size ? openDock() : createSession({}); };
  termEl.collapseBtn.onclick = collapseDock;
  termEl.maximizeBtn.onclick = toggleDockMaximize;
  termEl.newBtn.onclick = () => createSession({});
  setupTermResize();
  window.addEventListener('resize', () => {
    if (termEl.dock.classList.contains('maximized')) termEl.dock.style.height = window.innerHeight + 'px';
    if (activeSession) fitSession(activeSession);
  });
}

function openModal(p = null) {
  currentEditId = p ? p.id : null;
  el.modalTitle.textContent = p ? '编辑项目' : '新建项目';
  if (p) {
    el.id.value = p.id;
    el.name.value = p.name;
    el.path.value = p.localPath;
    el.url.value = p.remoteUrl;
    el.machine.value = p.machine;
    el.group.value = p.group || '';
    el.desc.value = p.description;
    if (p.machine === 'server' && p.serverId) {
      el.serverSelectWrap.style.display = '';
      renderServerOptions();
      el.serverSelect.value = p.serverId;
    } else {
      el.serverSelectWrap.style.display = 'none';
    }
  } else {
    el.form.reset();
    el.id.value = '';
    el.serverSelectWrap.style.display = 'none';
  }
  el.modal.classList.add('active');
  el.name.focus();
}

function closeModal() {
  el.modal.classList.remove('active');
  currentEditId = null;
}

// 通用确认弹窗（WKWebView 不支持原生 confirm，统一走应用内弹窗）
function askConfirm(kind, name, onConfirm) {
  $('confirm-kind').textContent = kind;
  el.deleteName.textContent = name;
  pendingConfirm = onConfirm;
  el.confirm.classList.add('active');
}

function del(id, name) {
  askConfirm('项目', name, async () => {
    await invoke('delete_project', { id });
    msg('删除成功', 'success');
    await load();
  });
}

function closeDel() {
  el.confirm.classList.remove('active');
  pendingConfirm = null;
}

async function browse() {
  try {
    const r = await invoke('open_folder_dialog');
    if (r) el.path.value = r;
  } catch (e) {
    console.error('选择文件夹失败:', e);
  }
}

async function openTerminal(p) {
  try {
    await createSession({ cwd: p.localPath, name: p.name, autoClaude: true });
  } catch (e) {
    console.error('打开终端失败:', e);
    msg('打开终端失败: ' + (e.message || e), 'error');
  }
}

async function submit(e) {
  e.preventDefault();
  if (!el.form.checkValidity()) {
    el.form.reportValidity();
    return;
  }

  const data = {
    name: el.name.value.trim(),
    localPath: el.path.value.trim(),
    remoteUrl: el.url.value.trim(),
    machine: el.machine.value,
    serverId: el.machine.value === 'server' ? el.serverSelect.value : '',
    group: el.group.value.trim(),
    description: el.desc.value.trim(),
  };
  if (currentEditId) data.id = currentEditId;

  if (data.machine === 'server' && !data.serverId) {
    msg('请选择服务器', 'error');
    return;
  }

  try {
    if (currentEditId) {
      await invoke('update_project', data);
      msg('更新成功', 'success');
    } else {
      await invoke('add_project', data);
      msg('创建成功', 'success');
    }
    closeModal();
    await load();
  } catch (e) {
    console.error('操作失败:', e);
    msg('操作失败: ' + (e.message || e), 'error');
  }
}

async function doDelete() {
  if (!pendingConfirm) return;
  const fn = pendingConfirm;
  try {
    await fn();
  } catch (e) {
    msg(typeof e === 'string' ? e : (e.message || '删除失败'), 'error');
  } finally {
    closeDel();
  }
}

async function exportExcel() {
  try {
    await invoke('export_excel');
    msg('导出成功', 'success');
  } catch (e) {
    if (e !== '未选择保存位置') msg('导出失败', 'error');
  }
}

// ========== 服务器管理 ==========

function renderServerOptions() {
  el.serverSelect.innerHTML = '<option value="">请选择服务器</option>' +
    servers.map(s => `<option value="${s.id}">${esc(s.name)} (${esc(s.host)})</option>`).join('');
}

function openServerModal(s = null) {
  currentServerEditId = s ? s.id : null;
  el.serverModalTitle.textContent = s ? '编辑服务器' : '添加服务器';
  if (s) {
    el.serverId.value = s.id;
    el.serverName.value = s.name;
    el.serverHost.value = s.host;
    el.serverPort.value = s.port || 22;
    el.serverUser.value = s.user;
    el.serverAuthType.value = s.authType || 'password';
    el.serverNote.value = s.note || '';
  } else {
    el.serverForm.reset();
    el.serverId.value = '';
    el.serverPort.value = '22';
    el.serverAuthType.value = 'password';
  }
  el.serverModal.classList.add('active');
  el.serverName.focus();
}

function closeServerModal() {
  el.serverModal.classList.remove('active');
  currentServerEditId = null;
}

async function submitServer(e) {
  e.preventDefault();
  if (!el.serverForm.checkValidity()) {
    el.serverForm.reportValidity();
    return;
  }

  const data = {
    name: el.serverName.value.trim(),
    host: el.serverHost.value.trim(),
    port: parseInt(el.serverPort.value) || 22,
    user: el.serverUser.value.trim(),
    authType: el.serverAuthType.value,
    note: el.serverNote.value.trim(),
  };
  if (currentServerEditId) data.id = currentServerEditId;

  try {
    if (currentServerEditId) {
      await invoke('update_server', data);
      msg('服务器更新成功', 'success');
    } else {
      await invoke('add_server', data);
      msg('服务器添加成功', 'success');
    }
    closeServerModal();
    await load();
    // 刷新服务器列表并重新打开，确保新数据可见
    openServerList();
    // 同步刷新项目表单中的服务器下拉（运行环境为服务器时）
    if (el.machine.value === 'server') renderServerOptions();
  } catch (e) {
    console.error('服务器操作失败:', e);
    msg('操作失败: ' + (e.message || e), 'error');
  }
}

function openServerList() {
  renderServerList();
  el.serverListOverlay.classList.add('active');
}

function closeServerList() {
  el.serverListOverlay.classList.remove('active');
}

function renderServerList() {
  if (!servers.length) {
    el.serverList.style.display = 'none';
    el.serverEmpty.style.display = '';
    return;
  }
  el.serverEmpty.style.display = 'none';
  el.serverList.style.display = '';

  el.serverList.innerHTML = servers.map(s => `
    <div class="server-card" data-id="${s.id}">
      <div class="server-card-main">
        <div class="server-card-name">${esc(s.name)}</div>
        <div class="server-card-info">
          <span>${esc(s.user)}@${esc(s.host)}:${s.port}</span>
          <span class="tag ${s.authType === 'key' ? 'tag-local' : 'tag-ssh'}">${s.authType === 'key' ? '秘钥' : '密码'}</span>
        </div>
        ${s.note ? `<div class="server-card-note">${esc(s.note)}</div>` : ''}
      </div>
      <div class="server-card-actions">
        <button class="action-btn edit-server-btn" title="编辑">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M16.862 4.487l1.687-1.688a1.875 1.875 0 112.652 2.652L10.582 16.07a4.5 4.5 0 01-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 011.13-1.897l8.932-8.931zm0 0L19.5 7.125M18 14v4.75A2.25 2.25 0 0115.75 21H5.25A2.25 2.25 0 013 18.75V8.25A2.25 2.25 0 015.25 6H10"/></svg>
        </button>
        <button class="action-btn danger del-server-btn" title="删除">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M14.74 9l-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 01-2.244 2.077H8.084a2.25 2.25 0 01-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 00-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 013.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 00-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 00-7.5 0"/></svg>
        </button>
      </div>
    </div>
  `).join('');

  el.serverList.querySelectorAll('.server-card').forEach(card => {
    const id = card.dataset.id;
    const s = servers.find(x => x.id === id);
    if (!s) return;
    card.querySelector('.edit-server-btn').onclick = () => { closeServerList(); openServerModal(s); };
    card.querySelector('.del-server-btn').onclick = () => {
      askConfirm('服务器', s.name, async () => {
        await invoke('delete_server', { id: s.id });
        msg('删除成功', 'success');
        await load();
        renderServerList();
      });
    };
  });
}

// ========== 工具函数 ==========

function msg(text, type = 'info') {
  const icon = type === 'success'
    ? '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/></svg>'
    : '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z"/></svg>';
  const d = document.createElement('div');
  d.className = `message ${type}`;
  d.innerHTML = `${icon}<span>${text}</span>`;
  el.toasts.appendChild(d);
  setTimeout(() => d.remove(), 3000);
}

function esc(s) {
  if (!s) return '';
  const d = document.createElement('div');
  d.textContent = s;
  return d.innerHTML;
}

function short(p) {
  if (!p || p.length <= 30) return p || '';
  const parts = p.split('/');
  return parts.length <= 3 ? p : `${parts[0]}/…/${parts.at(-1)}`;
}

function repo(url) {
  if (!url) return '';
  try { return url.split('/').pop().replace('.git', ''); } catch { return url; }
}

function tagCls(m) {
  if (m === 'local') return 'tag-local';
  if (m === 'server') return 'tag-ssh';
  return 'tag-other';
}

function tagLabel(m) {
  return { local: '本地电脑', server: '服务器' }[m] || m || '未知';
}

// ========== 扫描导入 ==========

let scannedProjects = [];

async function startScan() {
  try {
    const dir = await invoke('open_pick_directory');
    if (!dir) return;
    el.scanStatus.textContent = '扫描中...';
    el.scanList.innerHTML = '';
    el.scanEmpty.style.display = 'none';
    el.scanModal.classList.add('active');

    scannedProjects = await invoke('scan_directory', { path: dir });

    el.scanStatus.textContent = `在 ${dir} 中发现 ${scannedProjects.length} 个项目`;

    if (!scannedProjects.length) {
      el.scanEmpty.style.display = '';
      return;
    }

    el.scanList.innerHTML = scannedProjects.map((p, i) => `
      <label class="scan-item" data-index="${i}">
        <input type="checkbox" checked class="scan-checkbox" />
        <div class="scan-item-info">
          <div class="scan-item-name">${esc(p.name)}</div>
          <div class="scan-item-path">${esc(p.path)}</div>
          ${p.remoteUrl ? `<div class="scan-item-url">${esc(p.remoteUrl)}</div>` : ''}
        </div>
      </label>
    `).join('');
  } catch (e) {
    console.error('扫描失败:', e);
    msg('扫描失败: ' + (e.message || e), 'error');
  }
}

function closeScanModal() {
  el.scanModal.classList.remove('active');
  scannedProjects = [];
}

async function importScanned() {
  const checkboxes = el.scanList.querySelectorAll('.scan-checkbox');
  const toImport = [];
  checkboxes.forEach((cb, i) => {
    if (cb.checked && scannedProjects[i]) {
      toImport.push(scannedProjects[i]);
    }
  });

  if (!toImport.length) {
    msg('请至少选择一个项目', 'error');
    return;
  }

  // 按本地路径去重，跳过已存在的项目，避免重复扫描导入产生重复条目
  const existingPaths = new Set(projects.map(p => p.localPath));

  let success = 0;
  let fail = 0;
  let skipped = 0;
  for (const p of toImport) {
    if (existingPaths.has(p.path)) {
      skipped++;
      continue;
    }
    try {
      await invoke('add_project', {
        name: p.name,
        localPath: p.path,
        remoteUrl: p.remoteUrl || '',
        machine: 'local',
        serverId: '',
        group: p.group || '',
        description: '',
      });
      existingPaths.add(p.path);
      success++;
    } catch (e) {
      console.error('导入失败:', p.name, e);
      fail++;
    }
  }

  closeScanModal();
  await load();

  const parts = [`${success} 个成功`];
  if (skipped) parts.push(`${skipped} 个已存在跳过`);
  if (fail) parts.push(`${fail} 个失败`);
  msg(`导入完成：${parts.join('，')}`, fail ? 'error' : 'success');
}

// ========== 内置终端 ==========

const termEl = {
  dock: $('terminal-dock'),
  tabs: $('terminal-tabs'),
  bodies: $('terminal-bodies'),
  resize: $('terminal-resize'),
  newBtn: $('terminal-new-btn'),
  maximizeBtn: $('terminal-maximize-btn'),
  collapseBtn: $('terminal-collapse-btn'),
  fab: $('terminal-fab'),
  fabBadge: $('terminal-fab-badge'),
};

const sessions = new Map(); // id -> { term, fit, tabEl, bodyEl, name, status }
let activeSession = null;
let termSeq = 0;
let termEventsBound = false;

function b64ToBytes(b64) {
  const bin = atob(b64);
  const bytes = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
  return bytes;
}

async function bindTermEvents() {
  if (termEventsBound) return;
  termEventsBound = true;
  const listen = window.__TAURI__.event.listen;
  await listen('terminal-output', e => {
    const s = sessions.get(e.payload.id);
    if (s) s.term.write(b64ToBytes(e.payload.data));
  });
  await listen('terminal-exit', e => {
    const s = sessions.get(e.payload);
    if (s) {
      s.status = 'exited';
      s.tabEl.classList.add('exited');
      s.term.write('\r\n\x1b[90m[会话已结束]\x1b[0m\r\n');
    }
  });
}

function openDock() {
  termEl.dock.classList.add('active');
  termEl.fab.classList.add('hidden');
  if (activeSession) requestAnimationFrame(() => fitSession(activeSession));
}

function collapseDock() {
  termEl.dock.classList.remove('active');
  termEl.fab.classList.remove('hidden');
}

// 最大化/还原终端抽屉，最大化时占满整个窗口高度、不留顶部白边
let dockPrevHeight = null;
function toggleDockMaximize() {
  const maxed = termEl.dock.classList.toggle('maximized');
  if (maxed) {
    dockPrevHeight = termEl.dock.offsetHeight;
    termEl.dock.style.height = window.innerHeight + 'px';
    termEl.maximizeBtn.title = '还原';
  } else {
    termEl.dock.style.height = (dockPrevHeight || 340) + 'px';
    termEl.maximizeBtn.title = '最大化';
  }
  if (activeSession) requestAnimationFrame(() => fitSession(activeSession));
}

function updateFabBadge() {
  const n = sessions.size;
  termEl.fabBadge.style.display = n ? '' : 'none';
  termEl.fabBadge.textContent = n;
}

function fitSession(id) {
  const s = sessions.get(id);
  if (!s) return;
  try {
    s.fit.fit();
    invoke('terminal_resize', { id, cols: s.term.cols, rows: s.term.rows }).catch(() => {});
  } catch (e) {}
}

function activateSession(id) {
  const s = sessions.get(id);
  if (!s) return;
  activeSession = id;
  sessions.forEach((other, oid) => {
    const on = oid === id;
    other.tabEl.classList.toggle('active', on);
    other.bodyEl.classList.toggle('active', on);
  });
  fitSession(id);
  s.term.focus();
}

async function closeSession(id) {
  const s = sessions.get(id);
  if (!s) return;
  try { await invoke('terminal_close', { id }); } catch (e) {}
  s.term.dispose();
  s.tabEl.remove();
  s.bodyEl.remove();
  sessions.delete(id);
  if (activeSession === id) {
    activeSession = null;
    const next = sessions.keys().next().value;
    if (next) activateSession(next);
    else collapseDock();
  }
  updateFabBadge();
}

async function createSession({ cwd = '', name = '', autoClaude = false }) {
  await bindTermEvents();
  const id = `term-${Date.now()}-${++termSeq}`;
  const label = name || `终端 ${termSeq}`;

  const bodyEl = document.createElement('div');
  bodyEl.className = 'term-body';
  bodyEl.dataset.id = id;
  termEl.bodies.appendChild(bodyEl);

  const tabEl = document.createElement('div');
  tabEl.className = 'term-tab';
  tabEl.dataset.id = id;
  tabEl.innerHTML =
    `<span class="term-tab-dot"></span>` +
    `<span class="term-tab-name" title="${esc(label)}">${esc(label)}</span>` +
    `<span class="term-tab-close" title="关闭"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 18L18 6M6 6l12 12"/></svg></span>`;
  termEl.tabs.appendChild(tabEl);
  tabEl.onclick = (ev) => {
    if (ev.target.closest('.term-tab-close')) { closeSession(id); return; }
    activateSession(id);
  };

  const term = new window.Terminal({
    fontSize: 13,
    fontFamily: '"JetBrains Mono", Menlo, Monaco, "Courier New", monospace',
    cursorBlink: true,
    scrollback: 5000,
    theme: { background: '#14171e', foreground: '#e6eaf2', cursor: '#1677ff' },
  });
  const fit = new window.FitAddon.FitAddon();
  term.loadAddon(fit);
  term.open(bodyEl);
  term.onData(d => invoke('terminal_write', { id, data: d }).catch(() => {}));

  sessions.set(id, { term, fit, tabEl, bodyEl, name: label, status: 'running' });

  openDock();
  activateSession(id);
  requestAnimationFrame(() => fitSession(id));
  updateFabBadge();

  try {
    await invoke('terminal_create', { id, cwd, cols: term.cols || 80, rows: term.rows || 24 });
    fitSession(id);
    if (autoClaude) {
      setTimeout(() => invoke('terminal_write', { id, data: 'claude\r' }).catch(() => {}), 400);
    }
  } catch (e) {
    term.write(`\r\n\x1b[31m启动失败: ${e}\x1b[0m\r\n`);
  }

  return id;
}

function setupTermResize() {
  let startY = 0;
  let startH = 0;
  const onMove = (e) => {
    const h = Math.min(Math.max(startH + (startY - e.clientY), 160), window.innerHeight);
    termEl.dock.style.height = h + 'px';
    if (activeSession) fitSession(activeSession);
  };
  const onUp = () => {
    document.removeEventListener('mousemove', onMove);
    document.removeEventListener('mouseup', onUp);
    document.body.style.userSelect = '';
  };
  termEl.resize.addEventListener('mousedown', (e) => {
    startY = e.clientY;
    startH = termEl.dock.offsetHeight;
    document.body.style.userSelect = 'none';
    document.addEventListener('mousemove', onMove);
    document.addEventListener('mouseup', onUp);
  });
}

init();
