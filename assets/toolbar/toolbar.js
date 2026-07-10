(() => {
  if (document.getElementById('mt-toolbar-btn')) return;
  if (window.self !== window.top) return;

  const API = '/__marmite__';
  const STORAGE_KEY = 'marmiteToolbar';
  const stored = JSON.parse(localStorage.getItem(STORAGE_KEY) || '{}');
  let panelOpen = !!stored.open;
  let metadata = null;
  let siteData = null;
  let isContentPage = false;
  let currentSlug = '';

  // Detect current slug from the page URL
  const path = window.location.pathname;
  const htmlMatch = path.match(/\/([^/]+)\.html$/);
  if (htmlMatch) {
    currentSlug = htmlMatch[1];
  } else if (path === '/' || path === '') {
    currentSlug = 'index';
  }

  const is404 = typeof window.__marmite_404_slug__ === 'string';

  // SVG icon for the toolbar button (a small wrench/gear)
  const ICON_SVG = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>`;

  // --- Utility functions ---

  function toast(msg, isError) {
    let el = document.querySelector('.mt-toast');
    if (!el) {
      el = document.createElement('div');
      el.className = 'mt-toast';
      document.body.appendChild(el);
    }
    el.textContent = msg;
    el.classList.toggle('mt-error', !!isError);
    el.classList.add('mt-show');
    setTimeout(() => el.classList.remove('mt-show'), 3000);
  }

  function confirm(title, message) {
    return new Promise((resolve) => {
      const overlay = document.createElement('div');
      overlay.className = 'mt-confirm-overlay';
      overlay.innerHTML = `
        <div class="mt-confirm-box">
          <h4>${title}</h4>
          <p>${message}</p>
          <div class="mt-confirm-actions">
            <button class="mt-btn" id="mt-confirm-no">Cancel</button>
            <button class="mt-btn mt-btn-danger" id="mt-confirm-yes">Confirm</button>
          </div>
        </div>`;
      document.body.appendChild(overlay);
      overlay.querySelector('#mt-confirm-yes').onclick = () => { overlay.remove(); resolve(true); };
      overlay.querySelector('#mt-confirm-no').onclick = () => { overlay.remove(); resolve(false); };
    });
  }

  async function api(method, path, body) {
    const opts = { method, headers: {} };
    if (body) {
      opts.headers['Content-Type'] = 'application/json';
      opts.body = JSON.stringify(body);
    }
    const resp = await fetch(`${API}${path}`, opts);
    const data = await resp.json();
    if (!resp.ok) throw new Error(data.error || `HTTP ${resp.status}`);
    return data;
  }

  // Intercept live-reload to prevent unwanted page refresh during mutations.
  //
  // Flow: call holdReload() BEFORE the API call that writes files. This suppresses
  // any live-reload that fires while the await is in-flight. After the API call
  // resolves, call redirectAfterRebuild(slug) or reloadAfterRebuild() to set the
  // final destination. If live-reload already fired while held, navigates immediately.
  // If not, waits for the next live-reload (or falls back to polling after 8s).
  const _origReload = window.location.reload.bind(window.location);
  let _held = false;
  let _reloadFiredWhileHeld = false;
  let _pendingPoll = null;

  function holdReload() {
    _held = true;
    _reloadFiredWhileHeld = false;
    window.location.reload = () => { _reloadFiredWhileHeld = true; };
  }

  function _navigate(target) {
    _held = false;
    _reloadFiredWhileHeld = false;
    if (_pendingPoll) { clearInterval(_pendingPoll); clearTimeout(_pendingPoll); _pendingPoll = null; }
    window.location.reload = _origReload;
    if (target === window.location.href || target === null) {
      _origReload();
    } else {
      window.location.href = target;
    }
  }

  function _waitAndNavigate(target) {
    toast('Rebuilding site...');
    // If live-reload already fired while we were awaiting the API call, go now
    if (_reloadFiredWhileHeld) { _navigate(target); return; }
    // Otherwise wait for the next live-reload
    window.location.reload = () => _navigate(target);
    // Fallback for --serve without --watch: poll target URL or plain timeout
    if (_pendingPoll) clearInterval(_pendingPoll);
    if (target) {
      let attempts = 0;
      _pendingPoll = setInterval(async () => {
        attempts++;
        if (!_held) { clearInterval(_pendingPoll); _pendingPoll = null; return; }
        if (attempts > 16) { _navigate(target); return; }
        try {
          const resp = await fetch(target, { cache: 'no-store' });
          if (resp.ok) _navigate(target);
        } catch (e) { /* keep polling */ }
      }, 500);
    } else {
      // Reload case: no URL to poll, just timeout
      _pendingPoll = setTimeout(() => { _navigate(target); }, 8000);
    }
  }

  function redirectAfterRebuild(slug) {
    if (!_held) holdReload();
    _waitAndNavigate('/' + slug + '.html');
  }

  function reloadAfterRebuild() {
    if (!_held) holdReload();
    _waitAndNavigate(null);
  }

  // --- Autocomplete helper ---

  function createAutocomplete(input, items, onSelect) {
    const wrap = document.createElement('div');
    wrap.className = 'mt-autocomplete-wrap';
    input.parentNode.insertBefore(wrap, input);
    wrap.appendChild(input);

    const list = document.createElement('div');
    list.className = 'mt-autocomplete-list';
    wrap.appendChild(list);

    input.addEventListener('input', () => {
      const val = input.value.toLowerCase();
      // For comma-separated inputs, only match the last segment
      const parts = val.split(',');
      const lastPart = parts[parts.length - 1].trim();
      if (!lastPart) { list.classList.remove('mt-show'); return; }

      const matches = items.filter(i => i.toLowerCase().includes(lastPart));
      if (matches.length === 0) { list.classList.remove('mt-show'); return; }

      list.innerHTML = matches.slice(0, 10).map(m =>
        `<div class="mt-autocomplete-item">${m}</div>`
      ).join('');
      list.classList.add('mt-show');
    });

    list.addEventListener('click', (e) => {
      const item = e.target.closest('.mt-autocomplete-item');
      if (!item) return;
      if (onSelect) {
        onSelect(item.textContent);
      } else {
        // For comma-separated, replace last part
        const parts = input.value.split(',').map(s => s.trim()).filter(Boolean);
        parts[parts.length - 1] = item.textContent;
        input.value = parts.join(', ') + ', ';
      }
      list.classList.remove('mt-show');
      input.focus();
    });

    document.addEventListener('click', (e) => {
      if (!wrap.contains(e.target)) list.classList.remove('mt-show');
    });
  }

  // --- Build UI ---

  // Floating button
  const btn = document.createElement('button');
  btn.id = 'mt-toolbar-btn';
  btn.innerHTML = ICON_SVG;
  btn.title = 'Marmite Toolbar';
  document.body.appendChild(btn);

  // Overlay
  const overlay = document.createElement('div');
  overlay.id = 'mt-panel-overlay';
  document.body.appendChild(overlay);

  // Panel
  const panel = document.createElement('div');
  panel.id = 'mt-panel';
  document.body.appendChild(panel);

  function saveState() {
    const activeTab = panel.querySelector('.mt-tab.mt-active');
    localStorage.setItem(STORAGE_KEY, JSON.stringify({
      open: panelOpen,
      tab: activeTab ? activeTab.dataset.tab : null,
    }));
  }

  function togglePanel() {
    panelOpen = !panelOpen;
    panel.classList.toggle('mt-open', panelOpen);
    overlay.classList.toggle('mt-open', panelOpen);
    saveState();
  }

  btn.addEventListener('click', togglePanel);
  overlay.addEventListener('click', togglePanel);

  // --- Tab rendering ---

  const CONTENT_TABS = ['Info', 'Edit', 'Actions', 'Site', 'Layout', 'Config'];
  const NON_CONTENT_TABS = ['Site', 'Layout', 'Config'];

  function renderPanel() {
    const tabs = isContentPage ? CONTENT_TABS : NON_CONTENT_TABS;
    const savedTab = stored.tab && tabs.includes(stored.tab) ? stored.tab : tabs[0];

    panel.innerHTML = `
      <div class="mt-panel-header">
        <h3>Marmite <span style="font-weight:400;font-size:11px;opacity:0.6">${siteData && siteData.marmite_version ? siteData.marmite_version : ''}</span></h3>
        <div style="display:flex;align-items:center;gap:8px">
          <a href="${API}/editor/${isContentPage ? currentSlug : ''}" class="mt-btn" style="font-size:11px;padding:2px 8px;text-decoration:none">Editor</a>
          <button class="mt-close-btn" id="mt-panel-close">&times;</button>
        </div>
      </div>
      <div class="mt-tabs">
        ${tabs.map(t => `<button class="mt-tab${t === savedTab ? ' mt-active' : ''}" data-tab="${t}">${t}</button>`).join('')}
      </div>
      ${tabs.map(t => `<div class="mt-tab-content${t === savedTab ? ' mt-active' : ''}" data-panel="${t}"></div>`).join('')}
    `;

    panel.querySelector('#mt-panel-close').addEventListener('click', togglePanel);

    // Tab switching
    panel.querySelectorAll('.mt-tab').forEach(tab => {
      tab.addEventListener('click', () => {
        panel.querySelectorAll('.mt-tab').forEach(t => t.classList.remove('mt-active'));
        panel.querySelectorAll('.mt-tab-content').forEach(t => t.classList.remove('mt-active'));
        tab.classList.add('mt-active');
        panel.querySelector(`[data-panel="${tab.dataset.tab}"]`).classList.add('mt-active');
        saveState();
      });
    });

    // Render each tab
    if (isContentPage) {
      renderInfoTab();
      renderEditTab();
      renderActionsTab();
    }
    renderSiteTab();
    renderLayoutTab();
    renderConfigTab();
  }

  function renderInfoTab() {
    const container = panel.querySelector('[data-panel="Info"]');
    if (!metadata) {
      container.innerHTML = '<p>Loading metadata...</p>';
      return;
    }
    const fm = metadata.frontmatter || {};
    const rows = [];
    if (fm.title) rows.push(['Title', fm.title]);
    if (fm.slug) rows.push(['Slug', fm.slug]);
    if (fm.date) rows.push(['Date', fm.date]);
    if (fm.description) rows.push(['Description', fm.description]);
    if (fm.tags && fm.tags.length) rows.push(['Tags', fm.tags.join(', ')]);
    if (fm.authors && fm.authors.length) rows.push(['Authors', fm.authors.join(', ')]);
    if (fm.stream) rows.push(['Stream', fm.stream]);
    if (fm.series) rows.push(['Series', fm.series]);
    if (fm.language) rows.push(['Language', fm.language]);
    if (fm.translates) rows.push(['Translates', fm.translates]);
    if (fm.pinned) rows.push(['Pinned', 'Yes']);
    if (metadata.source_path) rows.push(['Source', metadata.source_path]);
    if (metadata.last_updated) rows.push(['Modified', new Date(metadata.last_updated).toLocaleString()]);

    container.innerHTML = `
      <div class="mt-meta-grid">
        ${rows.map(([k, v]) => `<span class="mt-meta-key">${k}</span><span class="mt-meta-val">${v}</span>`).join('')}
      </div>
    `;
  }

  function renderEditTab() {
    const container = panel.querySelector('[data-panel="Edit"]');
    if (!metadata) {
      container.innerHTML = '<p>Loading...</p>';
      return;
    }
    const fm = metadata.frontmatter || {};
    const hasDate = !!fm.date;

    container.innerHTML = `
      <div class="mt-field">
        <label>Title
        <input type="text" id="mt-edit-title" value="${(fm.title || '').replace(/"/g, '&quot;')}">
        </label>
      </div>
      <div class="mt-field">
        <label>Slug
        <input type="text" id="mt-edit-slug" value="${(fm.slug || '').replace(/"/g, '&quot;')}">
        </label>
      </div>
      <div class="mt-field">
        <label>Description
        <textarea id="mt-edit-desc">${fm.description || ''}</textarea>
        </label>
      </div>
      <div class="mt-field">
        <label>Date
        <input type="datetime-local" id="mt-edit-date" value="${fm.date ? fm.date.replace(' ', 'T').slice(0, 19) : ''}"
          ${!hasDate ? 'placeholder="No date (page)"' : ''}>
        </label>
      </div>
      <div class="mt-field">
        <label>Tags (comma-separated)
        <input type="text" id="mt-edit-tags" value="${(fm.tags || []).join(', ')}">
        </label>
      </div>
      <div class="mt-field">
        <label>Stream
        <input type="text" id="mt-edit-stream" value="${fm.stream || ''}">
        </label>
      </div>
      <div class="mt-field">
        <label>Series
        <input type="text" id="mt-edit-series" value="${fm.series || ''}">
        </label>
      </div>
      <div class="mt-field">
        <label>Authors (comma-separated)
        <input type="text" id="mt-edit-authors" value="${(fm.authors || []).join(', ')}">
        </label>
      </div>
      <div class="mt-field">
        <label>Language
        <input type="text" id="mt-edit-lang" value="${fm.language || ''}">
        </label>
      </div>
      <div class="mt-field">
        <label>Translates (slug)
        <input type="text" id="mt-edit-translates" value="${fm.translates || ''}">
        </label>
      </div>
      <div class="mt-field">
        <label>Banner Image
        <input type="text" id="mt-edit-banner" value="${(fm.banner_image || '').replace(/"/g, '&quot;')}" placeholder="media/banner.jpg">
        </label>
      </div>
      <div class="mt-field">
        <label>Card Image
        <input type="text" id="mt-edit-card" value="${(fm.card_image || '').replace(/"/g, '&quot;')}" placeholder="media/card.jpg">
        </label>
      </div>
      <div class="mt-field">
        <label>Pinned
        <select id="mt-edit-pinned">
          <option value=""${!fm.pinned ? ' selected' : ''}>No</option>
          <option value="true"${fm.pinned ? ' selected' : ''}>Yes</option>
        </select>
        </label>
      </div>
      <div class="mt-field">
        <label>Comments
        <select id="mt-edit-comments">
          <option value=""${fm.comments === null || fm.comments === undefined ? ' selected' : ''}>Default</option>
          <option value="true"${fm.comments === true ? ' selected' : ''}>Enabled</option>
          <option value="false"${fm.comments === false ? ' selected' : ''}>Disabled</option>
        </select>
        </label>
      </div>
      <div class="mt-field">
        <label>Extra (JSON)
        <textarea id="mt-edit-extra">${fm.extra ? JSON.stringify(fm.extra, null, 2) : ''}</textarea>
        </label>
      </div>
      <button class="mt-btn mt-btn-primary mt-btn-block" id="mt-edit-save">Save Frontmatter</button>
    `;

    if (siteData) {
      if (siteData.tags) createAutocomplete(container.querySelector('#mt-edit-tags'), siteData.tags);
      if (siteData.streams) createAutocomplete(container.querySelector('#mt-edit-stream'), siteData.streams, (v) => { container.querySelector('#mt-edit-stream').value = v; });
      if (siteData.series) createAutocomplete(container.querySelector('#mt-edit-series'), siteData.series, (v) => { container.querySelector('#mt-edit-series').value = v; });
      if (siteData.authors) createAutocomplete(container.querySelector('#mt-edit-authors'), siteData.authors);
      const allLangs = siteData.iso_languages || siteData.languages || [];
      if (allLangs.length) createAutocomplete(container.querySelector('#mt-edit-lang'), allLangs, (v) => { container.querySelector('#mt-edit-lang').value = v; });
      if (siteData.slugs) createAutocomplete(container.querySelector('#mt-edit-translates'), siteData.slugs, (v) => { container.querySelector('#mt-edit-translates').value = v; });
      if (siteData.images) {
        createAutocomplete(container.querySelector('#mt-edit-banner'), siteData.images, (v) => { container.querySelector('#mt-edit-banner').value = v; });
        createAutocomplete(container.querySelector('#mt-edit-card'), siteData.images, (v) => { container.querySelector('#mt-edit-card').value = v; });
      }
    }

    container.querySelector('#mt-edit-save').addEventListener('click', async () => {
      const updates = {};
      const newTitle = container.querySelector('#mt-edit-title').value.trim();
      const newSlug = container.querySelector('#mt-edit-slug').value.trim();
      const newDesc = container.querySelector('#mt-edit-desc').value.trim();
      const rawDate = container.querySelector('#mt-edit-date').value.trim();
      const newDate = rawDate ? rawDate.replace('T', ' ') : '';
      const newTags = container.querySelector('#mt-edit-tags').value.split(',').map(s => s.trim()).filter(Boolean);
      const newStream = container.querySelector('#mt-edit-stream').value.trim();
      const newSeries = container.querySelector('#mt-edit-series').value.trim();
      const newAuthors = container.querySelector('#mt-edit-authors').value.split(',').map(s => s.trim()).filter(Boolean);
      const newLang = container.querySelector('#mt-edit-lang').value.trim();
      const newTranslates = container.querySelector('#mt-edit-translates').value.trim();
      const newBanner = container.querySelector('#mt-edit-banner').value.trim();
      const newCard = container.querySelector('#mt-edit-card').value.trim();
      const newPinned = container.querySelector('#mt-edit-pinned').value === 'true';
      const commentsVal = container.querySelector('#mt-edit-comments').value;
      const newComments = commentsVal === 'true' ? true : commentsVal === 'false' ? false : null;
      const extraText = container.querySelector('#mt-edit-extra').value.trim();

      if (newTitle !== (fm.title || '')) updates.title = newTitle;
      // Always write slug to frontmatter so the content has an explicit slug
      updates.slug = newSlug || fm.slug;
      if (newDesc !== (fm.description || '')) updates.description = newDesc || null;
      if (newDate !== (fm.date || '')) updates.date = newDate || null;
      if (JSON.stringify(newTags) !== JSON.stringify(fm.tags || [])) updates.tags = newTags.length ? newTags.join(', ') : null;
      if (newStream !== (fm.stream || '')) updates.stream = newStream || null;
      if (newSeries !== (fm.series || '')) updates.series = newSeries || null;
      if (JSON.stringify(newAuthors) !== JSON.stringify(fm.authors || [])) updates.authors = newAuthors.length ? newAuthors.join(', ') : null;
      if (newLang !== (fm.language || '')) updates.language = newLang || null;
      if (newTranslates !== (fm.translates || '')) updates.translates = newTranslates || null;
      if (newBanner !== (fm.banner_image || '')) updates.banner_image = newBanner || null;
      if (newCard !== (fm.card_image || '')) updates.card_image = newCard || null;
      if (newPinned !== (fm.pinned || false)) updates.pinned = newPinned || null;
      const origComments = fm.comments === undefined ? null : fm.comments;
      if (newComments !== origComments) updates.comments = newComments;
      if (extraText) {
        try {
          updates.extra = JSON.parse(extraText);
        } catch (e) {
          toast('Invalid JSON in Extra field', true);
          return;
        }
      } else if (fm.extra) {
        updates.extra = null;
      }

      // slug is always included; check if anything else changed
      const hasChanges = Object.keys(updates).some(k => k !== 'slug' || updates[k] !== fm.slug);
      if (!hasChanges) {
        toast('No changes to save');
        return;
      }

      try {
        holdReload();
        await api('PATCH', `/content/${currentSlug}`, updates);
        toast('Frontmatter saved');
        const targetSlug = newSlug || currentSlug;
        if (targetSlug !== currentSlug) {
          redirectAfterRebuild(targetSlug);
        } else {
          reloadAfterRebuild();
        }
      } catch (e) {
        toast('Save failed: ' + e.message, true);
      }
    });
  }

  function renderActionsTab() {
    const container = panel.querySelector('[data-panel="Actions"]');
    const fm = metadata ? metadata.frontmatter || {} : {};

    let translationsHtml = '';
    if (fm.translations && fm.translations.length) {
      translationsHtml = `<div style="margin-bottom:8px;font-size:12px;">Existing translations: ${fm.translations.map(t => t.lang || t.name).join(', ')}</div>`;
    }

    container.innerHTML = `
      <div class="mt-section-title">Translation</div>
      ${translationsHtml}
      <div class="mt-field">
        <label>Language code
        <input type="text" id="mt-action-trans-lang" placeholder="e.g. pt, es, fr">
        </label>
      </div>
      <div class="mt-field">
        <label>Title
        <input type="text" id="mt-action-trans-title" placeholder="Translated title">
        </label>
      </div>
      <button class="mt-btn mt-btn-primary mt-btn-block" id="mt-action-translate">Add Translation</button>

      <div class="mt-section-title">Clone / Copy</div>
      <div class="mt-field">
        <label>New title
        <input type="text" id="mt-action-clone-title" placeholder="Title for the copy">
        </label>
      </div>
      <button class="mt-btn mt-btn-block" id="mt-action-clone">Clone Content</button>

      <div class="mt-section-title">Move / Rename</div>
      <div class="mt-field">
        <label>New filename
        <input type="text" id="mt-action-move-name" value="${metadata && metadata.source_path ? metadata.source_path.split('/').pop() : ''}" placeholder="new-name.md">
        </label>
      </div>
      <button class="mt-btn mt-btn-block" id="mt-action-move">Move / Rename</button>

      <div class="mt-section-title">Danger Zone</div>
      <button class="mt-btn mt-btn-danger mt-btn-block" id="mt-action-delete">Delete Content</button>
    `;

    if (siteData) {
      const allLangs = siteData.iso_languages || siteData.languages || [];
      if (allLangs.length) createAutocomplete(container.querySelector('#mt-action-trans-lang'), allLangs, (v) => {
        container.querySelector('#mt-action-trans-lang').value = v;
      });
    }

    // Translation
    container.querySelector('#mt-action-translate').addEventListener('click', async () => {
      const lang = container.querySelector('#mt-action-trans-lang').value.trim();
      const title = container.querySelector('#mt-action-trans-title').value.trim();
      if (!lang) { toast('Language code is required', true); return; }
      if (!title) { toast('Title is required', true); return; }
      try {
        holdReload();
        const result = await api('POST', '/content', {
          title,
          lang,
          translates: currentSlug,
          tags: (fm.tags || []).join(', ') || undefined,
        });
        toast('Translation created');
        redirectAfterRebuild(result.slug);
      } catch (e) {
        toast('Failed: ' + e.message, true);
      }
    });

    // Clone - full filesystem copy, then patches title/slug
    container.querySelector('#mt-action-clone').addEventListener('click', async () => {
      const title = container.querySelector('#mt-action-clone-title').value.trim();
      if (!title) { toast('Title is required', true); return; }
      try {
        holdReload();
        const result = await api('POST', `/content/${currentSlug}/clone`, { title });
        toast('Content cloned');
        redirectAfterRebuild(result.slug);
      } catch (e) {
        toast('Failed: ' + e.message, true);
      }
    });

    // Move/Rename
    container.querySelector('#mt-action-move').addEventListener('click', async () => {
      const filename = container.querySelector('#mt-action-move-name').value.trim();
      if (!filename) { toast('Filename is required', true); return; }
      try {
        holdReload();
        const result = await api('POST', `/content/${currentSlug}/move`, { filename });
        toast('Content moved');
        redirectAfterRebuild(result.slug);
      } catch (e) {
        toast('Failed: ' + e.message, true);
      }
    });

    // Delete
    container.querySelector('#mt-action-delete').addEventListener('click', async () => {
      const ok = await confirm('Delete Content', `Are you sure you want to delete "${fm.title || currentSlug}"? This cannot be undone.`);
      if (!ok) return;
      try {
        holdReload();
        await api('DELETE', `/content/${currentSlug}`);
        toast('Content deleted');
        redirectAfterRebuild('index');
      } catch (e) {
        toast('Failed: ' + e.message, true);
      }
    });
  }

  function renderSiteTab() {
    const container = panel.querySelector('[data-panel="Site"]');
    const d = siteData || {};
    const postCount = d.post_count != null ? d.post_count : '-';
    const pageCount = d.page_count != null ? d.page_count : '-';
    const tagCount = (d.tags || []).length;
    const streamCount = (d.streams || []).length;
    const authorCount = (d.authors || []).length;
    const seriesCount = (d.series || []).length;
    const elapsed = d.elapsed_time ? (d.elapsed_time * 1000).toFixed(0) + 'ms' : '-';

    function statCard(num, label, href) {
      return `<a href="${href}" class="mt-stat-card" style="text-decoration:none;color:inherit">
        <div class="mt-stat-num">${num}</div>
        <div class="mt-stat-label">${label}</div>
      </a>`;
    }

    container.innerHTML = `
      <div class="mt-stats">
        ${statCard(postCount, 'Posts', '/streams.html')}
        ${statCard(pageCount, 'Pages', '/pages.html')}
        ${statCard(tagCount, 'Tags', '/tags.html')}
        ${statCard(streamCount, 'Streams', '/streams.html')}
        ${statCard(authorCount, 'Authors', '/authors.html')}
        ${statCard(seriesCount, 'Series', '/series.html')}
      </div>
      <div style="text-align:center;font-size:12px;margin-bottom:16px;opacity:0.7">Rendered in ${elapsed}</div>

      <div class="mt-section-title">Create</div>
      <div class="mt-field">
        <label>Title
        <input type="text" id="mt-site-new-title" placeholder="New content title">
        </label>
      </div>
      <div class="mt-field">
        <label>Tags (comma-separated)
        <input type="text" id="mt-site-new-tags" placeholder="tag1, tag2">
        </label>
      </div>
      <details class="mt-advanced">
        <summary style="cursor:pointer;font-size:12px;font-weight:600;margin-bottom:8px;user-select:none">+ Advanced</summary>
        <div class="mt-field">
          <label>Stream
          <input type="text" id="mt-site-new-stream" placeholder="e.g. tutorial, news">
          </label>
        </div>
        <div class="mt-field">
          <label>Series
          <input type="text" id="mt-site-new-series" placeholder="e.g. python-tutorial">
          </label>
        </div>
        <div class="mt-field">
          <label>Language
          <input type="text" id="mt-site-new-lang" placeholder="e.g. en, pt, es">
          </label>
        </div>
        <div class="mt-field">
          <label>Translates (slug of original post)
          <input type="text" id="mt-site-new-translates" placeholder="original-post-slug">
          </label>
        </div>
        <div class="mt-field">
          <label>Directory
          <input type="text" id="mt-site-new-directory" placeholder="e.g. tutorials/rust">
          </label>
        </div>
      </details>
      <button class="mt-btn mt-btn-primary mt-btn-block" id="mt-site-new-post">New Post</button>
      <button class="mt-btn mt-btn-block" id="mt-site-new-page">New Page</button>
    `;

    if (siteData) {
      if (siteData.tags) createAutocomplete(container.querySelector('#mt-site-new-tags'), siteData.tags);
      if (siteData.streams) createAutocomplete(container.querySelector('#mt-site-new-stream'), siteData.streams, (v) => { container.querySelector('#mt-site-new-stream').value = v; });
      if (siteData.series) createAutocomplete(container.querySelector('#mt-site-new-series'), siteData.series, (v) => { container.querySelector('#mt-site-new-series').value = v; });
      const allLangs = siteData.iso_languages || siteData.languages || [];
      if (allLangs.length) createAutocomplete(container.querySelector('#mt-site-new-lang'), allLangs, (v) => { container.querySelector('#mt-site-new-lang').value = v; });
      if (siteData.slugs) createAutocomplete(container.querySelector('#mt-site-new-translates'), siteData.slugs, (v) => { container.querySelector('#mt-site-new-translates').value = v; });
    }

    async function createContent(isPage) {
      const title = container.querySelector('#mt-site-new-title').value.trim();
      if (!title) { toast('Title is required', true); return; }
      const tags = container.querySelector('#mt-site-new-tags').value.trim();
      const stream = container.querySelector('#mt-site-new-stream').value.trim();
      const lang = container.querySelector('#mt-site-new-lang').value.trim();
      const translates = container.querySelector('#mt-site-new-translates').value.trim();
      const directory = container.querySelector('#mt-site-new-directory').value.trim();
      const body = { title };
      if (tags) body.tags = tags;
      if (isPage) body.page = true;
      if (lang) body.lang = lang;
      if (translates) body.translates = translates;
      if (directory) body.directory = directory;
      try {
        holdReload();
        const result = await api('POST', '/content', body);
        toast((isPage ? 'Page' : 'Post') + ' created');
        // If stream or series was set, patch frontmatter (create API doesn't support these)
        if (stream || container.querySelector('#mt-site-new-series').value.trim()) {
          const patch = {};
          if (stream) patch.stream = stream;
          const seriesVal = container.querySelector('#mt-site-new-series').value.trim();
          if (seriesVal) patch.series = seriesVal;
          await api('PATCH', `/content/${result.slug}`, patch);
        }
        redirectAfterRebuild(result.slug);
      } catch (e) {
        toast('Failed: ' + e.message, true);
      }
    }

    container.querySelector('#mt-site-new-post').addEventListener('click', () => createContent(false));
    container.querySelector('#mt-site-new-page').addEventListener('click', () => createContent(true));
  }

  function renderLayoutTab() {
    const container = panel.querySelector('[data-panel="Layout"]');
    const cfg = (siteData && siteData.config) ? siteData.config : {};
    const menu = cfg.menu || [];
    const esc = (v) => v == null ? '' : String(v).replace(/"/g, '&quot;');

    // Build menu editor rows
    const menuRows = menu.map((item, i) => `
      <div class="mt-menu-row" data-idx="${i}">
        <input type="text" class="mt-menu-name" name="mt-menu-${i}-name" value="${esc(item[0])}" placeholder="Label" style="flex:1">
        <input type="text" class="mt-menu-url" name="mt-menu-${i}-url" value="${esc(item[1])}" placeholder="URL" style="flex:1">
        <button class="mt-menu-up" title="Move up" style="padding:1px 5px;cursor:pointer;border:1px solid;border-radius:3px;background:none;font-size:11px">&uarr;</button>
        <button class="mt-menu-down" title="Move down" style="padding:1px 5px;cursor:pointer;border:1px solid;border-radius:3px;background:none;font-size:11px">&darr;</button>
        <button class="mt-menu-del" title="Remove" style="padding:1px 5px;cursor:pointer;border:1px solid;border-radius:3px;background:none;font-size:11px;color:#c0392b">&times;</button>
      </div>
    `).join('');

    function dnRow(title, key, val) {
      const dn = (val && typeof val === 'object') ? (val.display_name || '') : '';
      const desc = (val && typeof val === 'object') ? (val.description || '') : '';
      return `<div style="display:flex;gap:4px;margin-bottom:3px;align-items:center" class="mt-dn-row" data-section="${title}">
        <span style="font-weight:600;min-width:50px;font-size:11px">${key}</span>
        <input type="text" class="mt-dn-field" data-section="${title}" data-key="${key}" data-field="display_name" name="mt-dn-${title}-${key}-dn" value="${esc(dn)}" placeholder="Display name" style="flex:1">
        ${title === 'series' ? `<input type="text" class="mt-dn-field" data-section="${title}" data-key="${key}" data-field="description" name="mt-dn-${title}-${key}-desc" value="${esc(desc)}" placeholder="Description" style="flex:1">` : ''}
        <button class="mt-dn-del" title="Remove" style="padding:1px 5px;cursor:pointer;border:1px solid;border-radius:3px;background:none;font-size:11px;color:#c0392b">&times;</button>
      </div>`;
    }

    function displayNameSection(title, obj) {
      const entries = (obj && typeof obj === 'object') ? Object.entries(obj) : [];
      const rows = entries.map(([key, val]) => dnRow(title, key, val)).join('');
      return `<div class="mt-section-title">${title}</div>
        <div id="mt-dn-list-${title}">${rows}</div>
        <div style="display:flex;gap:4px;margin-top:4px">
          <input type="text" id="mt-dn-add-key-${title}" placeholder="Key" style="flex:1">
          <input type="text" id="mt-dn-add-dn-${title}" placeholder="Display name" style="flex:1">
          <button class="mt-btn mt-dn-add" data-section="${title}" style="white-space:nowrap">+</button>
        </div>`;
    }

    function authorRow(key, val) {
      const v = (val && typeof val === 'object') ? val : {};
      const linksStr = (v.links || []).map(l => l[0] + '=' + l[1]).join(', ');
      return `<details class="mt-author-entry" style="margin-bottom:4px;border:1px solid;border-radius:4px;padding:4px 6px">
        <summary style="cursor:pointer;font-weight:600;font-size:11px;display:flex;align-items:center;justify-content:space-between">${key}<button class="mt-author-del" title="Remove" style="padding:1px 5px;cursor:pointer;border:1px solid;border-radius:3px;background:none;font-size:11px;color:#c0392b">&times;</button></summary>
        <div style="margin-top:3px;display:flex;flex-direction:column;gap:2px">
          <input type="text" class="mt-author-field" data-author="${key}" data-field="name" name="mt-author-${key}-name" value="${esc(v.name)}" placeholder="Display name">
          <input type="text" class="mt-author-field" data-author="${key}" data-field="avatar" name="mt-author-${key}-avatar" value="${esc(v.avatar)}" placeholder="Avatar URL">
          <input type="text" class="mt-author-field" data-author="${key}" data-field="bio" name="mt-author-${key}-bio" value="${esc(v.bio)}" placeholder="Bio">
          <input type="text" class="mt-author-field" data-author="${key}" data-field="links" name="mt-author-${key}-links" value="${esc(linksStr)}" placeholder="Label=URL, Label=URL">
        </div>
      </details>`;
    }

    function authorsSection(authorsObj) {
      const entries = (authorsObj && typeof authorsObj === 'object' && !Array.isArray(authorsObj))
        ? Object.entries(authorsObj) : [];
      const rows = entries.map(([key, val]) => authorRow(key, val)).join('');
      return `<div class="mt-section-title">Authors</div>
        <div id="mt-authors-list">${rows}</div>
        <div style="display:flex;gap:4px;margin-top:4px">
          <input type="text" id="mt-author-add-key" placeholder="Author key" style="flex:1">
          <input type="text" id="mt-author-add-name" placeholder="Display name" style="flex:1">
          <button class="mt-btn" id="mt-author-add" style="white-space:nowrap">+</button>
        </div>`;
    }

    container.innerHTML = `
      <div class="mt-section-title">Menu</div>
      <div id="mt-menu-list" style="display:flex;flex-direction:column;gap:4px">
        ${menuRows}
      </div>
      <div style="display:flex;gap:4px;margin-top:8px">
        <input type="text" id="mt-menu-add-name" placeholder="Label" style="flex:1">
        <input type="text" id="mt-menu-add-url" placeholder="URL" style="flex:1">
        <button class="mt-btn" id="mt-menu-add" style="white-space:nowrap">+ Add</button>
      </div>

      <div class="mt-section-title">Titles</div>
      <div class="mt-field">
        <label>Pages Title
        <input type="text" class="mt-layout-title" name="mt-layout-title" data-key="pages_title" value="${esc(cfg.pages_title)}">
        </label>
      </div>
      <div class="mt-field">
        <label>Tags Title
        <input type="text" class="mt-layout-title" name="mt-layout-title" data-key="tags_title" value="${esc(cfg.tags_title)}">
        </label>
      </div>
      <div class="mt-field">
        <label>Archives Title
        <input type="text" class="mt-layout-title" name="mt-layout-title" data-key="archives_title" value="${esc(cfg.archives_title)}">
        </label>
      </div>
      <div class="mt-field">
        <label>Authors Title
        <input type="text" class="mt-layout-title" name="mt-layout-title" data-key="authors_title" value="${esc(cfg.authors_title)}">
        </label>
      </div>
      <div class="mt-field">
        <label>Streams Title
        <input type="text" class="mt-layout-title" name="mt-layout-title" data-key="streams_title" value="${esc(cfg.streams_title)}">
        </label>
      </div>
      <div class="mt-field">
        <label>Series Title
        <input type="text" class="mt-layout-title" name="mt-layout-title" data-key="series_title" value="${esc(cfg.series_title)}">
        </label>
      </div>
      <div class="mt-field">
        <label>Languages Title
        <input type="text" class="mt-layout-title" name="mt-layout-title" data-key="languages_title" value="${esc(cfg.languages_title)}">
        </label>
      </div>
      <div class="mt-field">
        <label>Search Title
        <input type="text" class="mt-layout-title" name="mt-layout-title" data-key="search_title" value="${esc(cfg.search_title)}">
        </label>
      </div>

      ${displayNameSection('streams', cfg.streams)}
      ${displayNameSection('series', cfg.series)}
      ${displayNameSection('languages', cfg.languages)}
      ${authorsSection(cfg.authors)}

      <button class="mt-btn mt-btn-primary mt-btn-block" id="mt-layout-save" style="margin-top:12px">Save Layout</button>
    `;

    // Menu row styles
    container.querySelectorAll('.mt-menu-row').forEach(row => {
      row.style.cssText = 'display:flex;gap:4px;align-items:center';
    });

    // Menu interactions
    const menuList = container.querySelector('#mt-menu-list');

    menuList.addEventListener('click', (e) => {
      const row = e.target.closest('.mt-menu-row');
      if (!row) return;
      const rows = [...menuList.querySelectorAll('.mt-menu-row')];
      const idx = rows.indexOf(row);
      if (e.target.classList.contains('mt-menu-del')) {
        row.remove();
      } else if (e.target.classList.contains('mt-menu-up') && idx > 0) {
        menuList.insertBefore(row, rows[idx - 1]);
      } else if (e.target.classList.contains('mt-menu-down') && idx < rows.length - 1) {
        rows[idx + 1].after(row);
      }
    });

    container.querySelector('#mt-menu-add').addEventListener('click', () => {
      const name = container.querySelector('#mt-menu-add-name').value.trim();
      const url = container.querySelector('#mt-menu-add-url').value.trim();
      if (!name || !url) { toast('Both label and URL are required', true); return; }
      const div = document.createElement('div');
      div.className = 'mt-menu-row';
      div.style.cssText = 'display:flex;gap:4px;align-items:center';
      div.innerHTML = `
        <input type="text" class="mt-menu-name" name="mt-menu-new-name" value="${esc(name)}" placeholder="Label" style="flex:1">
        <input type="text" class="mt-menu-url" name="mt-menu-new-url" value="${esc(url)}" placeholder="URL" style="flex:1">
        <button class="mt-menu-up" title="Move up" style="padding:1px 5px;cursor:pointer;border:1px solid;border-radius:3px;background:none;font-size:11px">&uarr;</button>
        <button class="mt-menu-down" title="Move down" style="padding:1px 5px;cursor:pointer;border:1px solid;border-radius:3px;background:none;font-size:11px">&darr;</button>
        <button class="mt-menu-del" title="Remove" style="padding:1px 5px;cursor:pointer;border:1px solid;border-radius:3px;background:none;font-size:11px;color:#c0392b">&times;</button>
      `;
      menuList.appendChild(div);
      container.querySelector('#mt-menu-add-name').value = '';
      container.querySelector('#mt-menu-add-url').value = '';
    });

    // Delete buttons for display name rows (delegated)
    ['streams', 'series', 'languages'].forEach(section => {
      const list = container.querySelector(`#mt-dn-list-${section}`);
      if (list) list.addEventListener('click', (e) => {
        if (e.target.classList.contains('mt-dn-del')) {
          e.target.closest('.mt-dn-row').remove();
        }
      });
    });

    // Delete buttons for author entries (delegated)
    const authorsList = container.querySelector('#mt-authors-list');
    if (authorsList) authorsList.addEventListener('click', (e) => {
      if (e.target.classList.contains('mt-author-del')) {
        e.preventDefault();
        e.target.closest('.mt-author-entry').remove();
      }
    });

    // Add buttons for display name sections (streams, series, languages)
    container.querySelectorAll('.mt-dn-add').forEach(btn => {
      btn.addEventListener('click', () => {
        const section = btn.dataset.section;
        const keyInput = container.querySelector(`#mt-dn-add-key-${section}`);
        const dnInput = container.querySelector(`#mt-dn-add-dn-${section}`);
        const key = keyInput.value.trim();
        const dn = dnInput.value.trim();
        if (!key) { toast('Key is required', true); return; }
        const list = container.querySelector(`#mt-dn-list-${section}`);
        list.insertAdjacentHTML('beforeend', dnRow(section, key, { display_name: dn }));
        keyInput.value = '';
        dnInput.value = '';
      });
    });

    // Autocomplete for add-key inputs using existing site data
    if (siteData) {
      const acMap = { streams: siteData.streams, series: siteData.series, languages: siteData.iso_languages || siteData.languages };
      for (const [section, items] of Object.entries(acMap)) {
        const input = container.querySelector(`#mt-dn-add-key-${section}`);
        if (input && items && items.length) createAutocomplete(input, items, (v) => { input.value = v; });
      }
      const authorKeyInput = container.querySelector('#mt-author-add-key');
      if (authorKeyInput && siteData.authors && siteData.authors.length) {
        createAutocomplete(authorKeyInput, siteData.authors, (v) => { authorKeyInput.value = v; });
      }
    }

    // Add button for authors
    const authorAddBtn = container.querySelector('#mt-author-add');
    if (authorAddBtn) {
      authorAddBtn.addEventListener('click', () => {
        const keyInput = container.querySelector('#mt-author-add-key');
        const nameInput = container.querySelector('#mt-author-add-name');
        const key = keyInput.value.trim();
        const name = nameInput.value.trim();
        if (!key) { toast('Author key is required', true); return; }
        const list = container.querySelector('#mt-authors-list');
        list.insertAdjacentHTML('beforeend', authorRow(key, { name }));
        keyInput.value = '';
        nameInput.value = '';
      });
    }

    // Save
    container.querySelector('#mt-layout-save').addEventListener('click', async () => {
      const updates = {};

      // Collect menu
      const menuItems = [];
      menuList.querySelectorAll('.mt-menu-row').forEach(row => {
        const name = row.querySelector('.mt-menu-name').value.trim();
        const url = row.querySelector('.mt-menu-url').value.trim();
        if (name && url) menuItems.push([name, url]);
      });
      if (JSON.stringify(menuItems) !== JSON.stringify(menu)) {
        updates.menu = menuItems;
      }

      // Collect titles
      container.querySelectorAll('.mt-layout-title').forEach(el => {
        const key = el.dataset.key;
        const val = el.value.trim();
        if (val !== (cfg[key] || '')) updates[key] = val || null;
      });

      // Collect display names for streams/series/languages (built from DOM only)
      ['streams', 'series', 'languages'].forEach(section => {
        const newData = {};
        container.querySelectorAll(`.mt-dn-field[data-section="${section}"]`).forEach(el => {
          const key = el.dataset.key;
          const field = el.dataset.field;
          const val = el.value.trim();
          if (!newData[key]) newData[key] = {};
          if (val) newData[key][field] = val;
        });
        if (JSON.stringify(newData) !== JSON.stringify(cfg[section] || {})) {
          updates[section] = newData;
        }
      });

      // Collect authors (built from DOM only)
      const newAuthors = {};
      container.querySelectorAll('.mt-author-field').forEach(el => {
        const author = el.dataset.author;
        const field = el.dataset.field;
        const val = el.value.trim();
        if (!newAuthors[author]) newAuthors[author] = {};
        if (field === 'links') {
          if (val) {
            newAuthors[author].links = val.split(',').map(pair => {
              const [label, ...urlParts] = pair.trim().split('=');
              return [label.trim(), urlParts.join('=').trim()];
            }).filter(([l, u]) => l && u);
          }
        } else if (val) {
          newAuthors[author][field] = val;
        }
      });
      const origAuthors = (cfg.authors && typeof cfg.authors === 'object' && !Array.isArray(cfg.authors)) ? cfg.authors : {};
      if (JSON.stringify(newAuthors) !== JSON.stringify(origAuthors)) {
        updates.authors = newAuthors;
      }

      if (Object.keys(updates).length === 0) {
        toast('No changes to save');
        return;
      }

      try {
        holdReload();
        await api('PATCH', '/config', updates);
        toast('Layout saved');
        reloadAfterRebuild();
      } catch (e) {
        toast('Save failed: ' + e.message, true);
      }
    });
  }

  function renderConfigTab() {
    const container = panel.querySelector('[data-panel="Config"]');
    const cfg = (siteData && siteData.config) ? siteData.config : {};

    const esc = (v) => v == null ? '' : String(v).replace(/"/g, '&quot;');

    container.innerHTML = `
      <div class="mt-section-title">General</div>
      <div class="mt-field">
        <label>Site Name
        <input type="text" class="mt-cfg-field" name="mt-cfg" data-key="name" value="${esc(cfg.name)}">
        </label>
      </div>
      <div class="mt-field">
        <label>Tagline
        <input type="text" class="mt-cfg-field" name="mt-cfg" data-key="tagline" value="${esc(cfg.tagline)}">
        </label>
      </div>
      <div class="mt-field">
        <label>URL
        <input type="text" class="mt-cfg-field" name="mt-cfg" data-key="url" value="${esc(cfg.url)}" placeholder="https://example.com">
        </label>
      </div>
      <div class="mt-field">
        <label>Language
        <input type="text" class="mt-cfg-field" name="mt-cfg" data-key="language" value="${esc(cfg.language)}">
        </label>
      </div>
      <div class="mt-field">
        <label>Footer
        <textarea class="mt-cfg-field" name="mt-cfg" data-key="footer">${cfg.footer || ''}</textarea>
        </label>
      </div>
      <div class="mt-field">
        <label>Default Author
        <input type="text" class="mt-cfg-field" name="mt-cfg" data-key="default_author" value="${esc(cfg.default_author)}">
        </label>
      </div>
      <div class="mt-field">
        <label>Default Date Format
        <input type="text" class="mt-cfg-field" name="mt-cfg" data-key="default_date_format" value="${esc(cfg.default_date_format)}" placeholder="%B %d, %Y">
        </label>
      </div>

      <div class="mt-section-title">Display</div>
      <div class="mt-field">
        <label>Pagination
        <input type="number" class="mt-cfg-field" name="mt-cfg" data-key="pagination" data-type="number" value="${cfg.pagination || 10}" min="1">
        </label>
      </div>
      <div class="mt-field">
        <label>Enable Search
        <select class="mt-cfg-field" name="mt-cfg" data-key="enable_search">
          <option value="true"${cfg.enable_search ? ' selected' : ''}>Yes</option>
          <option value="false"${!cfg.enable_search ? ' selected' : ''}>No</option>
        </select>
        </label>
      </div>
      <div class="mt-field">
        <label>Show Related Content
        <select class="mt-cfg-field" name="mt-cfg" data-key="enable_related_content">
          <option value="true"${cfg.enable_related_content ? ' selected' : ''}>Yes</option>
          <option value="false"${!cfg.enable_related_content ? ' selected' : ''}>No</option>
        </select>
        </label>
      </div>
      <div class="mt-field">
        <label>Show Next/Prev Links
        <select class="mt-cfg-field" name="mt-cfg" data-key="show_next_prev_links">
          <option value="true"${cfg.show_next_prev_links ? ' selected' : ''}>Yes</option>
          <option value="false"${!cfg.show_next_prev_links ? ' selected' : ''}>No</option>
        </select>
        </label>
      </div>

      <div class="mt-section-title">Images</div>
      <div class="mt-field">
        <label>Card Image
        <input type="text" class="mt-cfg-field" name="mt-cfg" data-key="card_image" value="${esc(cfg.card_image)}">
        </label>
      </div>
      <div class="mt-field">
        <label>Banner Image
        <input type="text" class="mt-cfg-field" name="mt-cfg" data-key="banner_image" value="${esc(cfg.banner_image)}">
        </label>
      </div>
      <div class="mt-field">
        <label>Logo Image
        <input type="text" class="mt-cfg-field" name="mt-cfg" data-key="logo_image" value="${esc(cfg.logo_image)}">
        </label>
      </div>

      <div class="mt-section-title">Paths</div>
      <div class="mt-field">
        <label>Content Path
        <input type="text" class="mt-cfg-field" name="mt-cfg" data-key="content_path" value="${esc(cfg.content_path)}" placeholder="content">
        </label>
      </div>
      <div class="mt-field">
        <label>Site Path
        <input type="text" class="mt-cfg-field" name="mt-cfg" data-key="site_path" value="${esc(cfg.site_path)}" placeholder="site">
        </label>
      </div>
      <div class="mt-field">
        <label>Media Path
        <input type="text" class="mt-cfg-field" name="mt-cfg" data-key="media_path" value="${esc(cfg.media_path)}" placeholder="media">
        </label>
      </div>

      <div class="mt-section-title">Extra (JSON)</div>
      <div class="mt-field">
        <textarea class="mt-cfg-field" name="mt-cfg" data-key="extra" id="mt-cfg-extra" style="min-height:100px">${cfg.extra ? JSON.stringify(cfg.extra, null, 2) : '{}'}</textarea>
      </div>

      <button class="mt-btn mt-btn-primary mt-btn-block" id="mt-cfg-save">Save Config</button>
    `;

    if (siteData && siteData.images) {
      ['card_image', 'banner_image', 'logo_image'].forEach(key => {
        const input = container.querySelector(`.mt-cfg-field[data-key="${key}"]`);
        if (input) createAutocomplete(input, siteData.images, (v) => { input.value = v; });
      });
    }

    container.querySelector('#mt-cfg-save').addEventListener('click', async () => {
      const updates = {};
      container.querySelectorAll('.mt-cfg-field[data-key]').forEach(el => {
        const key = el.dataset.key;
        let val;
        if (el.tagName === 'SELECT') {
          val = el.value === 'true' ? true : el.value === 'false' ? false : el.value;
        } else if (el.dataset.type === 'number') {
          val = parseInt(el.value, 10);
          if (isNaN(val)) return;
        } else {
          val = el.value;
        }

        if (key === 'extra') {
          try {
            val = JSON.parse(el.value);
          } catch (e) {
            toast('Invalid JSON in Extra field', true);
            return;
          }
        }

        const orig = cfg[key];
        if (key === 'extra') {
          if (JSON.stringify(val) !== JSON.stringify(orig || {})) updates[key] = val;
        } else if (typeof val === 'boolean') {
          if (val !== (orig || false)) updates[key] = val;
        } else if (typeof val === 'number') {
          if (val !== (orig || 0)) updates[key] = val;
        } else {
          if ((val || '') !== (orig || '')) updates[key] = val || null;
        }
      });

      if (Object.keys(updates).length === 0) {
        toast('No changes to save');
        return;
      }

      try {
        holdReload();
        await api('PATCH', '/config', updates);
        toast('Config saved');
        reloadAfterRebuild();
      } catch (e) {
        toast('Save failed: ' + e.message, true);
      }
    });
  }

  // --- 404 Create Page button ---

  function render404CreateButton() {
    if (!is404) return;
    const slug = window.__marmite_404_slug__;
    if (slug.includes('.')) return;
    const title = slug.replace(/-/g, ' ').replace(/\b\w/g, c => c.toUpperCase());
    const contentHtml = document.querySelector('.content-html');
    const target = contentHtml || document.querySelector('.main-content') || document.querySelector('main');
    if (!target) return;

    const div = document.createElement('div');
    div.className = 'mt-404-create';
    div.innerHTML = `
      <p style="margin-bottom:8px">Page not found. Create it as:</p>
      <button class="mt-404-create-btn" id="mt-404-create-page" style="margin-right:8px">Create Page</button>
      <button class="mt-404-create-btn" id="mt-404-create-post">Create Post</button>`;
    target.appendChild(div);

    async function create404(isPage) {
      try {
        holdReload();
        const body = { title };
        if (isPage) body.page = true;
        const result = await api('POST', '/content', body);
        toast((isPage ? 'Page' : 'Post') + ' created');
        redirectAfterRebuild(result.slug);
      } catch (e) {
        toast('Failed: ' + e.message, true);
      }
    }
    document.getElementById('mt-404-create-page').addEventListener('click', () => create404(true));
    document.getElementById('mt-404-create-post').addEventListener('click', () => create404(false));
  }

  // --- Init ---

  async function init() {
    // Fetch site data for autocomplete
    try {
      siteData = await (await fetch(`${API}/data`)).json();
      if (siteData && !siteData.watch_enabled) {
        toast('File watcher is not active. Run with --watch (-w) to enable auto-rebuild after edits.', true);
      }
    } catch (e) { /* ok */ }

    // Try to load metadata for current page
    if (currentSlug && currentSlug !== 'index' && currentSlug !== '404' && !is404) {
      try {
        const resp = await fetch(`/${currentSlug}.metadata.json`);
        if (resp.ok) {
          metadata = await resp.json();
          isContentPage = true;
        }
      } catch (e) { /* not a content page */ }
    }

    // Also try for index page (it might be a content page in some configs)
    if (currentSlug === 'index' && !is404) {
      try {
        const resp = await fetch('/index.metadata.json');
        if (resp.ok) {
          metadata = await resp.json();
          isContentPage = true;
        }
      } catch (e) { /* ok */ }
    }

    renderPanel();
    render404CreateButton();
    if (panelOpen) {
      panel.classList.add('mt-open');
      overlay.classList.add('mt-open');
    }
  }

  // Keyboard shortcut: Escape to close panel
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && panelOpen) {
      togglePanel();
    }
  });

  init();
})();
