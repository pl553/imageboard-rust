const API = '/api/v1';

// ---- state ----
let token = localStorage.getItem('token') || null;
let currentBoard = null;       // slug string
let currentThread = null;      // { slug, thread_number }
let currentPage = 1;
let allBoards = [];

// ---- startup ----
document.addEventListener('DOMContentLoaded', async () => {
    updateAuthUI();
    await loadBoards();
    showHome();
});

// ---- auth ----
function updateAuthUI() {
    const isAdmin = !!token;
    document.getElementById('admin-indicator').style.display = isAdmin ? 'flex' : 'none';
    document.getElementById('login-btn').style.display = isAdmin ? 'none' : 'inline-block';
}

function showLoginModal() {
    document.getElementById('login-error').style.display = 'none';
    document.getElementById('login-username').value = '';
    document.getElementById('login-password').value = '';
    showModal('login-modal');
}

async function doLogin() {
    const username = document.getElementById('login-username').value.trim();
    const password = document.getElementById('login-password').value;

    const res = await api('POST', '/auth/login', { username, password });
    if (res.ok) {
        token = res.data.token;
        localStorage.setItem('token', token);
        closeModal('login-modal');
        updateAuthUI();
        // reload current view so admin controls appear
        if (currentThread) {
            showThread(currentThread.slug, currentThread.thread_number);
        } else if (currentBoard) {
            showBoard(currentBoard, currentPage);
        } else {
            showHome();
        }
    } else {
        showError('login-error', res.data?.error || 'login failed');
    }
}

function logout() {
    token = null;
    localStorage.removeItem('token');
    updateAuthUI();
    if (currentThread) {
        showThread(currentThread.slug, currentThread.thread_number);
    } else if (currentBoard) {
        showBoard(currentBoard, currentPage);
    } else {
        showHome();
    }
}

// ---- boards ----
async function loadBoards() {
    const res = await api('GET', '/boards');
    if (res.ok) {
        allBoards = res.data;
        renderBoardNav();
    }
}

function renderBoardNav() {
    const nav = document.getElementById('board-nav');
    nav.innerHTML = allBoards.map(b =>
        `<a href="#" onclick="showBoard('${b.slug}',1);return false">[${b.slug}]</a>`
    ).join(' ');
}

function showHome() {
    currentBoard = null;
    currentThread = null;
    const app = document.getElementById('app');

    let adminHtml = token
        ? `<div style="margin-bottom:12px">
               <button onclick="showCreateBoardModal()" class="btn-primary">+ create board</button>
               <button onclick="showModal('change-password-modal')" class="btn-secondary" style="margin-left:8px">change password</button>
           </div>`
        : '';

    let boardsHtml = allBoards.length === 0
        ? '<p class="empty">no boards yet</p>'
        : `<div class="board-grid">${allBoards.map(boardCard).join('')}</div>`;

    app.innerHTML = `
        <h2 style="margin-bottom:12px">Boards</h2>
        ${adminHtml}
        ${boardsHtml}
    `;
}

function boardCard(b) {
    const deleteBtn = token
        ? `<button class="btn-danger" onclick="doDeleteBoard('${b.slug}')">delete</button>`
        : '';
    return `
        <div class="board-card">
            <h3><a href="#" onclick="showBoard('${b.slug}',1);return false">
                <span class="slug">/${b.slug}/</span> ${escHtml(b.name)}
            </a></h3>
            ${b.description ? `<div class="desc">${escHtml(b.description)}</div>` : ''}
            <div class="meta">${b.thread_count ?? 0} threads</div>
            <div class="card-actions">${deleteBtn}</div>
        </div>
    `;
}

function showCreateBoardModal() {
    document.getElementById('create-board-error').style.display = 'none';
    document.getElementById('new-board-slug').value = '';
    document.getElementById('new-board-name').value = '';
    document.getElementById('new-board-desc').value = '';
    showModal('create-board-modal');
}

