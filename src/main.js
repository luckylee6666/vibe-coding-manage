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
  launchMenu: $('launch-menu'),
  treeCtxMenu: $('tree-context-menu'),
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
    card.querySelector('.terminal-btn').onclick = (ev) => {
      ev.stopPropagation();
      openLaunchMenu(p, ev.currentTarget);
    };
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

  // 「打开终端」AI CLI 选择菜单
  el.launchMenu.querySelectorAll('.launch-item').forEach(item => {
    item.onclick = () => {
      const cmd = item.dataset.cmd;
      const p = launchMenuProject;
      closeLaunchMenu();
      if (p) openTerminal(p, cmd);
    };
  });
  document.addEventListener('click', (e) => {
    if (!e.target.closest('#launch-menu') && !e.target.closest('.terminal-btn')) closeLaunchMenu();
  });
  window.addEventListener('resize', closeLaunchMenu);
  document.addEventListener('scroll', closeLaunchMenu, true);

  // 内置终端
  termEl.fab.onclick = () => { sessions.size ? openDock() : createSession({}); };
  termEl.collapseBtn.onclick = collapseDock;
  termEl.maximizeBtn.onclick = toggleDockMaximize;
  termEl.newBtn.onclick = () => createSession({});
  termEl.themeBtn.onclick = (e) => {
    e.stopPropagation();
    termEl.themeMenu.classList.contains('active') ? closeThemeMenu() : openThemeMenu();
  };
  document.addEventListener('click', (e) => {
    if (!e.target.closest('.term-theme-wrap')) closeThemeMenu();
  });
  // 终端字号快捷键：⌘/Ctrl + 加号放大、减号缩小、0 复位（capture 阶段抢在 xterm 之前）
  document.addEventListener('keydown', (e) => {
    if (!termEl.dock.classList.contains('active')) return;
    if (!(e.metaKey || e.ctrlKey) || e.altKey) return;
    if (e.key === '=' || e.key === '+') { e.preventDefault(); setTermFontSize(currentFontSize + 1); }
    else if (e.key === '-' || e.key === '_') { e.preventDefault(); setTermFontSize(currentFontSize - 1); }
    else if (e.key === '0') { e.preventDefault(); setTermFontSize(TERM_FONT_DEFAULT); }
  }, true);
  // ⌘/Ctrl + 滚轮缩放字号
  termEl.bodies.addEventListener('wheel', (e) => {
    if (!(e.metaKey || e.ctrlKey)) return;
    e.preventDefault();
    setTermFontSize(currentFontSize + (e.deltaY < 0 ? 1 : -1));
  }, { passive: false });
  termEl.bodies.style.background = TERM_THEMES[currentTheme].theme.background;
  setupTermResize();
  // 文件树 + 内容预览
  termEl.treeBtn.onclick = toggleTree;
  termEl.treeRefreshBtn.onclick = () => renderTree(treeRoot);
  termEl.previewInsert.onclick = () => insertPathToTerminal(termEl.previewInsert.dataset.path || '');
  termEl.previewToggle.onclick = togglePreviewMode;
  termEl.previewClose.onclick = closePreview;
  // 渲染视图里的链接走系统浏览器，别让主 webview 导航走
  termEl.previewRich.addEventListener('click', (e) => {
    const a = e.target.closest('a[href]');
    if (!a) return;
    e.preventDefault();
    const href = a.getAttribute('href') || '';
    if (/^https?:\/\//i.test(href)) invoke('open_folder', { path: href }).catch(() => {});
  });
  setupTreeSplitter();
  setupTreeDrag();
  // 文件树右键菜单
  el.treeCtxMenu.querySelectorAll('.ctx-item').forEach(item => {
    item.onclick = () => {
      const action = item.dataset.action;
      const ctx = treeCtx;
      closeTreeCtx();
      if (!ctx) return;
      if (action === 'open') {
        const target = ctx.entry.isDir ? ctx.entry.path : parentDir(ctx.entry.path);
        invoke('open_folder', { path: target }).catch(e => msg('打开失败: ' + e, 'error'));
      } else if (action === 'insert') {
        insertPathToTerminal(ctx.entry.path);
      } else if (action === 'copy') {
        navigator.clipboard?.writeText(ctx.entry.path).then(
          () => msg('路径已复制', 'success'),
          () => msg('复制失败', 'error'),
        );
      } else if (action === 'trash') {
        askConfirm(ctx.entry.isDir ? '文件夹' : '文件', ctx.entry.name, async () => {
          try {
            await invoke('trash_path', { path: ctx.entry.path });
            if (ctx.row === treeActiveRow) closePreview();
            const next = ctx.row.nextElementSibling;
            if (next && next.classList.contains('tree-children')) next.remove();
            ctx.row.remove();
            msg('已移到废纸篓', 'success');
          } catch (e) {
            msg('删除失败: ' + e, 'error');
          }
        });
      }
    };
  });
  document.addEventListener('click', (e) => { if (!e.target.closest('#tree-context-menu')) closeTreeCtx(); });
  document.addEventListener('scroll', closeTreeCtx, true);
  const savedTreeW = parseInt(localStorage.getItem('term-tree-width'), 10);
  if (savedTreeW >= 140 && savedTreeW <= 480) termEl.tree.style.width = savedTreeW + 'px';
  const treeHidden = localStorage.getItem('term-tree-hidden') === '1';
  termEl.tree.classList.toggle('hidden', treeHidden);
  termEl.treeBtn.classList.toggle('active', !treeHidden);
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

async function openTerminal(p, cmd) {
  try {
    await createSession({ cwd: p.localPath, name: p.name, autoCmd: cmd });
  } catch (e) {
    console.error('打开终端失败:', e);
    msg('打开终端失败: ' + (e.message || e), 'error');
  }
}

// 「打开终端」AI CLI 选择菜单（固定定位，锚到点击的按钮）
let launchMenuProject = null;
function openLaunchMenu(p, anchorEl) {
  launchMenuProject = p;
  const menu = el.launchMenu;
  menu.classList.add('active'); // 先显示以测量尺寸
  const r = anchorEl.getBoundingClientRect();
  let left = Math.max(8, r.right - menu.offsetWidth);
  let top = r.bottom + 4;
  if (top + menu.offsetHeight > window.innerHeight - 8) top = r.top - menu.offsetHeight - 4;
  menu.style.left = left + 'px';
  menu.style.top = top + 'px';
}
function closeLaunchMenu() {
  el.launchMenu.classList.remove('active');
  launchMenuProject = null;
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
  const parts = p.split(/[/\\]/);
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
  treeBtn: $('terminal-tree-btn'),
  themeBtn: $('terminal-theme-btn'),
  themeMenu: $('terminal-theme-menu'),
  maximizeBtn: $('terminal-maximize-btn'),
  collapseBtn: $('terminal-collapse-btn'),
  fab: $('terminal-fab'),
  fabBadge: $('terminal-fab-badge'),
  tree: $('terminal-tree'),
  treeBody: $('tree-body'),
  treeRootName: $('tree-root-name'),
  treeRefreshBtn: $('tree-refresh-btn'),
  treeSplitter: $('tree-splitter'),
  preview: $('file-preview'),
  previewName: $('file-preview-name'),
  previewCode: $('file-preview-code'),
  previewPre: $('file-preview-pre'),
  previewImage: $('file-preview-image'),
  previewImg: $('file-preview-img'),
  previewRich: $('file-preview-rich'),
  previewPdf: $('file-preview-pdf'),
  previewToggle: $('file-preview-toggle'),
  previewBody: $('file-preview-body'),
  previewInsert: $('file-preview-insert'),
  previewClose: $('file-preview-close'),
};

// 终端配色方案
const TERM_THEMES = {
  // 上一版原色（深蓝灰底）
  'classic': {
    name: '默认深色',
    theme: { background: '#14171e', foreground: '#e6eaf2', cursor: '#1677ff' },
  },
  // macOS 终端 Homebrew 描述文件：黑底绿字 + 标准 ANSI 调色板
  'homebrew': {
    name: 'Homebrew',
    theme: {
      background: '#000000', foreground: '#00ff00', cursor: '#23ff18', selectionBackground: '#0860a8',
      black: '#000000', red: '#990000', green: '#00a600', yellow: '#999900',
      blue: '#0000b2', magenta: '#b200b2', cyan: '#00a6b2', white: '#bfbfbf',
      brightBlack: '#666666', brightRed: '#e50000', brightGreen: '#00d900', brightYellow: '#e5e500',
      brightBlue: '#0000ff', brightMagenta: '#e500e5', brightCyan: '#00e5e5', brightWhite: '#e5e5e5',
    },
  },
};
let currentTheme = localStorage.getItem('term-theme') || 'classic';
if (!TERM_THEMES[currentTheme]) currentTheme = 'classic';

// 终端字号（⌘+ / ⌘- / ⌘0 调整，⌘+滚轮缩放，持久化）
const TERM_FONT_MIN = 8, TERM_FONT_MAX = 32, TERM_FONT_DEFAULT = 13;
let currentFontSize = parseInt(localStorage.getItem('term-fontsize'), 10) || TERM_FONT_DEFAULT;
if (currentFontSize < TERM_FONT_MIN || currentFontSize > TERM_FONT_MAX) currentFontSize = TERM_FONT_DEFAULT;

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

  // 拖拽文件/文件夹到终端面板 → 把路径写入当前会话（同 macOS 终端）
  const overDock = (pos) => {
    if (!pos || !termEl.dock.classList.contains('active')) return false;
    const dpr = window.devicePixelRatio || 1;
    const x = pos.x / dpr, y = pos.y / dpr;
    const r = termEl.dock.getBoundingClientRect();
    return x >= r.left && x <= r.right && y >= r.top && y <= r.bottom;
  };
  await listen('tauri://drag-over', e => {
    termEl.dock.classList.toggle('drag-target', overDock(e.payload && e.payload.position));
  });
  await listen('tauri://drag-leave', () => {
    termEl.dock.classList.remove('drag-target');
  });
  await listen('tauri://drag-drop', e => {
    termEl.dock.classList.remove('drag-target');
    const p = e.payload || {};
    if (!overDock(p.position) || !activeSession) return;
    const paths = (p.paths || []).filter(Boolean);
    if (!paths.length) return;
    const data = paths.map(shellQuotePath).join(' ') + ' ';
    invoke('terminal_write', { id: activeSession, data }).catch(() => {});
    sessions.get(activeSession)?.term.focus();
  });
}