async function doCreateBoard() {
    const slug = document.getElementById('new-board-slug').value.trim();
    const name = document.getElementById('new-board-name').value.trim();
    const description = document.getElementById('new-board-desc').value.trim() || undefined;

    if (!slug || !name) {
        showError('create-board-error', 'slug and name are required');
        return;
    }

    const res = await api('POST', '/boards', { slug, name, description }, true);
    if (res.ok) {
        closeModal('create-board-modal');
        await loadBoards();
        showHome();
    } else {
        showError('create-board-error', res.data?.error || 'failed to create board');
    }
}

async function doDeleteBoard(slug) {
    if (!confirm(`delete board /${slug}/ and all its threads?`)) return;
    const res = await api('DELETE', `/boards/${slug}`, null, true);
    if (res.ok) {
        await loadBoards();
        showHome();
    } else {
        alert(res.data?.error || 'failed to delete board');
    }
}

// ---- threads (board view) ----
async function showBoard(slug, page) {
    currentBoard = slug;
    currentThread = null;
    currentPage = page;

    const app = document.getElementById('app');
    app.innerHTML = `<div class="loading">loading...</div>`;

    const res = await api('GET', `/boards/${slug}/threads?page=${page}&limit=10&preview_posts=3`);
    if (!res.ok) {
        app.innerHTML = `<div class="error-msg">${res.data?.error || 'failed to load board'}</div>`;
        return;
    }

    const { threads, pagination } = res.data;

    const boardRes = await api('GET', `/boards/${slug}`);
    const board = boardRes.ok ? boardRes.data : { slug, name: slug };

    let adminActions = token
        ? `<div class="board-actions">
               <button onclick="showCreateThreadModal()" class="btn-primary">+ new thread</button>
           </div>`
        : `<div class="board-actions">
               <button onclick="showCreateThreadModal()" class="btn-primary">+ new thread</button>
           </div>`;

    let threadsHtml = threads.length === 0
        ? '<p class="empty">no threads yet</p>'
        : threads.map(threadPreviewHtml).join('');

    app.innerHTML = `
        <div class="board-header">
            <span class="slug">/${board.slug}/</span>
            <h2>${escHtml(board.name)}</h2>
            ${adminActions}
        </div>
        ${threadsHtml}
        ${paginationHtml(pagination, slug)}
    `;
}

function threadPreviewHtml(t) {
    const op = t.op_post;
    const deleteBtn = token
        ? `<button class="btn-danger" onclick="doDeleteThread('${t.board_slug}',${t.post_number})">delete thread</button>`
        : '';

    const repliesHtml = t.last_posts && t.last_posts.length > 0
        ? `<div class="replies-preview">${t.last_posts.map(replyPostHtml).join('')}</div>`
        : '';

    const omitted = t.omitted_posts > 0
        ? `<span class="omitted">${t.omitted_posts} posts omitted</span>`
        : '';

    return `
        <div class="thread-preview">
            <div class="op-post">
                ${postImageHtml(op, 'thumb')}
                <div class="post-body">
                    ${postHeaderHtml(op)}
                    <div class="post-text">${escHtml(op.text)}</div>
                </div>
            </div>
            <div class="thread-footer">
                <a href="#" onclick="showThread('${t.board_slug}',${t.post_number});return false"
                   class="btn-primary" style="font-size:12px;padding:3px 10px">
                   open thread (${t.post_count} replies)
                </a>
                ${omitted}
                ${deleteBtn}
            </div>
            ${repliesHtml}
        </div>
    `;
}

function replyPostHtml(p) {
    return `
        <div class="reply-post">
            ${postImageHtml(p, 'thumb-small')}
            <div class="post-body">
                ${postHeaderHtml(p)}
                <div class="post-text">${escHtml(p.text)}</div>
            </div>
        </div>
    `;
}

function paginationHtml(pagination, slug) {
    if (pagination.total_pages <= 1) return '';
    let buttons = '';
    for (let i = 1; i <= pagination.total_pages; i++) {
        const active = i === pagination.page ? 'active' : '';
        buttons += `<button class="${active}" onclick="showBoard('${slug}',${i})">${i}</button>`;
    }
    return `<div class="pagination">${buttons}</div>`;
}

// ---- thread detail ----
async function showThread(slug, threadNumber) {
    currentBoard = slug;
    currentThread = { slug, thread_number: threadNumber };

    const app = document.getElementById('app');
    app.innerHTML = `<div class="loading">loading...</div>`;

    const res = await api('GET', `/boards/${slug}/threads/${threadNumber}`);
    if (!res.ok) {
        app.innerHTML = `<div class="error-msg">${res.data?.error || 'thread not found'}</div>`;
        return;
    }

    const t = res.data;
    const op = t.op_post;

    const opDeleteBtn = token
        ? `<button class="btn-danger" onclick="doDeleteThread('${slug}',${threadNumber})">delete thread</button>`
        : '';

    const repliesHtml = (t.posts || []).map(p => replyFullHtml(p, slug)).join('');

    app.innerHTML = `
        <div class="thread-detail-header">
            <a href="#" onclick="showBoard('${slug}',1);return false" class="back-link">← /${slug}/</a>
            <h2>Thread #${threadNumber}</h2>
        </div>
        <div class="op-full">
            ${postImageHtml(op, 'full')}
            <div class="post-body" style="flex:1">
                ${postHeaderHtml(op)}
                <div class="post-text">${escHtml(op.text)}</div>
                <div style="margin-top:8px;display:flex;gap:8px">
                    <button onclick="showReplyModal()" class="btn-primary" style="font-size:12px">reply</button>
                    ${opDeleteBtn}
                </div>
            </div>
        </div>
        <div class="replies-list">${repliesHtml}</div>
        <div style="margin-top:12px">
            <button onclick="showReplyModal()" class="btn-primary">post reply</button>
        </div>
    `;
}

function replyFullHtml(p, slug) {
    const deleteBtn = token
        ? `<div class="reply-actions">
               <button class="btn-danger" onclick="doDeletePost('${slug}',${p.post_number})">del</button>
           </div>`
        : '';

    return `
        <div class="reply-full">
            ${postImageHtml(p, 'medium')}
            <div class="post-body" style="flex:1">
                ${postHeaderHtml(p)}
                <div class="post-text">${escHtml(p.text)}</div>
            </div>
            ${deleteBtn}
        </div>
    `;
}

// ---- create thread ----
function showCreateThreadModal() {
    document.getElementById('create-thread-error').style.display = 'none';
    document.getElementById('thread-name').value = '';
    document.getElementById('thread-text').value = '';
    document.getElementById('thread-image').value = '';
    showModal('create-thread-modal');
}

async function doCreateThread() {
    const name = document.getElementById('thread-name').value.trim() || 'Anonymous';
    const text = document.getElementById('thread-text').value.trim();
    const imageFile = document.getElementById('thread-image').files[0];

    if (!text) {
        showError('create-thread-error', 'text is required');
        return;
    }

    const form = new FormData();
    form.append('name', name);
    form.append('text', text);
    if (imageFile) form.append('image', imageFile);

    const res = await apiForm('POST', `/boards/${currentBoard}/threads`, form);
    if (res.ok) {
        closeModal('create-thread-modal');
        showThread(currentBoard, res.data.post_number);
    } else {
        showError('create-thread-error', res.data?.error || 'failed to create thread');
    }
}

// ---- create reply ----
function showReplyModal() {
    document.getElementById('reply-error').style.display = 'none';
    document.getElementById('reply-name').value = '';
    document.getElementById('reply-text').value = '';
    document.getElementById('reply-image').value = '';
    showModal('reply-modal');
}

async function doCreateReply() {
    const name = document.getElementById('reply-name').value.trim() || 'Anonymous';
    const text = document.getElementById('reply-text').value.trim();
    const imageFile = document.getElementById('reply-image').files[0];

    if (!text) {
        showError('reply-error', 'text is required');
        return;
    }

    const form = new FormData();
    form.append('name', name);
    form.append('text', text);
    if (imageFile) form.append('image', imageFile);

    const { slug, thread_number } = currentThread;
    const res = await apiForm('POST', `/boards/${slug}/threads/${thread_number}/posts`, form);
    if (res.ok) {
        closeModal('reply-modal');
        showThread(slug, thread_number);
    } else {
        showError('reply-error', res.data?.error || 'failed to post reply');
    }
}