const IS_WINDOWS = navigator.userAgent.includes('Windows');

// shell 路径转义。Windows(PowerShell/cmd)用双引号；Unix(bash/zsh)用单引号
function shellQuotePath(p) {
  if (IS_WINDOWS) {
    // 含空格或 shell 元字符时双引号包裹（Windows 路径几乎不含 " / $ / 反引号）
    return /[\s&()^%!,;'`$]/.test(p) ? `"${p}"` : p;
  }
  if (/^[A-Za-z0-9_@%+=:,./~-]+$/.test(p)) return p;
  return "'" + p.replace(/'/g, "'\\''") + "'";
}

// 应用配色方案：更新所有已开会话 + 终端面板背景，并持久化
function setTermTheme(key) {
  if (!TERM_THEMES[key]) return;
  currentTheme = key;
  localStorage.setItem('term-theme', key);
  const t = TERM_THEMES[key].theme;
  sessions.forEach(s => { s.term.options.theme = t; });
  termEl.bodies.style.background = t.background;
  renderThemeMenu();
}

function renderThemeMenu() {
  termEl.themeMenu.innerHTML = '';
  Object.entries(TERM_THEMES).forEach(([key, def]) => {
    const t = def.theme;
    const opt = document.createElement('div');
    opt.className = 'term-theme-opt' + (key === currentTheme ? ' active' : '');
    // 无完整 ANSI 调色板的主题（如默认深色）回退到 前景/光标 色，色卡不留空
    const sw = [t.background, t.red || t.foreground, t.green || t.cursor, t.blue || t.cursor, t.yellow || t.foreground];
    opt.innerHTML =
      `<span class="term-theme-swatch">` +
      sw.map(c => `<i style="background:${c}"></i>`).join('') +
      `</span>` +
      `<span class="term-theme-label">${esc(def.name)}</span>` +
      `<svg class="term-theme-check" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><path d="M5 13l4 4L19 7"/></svg>`;
    opt.onclick = () => { setTermTheme(key); closeThemeMenu(); };
    termEl.themeMenu.appendChild(opt);
  });
}

function openThemeMenu() { renderThemeMenu(); termEl.themeMenu.classList.add('active'); }
function closeThemeMenu() { termEl.themeMenu.classList.remove('active'); }

// 调整终端字号：更新所有会话 + 重新 fit（行列数随字号变），并持久化
function setTermFontSize(size) {
  size = Math.max(TERM_FONT_MIN, Math.min(TERM_FONT_MAX, size));
  if (size === currentFontSize) return;
  currentFontSize = size;
  localStorage.setItem('term-fontsize', String(size));
  sessions.forEach((s, id) => { s.term.options.fontSize = size; fitSession(id); });
}

// ===== 文件树 + 内容预览 =====

let treeRoot = null;      // 当前树根（活动会话的 cwd）
let treeActiveRow = null; // 当前选中的文件行

const TREE_ICONS = {
  folder: '<svg class="tree-icon" viewBox="0 0 24 24" fill="none" stroke="#7aa2cf" stroke-width="1.8"><path d="M3 7a2 2 0 012-2h4l2 2h8a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2z"/></svg>',
  code: '<svg class="tree-icon" viewBox="0 0 24 24" fill="none" stroke="#98c379" stroke-width="1.9"><path d="M8 9l-3 3 3 3M16 9l3 3-3 3M13 7l-2 10"/></svg>',
  config: '<svg class="tree-icon" viewBox="0 0 24 24" fill="none" stroke="#e5c07b" stroke-width="1.8"><path d="M4 7h8M17 7h3M4 17h3M12 17h8"/><circle cx="15" cy="7" r="2"/><circle cx="9" cy="17" r="2"/></svg>',
  doc: '<svg class="tree-icon" viewBox="0 0 24 24" fill="none" stroke="#61afef" stroke-width="1.7"><path d="M6 3h8l4 4v14H6z"/><path d="M14 3v4h4M9 13h6M9 16h6"/></svg>',
  image: '<svg class="tree-icon" viewBox="0 0 24 24" fill="none" stroke="#c678dd" stroke-width="1.7"><rect x="3" y="4" width="18" height="16" rx="2"/><circle cx="9" cy="10" r="1.6"/><path d="M5 19l5-5 4 4 2-2 3 3"/></svg>',
  file: '<svg class="tree-icon" viewBox="0 0 24 24" fill="none" stroke="#8b94a4" stroke-width="1.7"><path d="M6 3h8l4 4v14H6z"/><path d="M14 3v4h4"/></svg>',
};

// 扩展名 → highlight.js 语言；返回 null 走自动识别。仅在该语言已注册时才用。
const HLJS_EXT = {
  js: 'javascript', mjs: 'javascript', cjs: 'javascript', jsx: 'javascript',
  ts: 'typescript', tsx: 'typescript', go: 'go', rs: 'rust', py: 'python',
  rb: 'ruby', java: 'java', kt: 'kotlin', c: 'c', h: 'c', cpp: 'cpp', cc: 'cpp',
  cs: 'csharp', php: 'php', swift: 'swift', sh: 'bash', bash: 'bash', zsh: 'bash',
  lua: 'lua', sql: 'sql', html: 'xml', xml: 'xml', vue: 'xml', css: 'css',
  scss: 'scss', less: 'less', json: 'json', yaml: 'yaml', yml: 'yaml',
  toml: 'ini', ini: 'ini', conf: 'ini', md: 'markdown', markdown: 'markdown',
  dockerfile: 'dockerfile', makefile: 'makefile',
};
function hljsLangFor(name) {
  const lower = name.toLowerCase();
  let lang = HLJS_EXT[lower] || HLJS_EXT[lower.split('.').pop() || ''];
  if (lang && window.hljs && window.hljs.getLanguage(lang)) return lang;
  return null;
}

function fileIconKey(name) {
  const ext = (name.split('.').pop() || '').toLowerCase();
  if (/^(js|mjs|cjs|ts|tsx|jsx|vue|go|rs|py|rb|java|kt|c|h|hpp|cpp|cc|cs|php|swift|sh|bash|zsh|lua|sql|html|css|scss)$/.test(ext)) return 'code';
  if (/^(json|ya?ml|toml|ini|env|conf|cfg|lock|xml|gradle|properties)$/.test(ext)) return 'config';
  if (/^(md|markdown|txt|rst|adoc|log|pdf|csv|tsv)$/.test(ext)) return 'doc';
  if (/^(png|jpe?g|gif|webp|bmp|ico|svg|avif)$/.test(ext)) return 'image';
  return 'file';
}

function makeTreeRow(entry, depth) {
  const row = document.createElement('div');
  row.className = 'tree-row';
  row.style.paddingLeft = (8 + depth * 14) + 'px';
  const chevron = `<svg class="tree-chevron${entry.isDir ? '' : ' leaf'}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.6"><path d="M9 6l6 6-6 6"/></svg>`;
  const icon = entry.isDir ? TREE_ICONS.folder : TREE_ICONS[fileIconKey(entry.name)];
  row.innerHTML = chevron + icon + `<span class="tree-name">${esc(entry.name)}</span>`;

  // 拖入终端 + 右键菜单（文件/文件夹通用）
  row.addEventListener('mousedown', (e) => startTreeDragWatch(entry, e));
  row.addEventListener('contextmenu', (e) => openTreeCtx(entry, row, e));

  if (!entry.isDir) {
    let clickTimer = null;
    // 单击=预览，双击=插入路径。延时去抖：双击时取消单击的预览
    row.onclick = () => {
      if (treeDragSuppressClick) { treeDragSuppressClick = false; return; }
      if (clickTimer) return;
      clickTimer = setTimeout(() => {
        clickTimer = null;
        if (treeActiveRow) treeActiveRow.classList.remove('active');
        treeActiveRow = row;
        row.classList.add('active');
        openPreview(entry.path, entry.name);
      }, 220);
    };
    row.ondblclick = () => {
      if (clickTimer) { clearTimeout(clickTimer); clickTimer = null; }
      insertPathToTerminal(entry.path);
    };
    return [row];
  }

  const childWrap = document.createElement('div');
  childWrap.className = 'tree-children';
  childWrap.style.display = 'none';
  let loaded = false;
  row.onclick = async () => {
    if (treeDragSuppressClick) { treeDragSuppressClick = false; return; }
    const expanded = row.classList.toggle('expanded');
    childWrap.style.display = expanded ? '' : 'none';
    if (expanded && !loaded) {
      loaded = true;
      childWrap.innerHTML = '<div class="tree-loading">…</div>';
      try {
        const items = await invoke('list_dir', { path: entry.path });
        childWrap.innerHTML = '';
        if (!items.length) childWrap.innerHTML = '<div class="tree-empty">空目录</div>';
        else items.forEach(it => makeTreeRow(it, depth + 1).forEach(n => childWrap.appendChild(n)));
      } catch (e) {
        childWrap.innerHTML = `<div class="tree-empty">${esc(String(e))}</div>`;
        loaded = false;
      }
    }
  };
  return [row, childWrap];
}

async function renderTree(cwd) {
  treeRoot = cwd || null;
  treeActiveRow = null;
  if (!cwd) {
    termEl.treeRootName.textContent = '无目录';
    termEl.treeRootName.title = '';
    termEl.treeBody.innerHTML = '<div class="tree-empty">此会话无项目根目录</div>';
    return;
  }
  termEl.treeRootName.textContent = cwd.replace(/[/\\]+$/, '').split(/[/\\]/).pop() || cwd;
  termEl.treeRootName.title = cwd;
  termEl.treeBody.innerHTML = '<div class="tree-loading">加载中…</div>';
  try {
    const items = await invoke('list_dir', { path: cwd });
    termEl.treeBody.innerHTML = '';
    if (!items.length) { termEl.treeBody.innerHTML = '<div class="tree-empty">空目录</div>'; return; }
    items.forEach(it => makeTreeRow(it, 0).forEach(n => termEl.treeBody.appendChild(n)));
  } catch (e) {
    termEl.treeBody.innerHTML = `<div class="tree-empty">${esc(String(e))}</div>`;
  }
}

function insertPathToTerminal(path) {
  if (!activeSession) return;
  invoke('terminal_write', { id: activeSession, data: shellQuotePath(path) + ' ' }).catch(() => {});
  sessions.get(activeSession)?.term.focus();
}

function isImageFile(name) { return /\.(png|jpe?g|gif|webp|bmp|ico|svg|avif)$/i.test(name); }
function isPdfFile(name) { return /\.pdf$/i.test(name); }
function isMarkdownFile(name) { return /\.(md|markdown)$/i.test(name); }
function isCsvFile(name) { return /\.(csv|tsv)$/i.test(name); }

let previewPdfUrl = null;            // 当前 PDF 的 object URL（需手动 revoke）
let previewRichState = null;         // { kind:'md'|'csv', content, name } 供源码/渲染切换

// 在四个视图(pre/image/rich/pdf)间切换显示
function showPreviewView(which) {
  termEl.previewPre.style.display = which === 'text' ? '' : 'none';
  termEl.previewImage.classList.toggle('active', which === 'image');
  termEl.previewRich.classList.toggle('active', which === 'rich');
  termEl.previewPdf.classList.toggle('active', which === 'pdf');
}

function revokePreviewPdf() {
  if (previewPdfUrl) { URL.revokeObjectURL(previewPdfUrl); previewPdfUrl = null; }
  termEl.previewPdf.removeAttribute('src');
}

function renderTextPreview(content, name, truncatedNote) {
  showPreviewView('text');
  termEl.previewCode.className = 'hljs';
  termEl.previewCode.removeAttribute('data-highlighted');
  termEl.previewCode.textContent = content;
  const lang = hljsLangFor(name);
  termEl.previewCode.className = lang ? `hljs language-${lang}` : 'hljs';
  try { window.hljs.highlightElement(termEl.previewCode); } catch (e) {}
  if (truncatedNote) {
    const note = document.createElement('div');
    note.className = 'file-preview-truncated';
    note.textContent = truncatedNote;
    termEl.preview.appendChild(note);
  }
}

// CSV/TSV 解析（处理引号包裹的字段）
function parseCSV(text, delim) {
  const rows = [];
  let row = [], field = '', inQ = false;
  for (let i = 0; i < text.length; i++) {
    const c = text[i];
    if (inQ) {
      if (c === '"') { if (text[i + 1] === '"') { field += '"'; i++; } else inQ = false; }
      else field += c;
    } else if (c === '"') inQ = true;
    else if (c === delim) { row.push(field); field = ''; }
    else if (c === '\n') { row.push(field); rows.push(row); row = []; field = ''; }
    else if (c !== '\r') field += c;
  }
  if (field.length || row.length) { row.push(field); rows.push(row); }
  return rows;
}

function renderCsvRich(content, name) {
  const rows = parseCSV(content, /\.tsv$/i.test(name) ? '\t' : ',');
  const MAXROW = 1000;
  const shown = rows.slice(0, MAXROW);
  let html = '<table class="csv-table">';
  shown.forEach((r, i) => {
    const tag = i === 0 ? 'th' : 'td';
    html += '<tr>' + r.map(c => `<${tag}>${esc(c)}</${tag}>`).join('') + '</tr>';
  });
  html += '</table>';
  if (rows.length > MAXROW) html += `<div class="csv-note">仅显示前 ${MAXROW} 行(共 ${rows.length} 行)</div>`;
  termEl.previewRich.innerHTML = html;
  showPreviewView('rich');
}

function renderRich() {
  if (!previewRichState) return;
  const { kind, content, name } = previewRichState;
  if (kind === 'md') {
    termEl.previewRich.className = 'file-preview-rich markdown-body';
    // marked 默认透传原始 HTML 且不净化 → 必须 DOMPurify 过滤，防止恶意 .md 在应用内执行脚本
    const raw = window.marked ? window.marked.parse(content) : esc(content);
    termEl.previewRich.innerHTML = window.DOMPurify ? window.DOMPurify.sanitize(raw) : esc(content);
    showPreviewView('rich');
  } else {
    termEl.previewRich.className = 'file-preview-rich';
    renderCsvRich(content, name);
  }
}

async function openPreview(path, name) {
  termEl.preview.querySelector('.file-preview-truncated')?.remove();
  termEl.previewName.textContent = name;
  termEl.previewName.title = path;
  termEl.previewInsert.dataset.path = path;
  termEl.preview.classList.add('active');
  revokePreviewPdf();
  previewRichState = null;
  termEl.previewToggle.style.display = 'none';
  termEl.previewToggle.classList.remove('active');

  // 图片
  if (isImageFile(name)) {
    showPreviewView('image');
    termEl.previewImg.removeAttribute('src');
    termEl.previewImg.alt = '加载中…';
    try {
      termEl.previewImg.src = await invoke('read_image', { path });
      termEl.previewImg.alt = name;
    } catch (e) { renderTextPreview(String(e), name); }
    return;
  }

  // PDF
  if (isPdfFile(name)) {
    showPreviewView('pdf');
    try {
      const b64 = await invoke('read_binary_base64', { path });
      const blob = new Blob([b64ToBytes(b64)], { type: 'application/pdf' });
      previewPdfUrl = URL.createObjectURL(blob);
      termEl.previewPdf.src = previewPdfUrl;
    } catch (e) { renderTextPreview(String(e), name); }
    return;
  }

  // Markdown / CSV：默认渲染，提供源码/渲染切换
  if (isMarkdownFile(name) || isCsvFile(name)) {
    try {
      const res = await invoke('read_file', { path });
      previewRichState = { kind: isMarkdownFile(name) ? 'md' : 'csv', content: res.content, name };
      termEl.previewToggle.style.display = '';
      termEl.previewToggle.classList.add('active'); // active=渲染态
      renderRich();
      return;
    } catch (e) { renderTextPreview(String(e), name); return; }
  }

  // 普通文本
  try {
    const res = await invoke('read_file', { path });
    renderTextPreview(
      res.content, name,
      res.truncated ? `文件超过 1MB，仅显示前 1MB（共 ${(res.size / 1048576).toFixed(1)} MB）` : null,
    );
  } catch (e) { renderTextPreview(String(e), name); }
}

// 源码 / 渲染切换（仅 md/csv）
function togglePreviewMode() {
  if (!previewRichState) return;
  const toRich = !termEl.previewToggle.classList.contains('active');
  termEl.previewToggle.classList.toggle('active', toRich);
  if (toRich) renderRich();
  else renderTextPreview(previewRichState.content, previewRichState.name, null);
}

function closePreview() {
  termEl.preview.classList.remove('active');
  revokePreviewPdf();
  previewRichState = null;
  if (treeActiveRow) { treeActiveRow.classList.remove('active'); treeActiveRow = null; }
}

function toggleTree() {
  const hidden = termEl.tree.classList.toggle('hidden');
  termEl.treeBtn.classList.toggle('active', !hidden);
  localStorage.setItem('term-tree-hidden', hidden ? '1' : '0');
  if (!hidden && treeRoot === null && activeSession) renderTree(sessions.get(activeSession).cwd);
  if (activeSession) requestAnimationFrame(() => fitSession(activeSession));
}

function setupTreeSplitter() {
  let startX = 0, startW = 0;
  const onMove = (e) => {
    const w = Math.min(Math.max(startW + (e.clientX - startX), 140), 480);
    termEl.tree.style.width = w + 'px';
    if (activeSession) fitSession(activeSession);
  };
  const onUp = () => {
    document.removeEventListener('mousemove', onMove);
    document.removeEventListener('mouseup', onUp);
    document.body.style.userSelect = '';
    localStorage.setItem('term-tree-width', String(termEl.tree.offsetWidth));
  };
  termEl.treeSplitter.addEventListener('mousedown', (e) => {
    startX = e.clientX;
    startW = termEl.tree.offsetWidth;
    document.body.style.userSelect = 'none';
    document.addEventListener('mousemove', onMove);
    document.addEventListener('mouseup', onUp);
  });
}

// ===== 树项拖入终端（自实现鼠标拖拽，绕开 Tauri 原生 drag-drop 对 HTML5 DnD 的干扰）=====
let treeDrag = null;
let treeDragSuppressClick = false;

function isOverTerminalArea(x, y) {
  if (!termEl.dock.classList.contains('active')) return false;
  const r = termEl.bodies.getBoundingClientRect();
  return x >= r.left && x <= r.right && y >= r.top && y <= r.bottom;
}

function startTreeDragWatch(entry, e) {
  if (e.button !== 0) return; // 仅左键
  treeDragSuppressClick = false;
  treeDrag = { entry, x: e.clientX, y: e.clientY, started: false, ghost: null };
}

function setupTreeDrag() {
  document.addEventListener('mousemove', (e) => {
    if (!treeDrag) return;
    if (!treeDrag.started) {
      if (Math.hypot(e.clientX - treeDrag.x, e.clientY - treeDrag.y) < 5) return; // 阈值，区分点击
      treeDrag.started = true;
      const g = document.createElement('div');
      g.className = 'tree-drag-ghost';
      g.textContent = treeDrag.entry.name;
      document.body.appendChild(g);
      treeDrag.ghost = g;
      document.body.style.userSelect = 'none';
    }
    treeDrag.ghost.style.left = (e.clientX + 12) + 'px';
    treeDrag.ghost.style.top = (e.clientY + 14) + 'px';
    termEl.dock.classList.toggle('drag-target', isOverTerminalArea(e.clientX, e.clientY));
  });
  document.addEventListener('mouseup', (e) => {
    if (!treeDrag) return;
    const d = treeDrag;
    treeDrag = null;
    if (d.ghost) d.ghost.remove();
    document.body.style.userSelect = '';
    termEl.dock.classList.remove('drag-target');
    if (d.started) {
      treeDragSuppressClick = true; // 抑制随后的 click（预览/展开）
      if (isOverTerminalArea(e.clientX, e.clientY) && activeSession) {
        insertPathToTerminal(d.entry.path);
      }
    }
  });
}

// ===== 文件树右键菜单：插入路径 / 复制路径 / 移到废纸篓 =====
let treeCtx = null;
function parentDir(p) {
  const norm = p.replace(/[/\\]+$/, '');
  const idx = Math.max(norm.lastIndexOf('/'), norm.lastIndexOf('\\'));
  return idx > 0 ? norm.slice(0, idx) : norm;
}

function openTreeCtx(entry, row, e) {
  e.preventDefault();
  treeCtx = { entry, row };
  $('ctx-open-label').textContent = entry.isDir ? '打开文件夹' : '打开所在文件夹';
  const menu = el.treeCtxMenu;
  menu.classList.add('active');
  menu.style.left = Math.min(e.clientX, window.innerWidth - menu.offsetWidth - 8) + 'px';
  menu.style.top = Math.min(e.clientY, window.innerHeight - menu.offsetHeight - 8) + 'px';
}
function closeTreeCtx() {
  el.treeCtxMenu.classList.remove('active');
  treeCtx = null;
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
  const rootChanged = s.cwd !== treeRoot;
  activeSession = id;
  sessions.forEach((other, oid) => {
    const on = oid === id;
    other.tabEl.classList.toggle('active', on);
    other.bodyEl.classList.toggle('active', on);
  });
  closePreview();
  if (rootChanged) renderTree(s.cwd);
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

async function createSession({ cwd = '', name = '', autoCmd = '' }) {
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
  const toolBadge = autoCmd
    ? `<span class="term-tab-tool tool-${esc(autoCmd)}">${esc(autoCmd)}</span>`
    : '';
  tabEl.innerHTML =
    `<span class="term-tab-dot"></span>` +
    `<span class="term-tab-name" title="${esc(label)}">${esc(label)}</span>` +
    toolBadge +
    `<span class="term-tab-close" title="关闭"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 18L18 6M6 6l12 12"/></svg></span>`;
  termEl.tabs.appendChild(tabEl);
  tabEl.onclick = (ev) => {
    if (ev.target.closest('.term-tab-close')) { closeSession(id); return; }
    activateSession(id);
  };

  const term = new window.Terminal({
    fontSize: currentFontSize,
    fontFamily: '"JetBrains Mono", Menlo, Monaco, "Courier New", monospace',
    cursorBlink: true,
    scrollback: 5000,
    theme: TERM_THEMES[currentTheme].theme,
  });
  const fit = new window.FitAddon.FitAddon();
  term.loadAddon(fit);
  term.open(bodyEl);
  term.onData(d => invoke('terminal_write', { id, data: d }).catch(() => {}));

  sessions.set(id, { term, fit, tabEl, bodyEl, name: label, status: 'running', cwd });

  openDock();
  activateSession(id);
  requestAnimationFrame(() => fitSession(id));
  updateFabBadge();

  try {
    await invoke('terminal_create', { id, cwd, cols: term.cols || 80, rows: term.rows || 24 });
    fitSession(id);
    if (autoCmd) {
      setTimeout(() => invoke('terminal_write', { id, data: autoCmd + '\r' }).catch(() => {}), 400);
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