// ---- delete thread / post ----
async function doDeleteThread(slug, threadNumber) {
    if (!confirm('delete this thread and all replies?')) return;
    const res = await api('DELETE', `/boards/${slug}/threads/${threadNumber}`, null, true);
    if (res.ok) {
        showBoard(slug, 1);
    } else {
        alert(res.data?.error || 'failed to delete thread');
    }
}

async function doDeletePost(slug, postNumber) {
    if (!confirm('delete this post?')) return;
    const res = await api('DELETE', `/boards/${slug}/posts/${postNumber}`, null, true);
    if (res.ok) {
        showThread(slug, currentThread.thread_number);
    } else {
        alert(res.data?.error || 'failed to delete post');
    }
}

// ---- change password ----
async function doChangePassword() {
    const current = document.getElementById('current-password').value;
    const next = document.getElementById('new-password').value;

    if (next.length < 8) {
        showError('change-password-error', 'new password must be at least 8 characters');
        return;
    }

    const res = await api('POST', '/auth/change-password', {
        current_password: current,
        new_password: next,
    }, true);

    if (res.ok) {
        closeModal('change-password-modal');
        alert('password changed');
    } else {
        showError('change-password-error', res.data?.error || 'failed');
    }
}

// ---- image helpers ----
function postImageHtml(post, size) {
    if (!post.image) return '';
    const thumbUrl = `/api/v1/images/thumb/${post.image.thumbnail_filename}`;
    const fullUrl = `/api/v1/images/${post.image.filename}`;
    const maxW = size === 'full' ? 200 : size === 'medium' ? 100 : size === 'thumb-small' ? 60 : 120;
    return `
        <div class="post-image-wrap">
            <img class="post-thumb"
                 src="${thumbUrl}"
                 style="max-width:${maxW}px;max-height:${maxW}px"
                 alt="${escHtml(post.image.original_name)}"
                 onclick="openLightbox('${fullUrl}')" />
        </div>
    `;
}

function openLightbox(url) {
    document.getElementById('lightbox-img').src = url;
    showModal('lightbox');
}

// ---- post header ----
function postHeaderHtml(p) {
    const date = new Date(p.created_at).toLocaleString();
    return `
        <div class="post-header">
            <span class="post-name">${escHtml(p.name)}</span>
            <span class="post-number">No.${p.post_number}</span>
            <span class="post-date">${date}</span>
        </div>
    `;
}

// ---- api helpers ----
async function api(method, path, body = null, auth = false) {
    const headers = { 'Content-Type': 'application/json' };
    if (auth && token) headers['Authorization'] = `Bearer ${token}`;

    try {
        const res = await fetch(API + path, {
            method,
            headers,
            body: body ? JSON.stringify(body) : undefined,
        });

        // 204 no content
        if (res.status === 204) return { ok: true, data: null };

        const data = await res.json().catch(() => ({}));
        return { ok: res.ok, status: res.status, data };
    } catch (e) {
        return { ok: false, status: 0, data: { error: 'network error' } };
    }
}

async function apiForm(method, path, formData) {
    const headers = {};
    if (token) headers['Authorization'] = `Bearer ${token}`;

    try {
        const res = await fetch(API + path, {
            method,
            headers,
            body: formData,
        });

        if (res.status === 204) return { ok: true, data: null };
        const data = await res.json().catch(() => ({}));
        return { ok: res.ok, status: res.status, data };
    } catch (e) {
        return { ok: false, status: 0, data: { error: 'network error' } };
    }
}

// ---- modal helpers ----
function showModal(id) { document.getElementById(id).style.display = 'flex'; }
function closeModal(id) { document.getElementById(id).style.display = 'none'; }

function showError(id, msg) {
    const el = document.getElementById(id);
    el.textContent = msg;
    el.style.display = 'block';
}

// close modals on backdrop click (except lightbox which handles its own)
document.addEventListener('click', e => {
    if (e.target.classList.contains('modal') && e.target.id !== 'lightbox') {
        e.target.style.display = 'none';
    }
});

// ---- utils ----
function escHtml(str) {
    if (!str) return '';
    return str
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;');
}
