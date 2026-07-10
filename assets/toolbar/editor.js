import { EditorView, basicSetup } from "codemirror";
import { EditorState, Compartment } from "@codemirror/state";
import { keymap } from "@codemirror/view";
import { markdown } from "@codemirror/lang-markdown";
import { oneDark } from "@codemirror/theme-one-dark";
import {
  solarizedLight, dracula, cobalt, coolGlow,
  amy, espresso, clouds, tomorrow,
  noctisLilac, rosePineDawn, ayuLight,
} from "thememirror";
import { autocompletion, startCompletion } from "@codemirror/autocomplete";

const editorThemes = {
  solarizedLight: { ext: solarizedLight, label: "Solarized Light" },
  ayuLight: { ext: ayuLight, label: "Ayu Light" },
  clouds: { ext: clouds, label: "Clouds" },
  rosePineDawn: { ext: rosePineDawn, label: "Rose Pine Dawn" },
  noctisLilac: { ext: noctisLilac, label: "Noctis Lilac" },
  tomorrow: { ext: tomorrow, label: "Tomorrow" },
  dracula: { ext: dracula, label: "Dracula" },
  oneDark: { ext: oneDark, label: "One Dark" },
  cobalt: { ext: cobalt, label: "Cobalt" },
  coolGlow: { ext: coolGlow, label: "Cool Glow" },
  amy: { ext: amy, label: "Amy" },
  espresso: { ext: espresso, label: "Espresso" },
};

// --- State ---
const slug = window.__marmite_editor_slug__;
const API = '/__marmite__';
const AUTOSAVE_KEY = `marmiteEditor_${slug}`;
const PREFS_KEY = 'marmiteEditorPrefs';

let editorView = null;
let frontmatter = {};
let sourcePath = '';
let originalBody = '';
let siteData = null;
let fileTree = null;
const emptyMode = !slug;
const rawMode = !emptyMode && (slug.includes('/') || slug.includes('.') || slug.startsWith('_'));
let isDirty = false;
let autoSaveTimer = null;

const themeCompartment = new Compartment();
const fontSizeCompartment = new Compartment();

// --- Utilities ---
const $ = (sel) => document.querySelector(sel);
const $$ = (sel) => document.querySelectorAll(sel);

function getPrefs() {
  try {
    return JSON.parse(localStorage.getItem(PREFS_KEY) || '{}');
  } catch { return {}; }
}

function savePrefs(updates) {
  const prefs = { ...getPrefs(), ...updates };
  localStorage.setItem(PREFS_KEY, JSON.stringify(prefs));
  return prefs;
}

function toast(msg, isError) {
  let el = document.querySelector('.me-toast');
  if (!el) {
    el = document.createElement('div');
    el.className = 'me-toast';
    document.body.appendChild(el);
  }
  el.textContent = msg;
  el.classList.toggle('me-error', !!isError);
  el.classList.add('me-show');
  setTimeout(() => el.classList.remove('me-show'), 3000);
}

function confirmDialog(title, message) {
  return new Promise((resolve) => {
    const overlay = document.createElement('div');
    overlay.className = 'me-confirm-overlay';
    overlay.innerHTML = `
      <div class="me-confirm-box">
        <h4>${title}</h4>
        <p>${message}</p>
        <div class="me-confirm-actions">
          <button class="me-btn" id="me-confirm-no">Cancel</button>
          <button class="me-btn me-btn-primary" id="me-confirm-yes">OK</button>
        </div>
      </div>`;
    document.body.appendChild(overlay);
    overlay.querySelector('#me-confirm-yes').onclick = () => { overlay.remove(); resolve(true); };
    overlay.querySelector('#me-confirm-no').onclick = () => { overlay.remove(); resolve(false); };
  });
}

function promptDialog(title, label, defaultValue) {
  return new Promise((resolve) => {
    const overlay = document.createElement('div');
    overlay.className = 'me-confirm-overlay';
    overlay.innerHTML = `
      <div class="me-confirm-box">
        <h4>${title}</h4>
        <div class="me-field" style="margin-bottom:16px">
          <label>${label}
          <input type="text" id="me-prompt-input" value="${(defaultValue || '').replace(/"/g, '&quot;')}">
          </label>
        </div>
        <div class="me-confirm-actions">
          <button class="me-btn" id="me-prompt-cancel">Cancel</button>
          <button class="me-btn me-btn-primary" id="me-prompt-ok">OK</button>
        </div>
      </div>`;
    document.body.appendChild(overlay);
    const input = overlay.querySelector('#me-prompt-input');
    input.focus();
    input.select();
    overlay.querySelector('#me-prompt-ok').onclick = () => { overlay.remove(); resolve(input.value.trim()); };
    overlay.querySelector('#me-prompt-cancel').onclick = () => { overlay.remove(); resolve(null); };
    input.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') { overlay.remove(); resolve(input.value.trim()); }
      if (e.key === 'Escape') { overlay.remove(); resolve(null); }
    });
  });
}

async function api(method, path, body) {
  const opts = { method, headers: {} };
  if (body !== undefined) {
    opts.headers['Content-Type'] = 'application/json';
    opts.body = JSON.stringify(body);
  }
  const resp = await fetch(`${API}${path}`, opts);
  const data = await resp.json();
  if (!resp.ok) throw new Error(data.error || `HTTP ${resp.status}`);
  return data;
}

// --- Autocomplete for sidebar ---
function createAutocomplete(input, items, onSelect) {
  const wrap = document.createElement('div');
  wrap.className = 'me-autocomplete-wrap';
  input.parentNode.insertBefore(wrap, input);
  wrap.appendChild(input);

  const list = document.createElement('div');
  list.className = 'me-autocomplete-list';
  wrap.appendChild(list);

  input.addEventListener('input', () => {
    const val = input.value.toLowerCase();
    const parts = val.split(',');
    const lastPart = parts[parts.length - 1].trim();
    if (!lastPart) { list.classList.remove('me-show'); return; }
    const matches = items.filter(i => i.toLowerCase().includes(lastPart));
    if (matches.length === 0) { list.classList.remove('me-show'); return; }
    list.innerHTML = matches.slice(0, 10).map(m =>
      `<div class="me-autocomplete-item">${m}</div>`
    ).join('');
    list.classList.add('me-show');
  });

  list.addEventListener('click', (e) => {
    const item = e.target.closest('.me-autocomplete-item');
    if (!item) return;
    if (onSelect) {
      onSelect(item.textContent);
    } else {
      const parts = input.value.split(',').map(s => s.trim()).filter(Boolean);
      parts[parts.length - 1] = item.textContent;
      input.value = parts.join(', ') + ', ';
    }
    list.classList.remove('me-show');
    input.focus();
  });

  document.addEventListener('click', (e) => {
    if (!wrap.contains(e.target)) list.classList.remove('me-show');
  });
}

// --- Live-reload ---
// Instead of including livereload.js (which would reload the whole page),
// we open our own WebSocket and only refresh the preview iframe on rebuild.
function connectLiveReload() {
  const isHttps = window.location.protocol === 'https:';
  const host = window.location.hostname.includes(':')
    ? `[${window.location.hostname}]` : window.location.hostname;
  const port = window.location.port ? `:${window.location.port}` : '';
  const url = `${isHttps ? 'wss' : 'ws'}://${host}${port}/__marmite__/livereload`;

  const ws = new WebSocket(url);
  ws.addEventListener('message', (event) => {
    try {
      const payload = JSON.parse(event.data);
      if (payload.event === 'reload') {
        const iframe = $('#me-preview-frame');
        if (iframe && iframe.src) {
          iframe.src = iframe.src.split('?')[0] + '?t=' + Date.now();
        }
      }
    } catch { /* ignore */ }
  });
  ws.addEventListener('close', () => setTimeout(connectLiveReload, 2000));
  ws.addEventListener('error', () => ws.close());
}

// --- Preview sync ---
// When the editor regains focus, snap the preview back to the page being edited
// (the user may have browsed to other pages in the preview iframe).
function syncPreviewToSlug() {
  if (rawMode) return;
  const iframe = $('#me-preview-frame');
  if (!iframe || !iframe.src) return;
  const expectedPath = `/${slug}.html`;
  try {
    const current = new URL(iframe.src);
    if (current.pathname !== expectedPath) {
      iframe.src = expectedPath;
    }
  } catch {
    iframe.src = expectedPath;
  }
}

// --- CodeMirror Editor ---
function getEditorThemeExt(name) {
  const entry = editorThemes[name];
  return entry ? entry.ext : dracula;
}

function makeFontSizeTheme(size) {
  return EditorView.theme({
    ".cm-content": { fontSize: size + "px" },
    ".cm-gutters": { fontSize: size + "px" },
  });
}

const FRONTMATTER_KEYS = [
  "title", "slug", "date", "tags", "authors", "description",
  "stream", "series", "pinned", "toc", "comments",
  "card_image", "banner_image", "language", "translates", "extra",
];

function editorCompletions(context) {
  const pos = context.pos;
  const line = context.state.doc.lineAt(pos);
  const textBefore = line.text.slice(0, pos - line.from);
  const doc = context.state.doc.toString();

  // Wikilinks: [[partial
  const wikiMatch = textBefore.match(/\[\[([^\]|]*)$/);
  if (wikiMatch && siteData && siteData.content_items) {
    const partial = wikiMatch[1].toLowerCase();
    const from = pos - wikiMatch[1].length;
    const textAfter = line.text.slice(pos - line.from);
    const closingBrackets = textAfter.match(/^\]{0,2}/)[0].length;
    const to = pos + closingBrackets;
    const filtered = partial
      ? siteData.content_items.filter(c => c.title.toLowerCase().includes(partial) || c.slug.includes(partial))
      : siteData.content_items;
    const options = filtered.map(c => ({
      label: c.title,
      detail: c.slug,
      apply: c.title === c.slug ? `${c.slug}]]` : `${c.title}|${c.slug}]]`,
    }));
    if (options.length) return { from, to, options, filter: false };
  }

  // Shortcodes: <!-- .partial
  const scMatch = textBefore.match(/<!--\s*\.(\w*)$/);
  if (scMatch && siteData && siteData.shortcodes) {
    const partial = scMatch[1].toLowerCase();
    const from = pos - scMatch[1].length;
    const options = siteData.shortcodes
      .filter(name => name.toLowerCase().startsWith(partial))
      .map(name => ({ label: name, apply: `${name} -->` }));
    if (options.length) return { from, options };
  }

  // Image paths: ![...]( or src="
  const imgMatch = textBefore.match(/(?:\!\[[^\]]*\]\(|src=["'])([^)"']*)$/);
  if (imgMatch && siteData && siteData.images) {
    const partial = imgMatch[1].toLowerCase();
    const from = pos - imgMatch[1].length;
    const options = siteData.images
      .filter(img => img.toLowerCase().includes(partial))
      .slice(0, 15)
      .map(img => ({ label: img }));
    if (options.length) return { from, options };
  }

  // Frontmatter context
  const beforeCursor = doc.slice(0, pos);
  const fmStart = beforeCursor.indexOf("---\n");
  if (fmStart === -1 || fmStart > 5) return null;
  const fmEnd = beforeCursor.indexOf("\n---", fmStart + 4);
  if (fmEnd !== -1 && pos > fmEnd + 4) return null;

  // Frontmatter key
  const keyMatch = textBefore.match(/^(\w*)$/);
  if (keyMatch && keyMatch[1].length > 0) {
    const partial = keyMatch[1].toLowerCase();
    const from = pos - keyMatch[1].length;
    const options = FRONTMATTER_KEYS
      .filter(k => k.startsWith(partial))
      .map(k => ({ label: k, apply: `${k}: ` }));
    if (options.length) return { from, options };
  }

  // Frontmatter values: tags, authors, stream, series
  const tagsMatch = textBefore.match(/^tags:\s*(?:.*,\s*)?(\w*)$/);
  if (tagsMatch && siteData && siteData.tags) {
    const partial = tagsMatch[1].toLowerCase();
    const from = pos - tagsMatch[1].length;
    const options = siteData.tags
      .filter(t => t.toLowerCase().startsWith(partial))
      .map(t => ({ label: t }));
    if (options.length) return { from, options };
  }

  const authorsMatch = textBefore.match(/^authors?:\s*(?:.*,\s*)?(\w*)$/);
  if (authorsMatch && siteData && siteData.authors) {
    const partial = authorsMatch[1].toLowerCase();
    const from = pos - authorsMatch[1].length;
    const options = siteData.authors
      .filter(a => a.toLowerCase().startsWith(partial))
      .map(a => ({ label: a }));
    if (options.length) return { from, options };
  }

  const streamMatch = textBefore.match(/^stream:\s*(\w*)$/);
  if (streamMatch && siteData && siteData.streams) {
    const partial = streamMatch[1].toLowerCase();
    const from = pos - streamMatch[1].length;
    const options = siteData.streams
      .filter(s => s.toLowerCase().startsWith(partial))
      .map(s => ({ label: s }));
    if (options.length) return { from, options };
  }

  const seriesMatch = textBefore.match(/^series:\s*(.*)$/);
  if (seriesMatch && siteData && siteData.series) {
    const partial = seriesMatch[1].toLowerCase();
    const from = pos - seriesMatch[1].length;
    const options = siteData.series
      .filter(s => s.toLowerCase().startsWith(partial))
      .map(s => ({ label: s }));
    if (options.length) return { from, options };
  }

  return null;
}

function createEditor(content) {
  if (editorView) editorView.destroy();

  const prefs = getPrefs();
  const fontSize = prefs.fontSize || 14;
  const themeName = prefs.editorTheme || 'dracula';

  const state = EditorState.create({
    doc: content,
    extensions: [
      basicSetup,
      markdown(),
      themeCompartment.of(getEditorThemeExt(themeName)),
      fontSizeCompartment.of(makeFontSizeTheme(fontSize)),
      autocompletion({ override: [editorCompletions] }),
      EditorView.inputHandler.of((view, from, to, text) => {
        if (text === '[' || text === '.') {
          setTimeout(() => startCompletion(view), 0);
        }
        return false;
      }),
      keymap.of([{
        key: "Mod-s",
        run: () => { saveContent(); return true; },
      }]),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          isDirty = true;
          updateDirtyIndicator();
          scheduleAutoSave();
        }
        // Update cursor position
        const cursor = update.state.selection.main.head;
        const line = update.state.doc.lineAt(cursor);
        const col = cursor - line.from + 1;
        $('#me-cursor-pos').textContent = `Ln ${line.number}, Col ${col}`;
      }),
      EditorView.domEventHandlers({
        focus: () => { syncPreviewToSlug(); },
      }),
    ],
  });

  editorView = new EditorView({
    state,
    parent: $('#me-editor-container'),
  });

  // Set up theme/font controls
  const themeSelect = $('#me-editor-theme');
  themeSelect.value = themeName;
  const fontRange = $('#me-font-size');
  fontRange.value = fontSize;
  $('#me-font-size-label').textContent = fontSize + 'px';
}

function updateDirtyIndicator() {
  const el = $('#me-dirty-indicator');
  if (el) el.classList.toggle('me-visible', isDirty);
}

// --- Auto-save ---
// Debounced: after user stops typing for 1.5s, save to server (triggers rebuild + preview refresh).
// Also saves a localStorage backup immediately for crash recovery.
let autoSaveLocalTimer = null;

function scheduleAutoSave() {
  // Immediate localStorage backup (debounced 500ms)
  if (autoSaveLocalTimer) clearTimeout(autoSaveLocalTimer);
  autoSaveLocalTimer = setTimeout(() => {
    if (!editorView) return;
    localStorage.setItem(AUTOSAVE_KEY, JSON.stringify({
      body: editorView.state.doc.toString(),
      timestamp: Date.now(),
      isDraft: true,
    }));
  }, 500);

  // Debounced server save (1.5s after last keystroke)
  if (autoSaveTimer) clearTimeout(autoSaveTimer);
  autoSaveTimer = setTimeout(() => {
    if (!editorView) return;
    saveContent();
  }, 1500);
}

// --- Save ---
async function saveContent() {
  if (!editorView) return;
  const body = editorView.state.doc.toString();

  try {
    if (rawMode) {
      await fetch(`${API}/file/${slug}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content: body }),
      }).then(async r => {
        if (!r.ok) { const d = await r.json(); throw new Error(d.error || r.statusText); }
      });
    } else {
      const fmUpdates = collectFrontmatterUpdates();
      const result = await api('PUT', `/content/${slug}/body`, {
        body,
        frontmatter: Object.keys(fmUpdates).length > 0 ? fmUpdates : undefined,
      });
      frontmatter = result.frontmatter || frontmatter;
      renderInfoPanel();
    }

    originalBody = body;
    isDirty = false;
    updateDirtyIndicator();

    localStorage.setItem(AUTOSAVE_KEY, JSON.stringify({
      body,
      timestamp: Date.now(),
      isDraft: false,
    }));

    toast('Saved');
  } catch (e) {
    toast('Save failed: ' + e.message, true);
  }
}

async function saveAs() {
  const title = await promptDialog('Save As', 'New title:', frontmatter.title || '');
  if (!title) return;

  try {
    const result = await api('POST', '/content', { title });
    const newSlug = result.slug;
    const body = editorView.state.doc.toString();

    await api('PUT', `/content/${newSlug}/body`, { body });
    toast('Saved as new content');
    window.location.href = `${API}/editor/${newSlug}`;
  } catch (e) {
    toast('Save As failed: ' + e.message, true);
  }
}

function collectFrontmatterUpdates() {
  const updates = {};
  const panel = $('#me-panel-edit');
  if (!panel) return updates;

  const fields = {
    'me-edit-title': { key: 'title', orig: frontmatter.title },
    'me-edit-slug': { key: 'slug', orig: frontmatter.slug },
    'me-edit-desc': { key: 'description', orig: frontmatter.description },
    'me-edit-stream': { key: 'stream', orig: frontmatter.stream },
    'me-edit-series': { key: 'series', orig: frontmatter.series },
    'me-edit-lang': { key: 'language', orig: frontmatter.language },
    'me-edit-translates': { key: 'translates', orig: frontmatter.translates },
    'me-edit-banner': { key: 'banner_image', orig: frontmatter.banner_image },
    'me-edit-card': { key: 'card_image', orig: frontmatter.card_image },
  };

  for (const [id, { key, orig }] of Object.entries(fields)) {
    const el = panel.querySelector(`#${id}`);
    if (!el) continue;
    const val = el.value.trim();
    if (val !== (orig || '')) {
      updates[key] = val || null;
    }
  }

  const dateEl = panel.querySelector('#me-edit-date');
  if (dateEl) {
    const rawDate = dateEl.value.trim();
    const newDate = rawDate ? rawDate.replace('T', ' ') : '';
    if (newDate !== (frontmatter.date || '')) {
      updates.date = newDate || null;
    }
  }

  const tagsEl = panel.querySelector('#me-edit-tags');
  if (tagsEl) {
    const newTags = tagsEl.value.split(',').map(s => s.trim()).filter(Boolean);
    const origTags = Array.isArray(frontmatter.tags) ? frontmatter.tags : (frontmatter.tags ? String(frontmatter.tags).split(',').map(s => s.trim()).filter(Boolean) : []);
    if (JSON.stringify(newTags) !== JSON.stringify(origTags)) {
      updates.tags = newTags.length ? newTags.join(', ') : null;
    }
  }

  const authorsEl = panel.querySelector('#me-edit-authors');
  if (authorsEl) {
    const newAuthors = authorsEl.value.split(',').map(s => s.trim()).filter(Boolean);
    const origAuthors = Array.isArray(frontmatter.authors) ? frontmatter.authors : (frontmatter.authors ? String(frontmatter.authors).split(',').map(s => s.trim()).filter(Boolean) : []);
    if (JSON.stringify(newAuthors) !== JSON.stringify(origAuthors)) {
      updates.authors = newAuthors.length ? newAuthors.join(', ') : null;
    }
  }

  const pinnedEl = panel.querySelector('#me-edit-pinned');
  if (pinnedEl) {
    const newPinned = pinnedEl.value === 'true';
    if (newPinned !== (frontmatter.pinned || false)) {
      updates.pinned = newPinned || null;
    }
  }

  const commentsEl = panel.querySelector('#me-edit-comments');
  if (commentsEl) {
    const v = commentsEl.value;
    const newComments = v === 'true' ? true : v === 'false' ? false : null;
    const origComments = frontmatter.comments === undefined ? null : frontmatter.comments;
    if (newComments !== origComments) updates.comments = newComments;
  }

  const extraEl = panel.querySelector('#me-edit-extra');
  if (extraEl) {
    const extraText = extraEl.value.trim();
    if (extraText) {
      try {
        updates.extra = JSON.parse(extraText);
      } catch { /* ignore invalid JSON */ }
    } else if (frontmatter.extra) {
      updates.extra = null;
    }
  }

  return updates;
}

// --- Sidebar rendering ---
function renderInfoPanel() {
  const container = $('#me-panel-info');
  if (!container) return;

  const rows = [];
  if (rawMode) {
    rows.push(['File', sourcePath || slug]);
  } else {
    const fm = frontmatter || {};
    if (fm.title) rows.push(['Title', fm.title]);
    if (fm.slug) rows.push(['Slug', fm.slug]);
    if (fm.date) rows.push(['Date', fm.date]);
    if (fm.description) rows.push(['Description', fm.description]);
    if (fm.tags) rows.push(['Tags', Array.isArray(fm.tags) ? fm.tags.join(', ') : String(fm.tags)]);
    if (fm.authors) rows.push(['Authors', Array.isArray(fm.authors) ? fm.authors.join(', ') : String(fm.authors)]);
    if (fm.stream) rows.push(['Stream', fm.stream]);
    if (fm.series) rows.push(['Series', fm.series]);
    if (fm.language) rows.push(['Language', fm.language]);
    if (fm.translates) rows.push(['Translates', fm.translates]);
    if (fm.pinned) rows.push(['Pinned', 'Yes']);
    if (sourcePath) rows.push(['Source', sourcePath]);
  }

  let treeHtml = '';
  const projectEmpty = !fileTree || fileTree.length === 0;
  if (!projectEmpty) {
    treeHtml = renderFileTree(fileTree);
  }

  container.innerHTML = `
    <div class="me-meta-grid">
      ${rows.map(([k, v]) => `<span class="me-meta-key">${k}</span><span class="me-meta-val">${v}</span>`).join('')}
    </div>
    <div class="me-section-title" style="margin-top:16px">Project Files</div>
    ${projectEmpty
      ? `<div class="me-empty-project">
           <p style="font-size:13px;opacity:0.7;margin:0 0 12px">This project is empty.</p>
           <button class="me-btn me-btn-primary" id="me-init-project" style="width:100%;justify-content:center">Initialize a marmite project</button>
         </div>`
      : `<div class="me-file-tree">${treeHtml}</div>`
    }
  `;

  if (projectEmpty) {
    const initBtn = container.querySelector('#me-init-project');
    if (initBtn) {
      initBtn.addEventListener('click', async () => {
        initBtn.disabled = true;
        initBtn.textContent = 'Initializing...';
        try {
          const resp = await fetch(`${API}/init`, { method: 'POST' });
          const data = await resp.json();
          if (!resp.ok) throw new Error(data.error || 'Init failed');
          toast('Project initialized');
          setTimeout(() => window.location.reload(), 2000);
        } catch (e) {
          toast('Failed: ' + e.message, true);
          initBtn.disabled = false;
          initBtn.textContent = 'Initialize a marmite project';
        }
      });
    }
  }
}

function renderFileTree(files) {
  const tree = {};
  for (const f of files) {
    const parts = f.path.split('/');
    let node = tree;
    for (let i = 0; i < parts.length - 1; i++) {
      if (!node[parts[i]]) node[parts[i]] = {};
      node = node[parts[i]];
    }
    node[parts[parts.length - 1]] = f;
  }

  function renderNode(obj, depth) {
    let html = '';
    const entries = Object.entries(obj).sort(([a], [b]) => {
      const aIsDir = typeof obj[a] === 'object' && !obj[a].path;
      const bIsDir = typeof obj[b] === 'object' && !obj[b].path;
      if (aIsDir && !bIsDir) return -1;
      if (!aIsDir && bIsDir) return 1;
      return a.localeCompare(b);
    });
    for (const [name, value] of entries) {
      if (typeof value === 'object' && !value.path) {
        html += `<details class="me-tree-dir"${depth === 0 ? ' open' : ''}>
          <summary class="me-tree-label me-tree-folder">${name}/</summary>
          <div class="me-tree-children">${renderNode(value, depth + 1)}</div>
        </details>`;
      } else {
        const f = value;
        if (f.slug) {
          const isCurrent = f.slug === slug;
          html += `<a href="${API}/editor/${f.slug}" class="me-tree-label me-tree-file${isCurrent ? ' me-tree-current' : ''}">${name}</a>`;
        } else if (f.fragment || f.editable) {
          const isCurrent = f.path === slug;
          html += `<a href="#" class="me-tree-label me-tree-file me-tree-editable${isCurrent ? ' me-tree-current' : ''}" data-filepath="${f.path}">${name}</a>`;
        } else {
          html += `<span class="me-tree-label me-tree-file me-tree-binary">${name}</span>`;
        }
      }
    }
    return html;
  }

  return renderNode(tree, 0);
}

function renderEditPanel() {
  const container = $('#me-panel-edit');
  if (!container) return;
  const fm = frontmatter || {};
  const esc = (v) => v == null ? '' : String(v).replace(/"/g, '&quot;');

  container.innerHTML = `
    <div class="me-field"><label>Title<input type="text" id="me-edit-title" value="${esc(fm.title)}"></label></div>
    <div class="me-field"><label>Slug<input type="text" id="me-edit-slug" value="${esc(fm.slug)}"></label></div>
    <div class="me-field"><label>Description<textarea id="me-edit-desc">${fm.description || ''}</textarea></label></div>
    <div class="me-field"><label>Date<input type="datetime-local" id="me-edit-date" value="${fm.date ? fm.date.replace(' ', 'T').slice(0, 19) : ''}"></label></div>
    <div class="me-field"><label>Tags (comma-separated)<input type="text" id="me-edit-tags" value="${(Array.isArray(fm.tags) ? fm.tags.join(', ') : (fm.tags || ''))}"></label></div>
    <div class="me-field"><label>Stream<input type="text" id="me-edit-stream" value="${fm.stream || ''}"></label></div>
    <div class="me-field"><label>Series<input type="text" id="me-edit-series" value="${fm.series || ''}"></label></div>
    <div class="me-field"><label>Authors (comma-separated)<input type="text" id="me-edit-authors" value="${(Array.isArray(fm.authors) ? fm.authors.join(', ') : (fm.authors || ''))}"></label></div>
    <div class="me-field"><label>Language<input type="text" id="me-edit-lang" value="${fm.language || ''}"></label></div>
    <div class="me-field"><label>Translates (slug)<input type="text" id="me-edit-translates" value="${fm.translates || ''}"></label></div>
    <div class="me-field"><label>Banner Image<input type="text" id="me-edit-banner" value="${esc(fm.banner_image)}" placeholder="media/banner.jpg"></label></div>
    <div class="me-field"><label>Card Image<input type="text" id="me-edit-card" value="${esc(fm.card_image)}" placeholder="media/card.jpg"></label></div>
    <div class="me-field"><label>Pinned<select id="me-edit-pinned"><option value=""${!fm.pinned ? ' selected' : ''}>No</option><option value="true"${fm.pinned ? ' selected' : ''}>Yes</option></select></label></div>
    <div class="me-field"><label>Comments<select id="me-edit-comments"><option value=""${fm.comments == null ? ' selected' : ''}>Default</option><option value="true"${fm.comments === true ? ' selected' : ''}>Enabled</option><option value="false"${fm.comments === false ? ' selected' : ''}>Disabled</option></select></label></div>
    <div class="me-field"><label>Extra (JSON)<textarea id="me-edit-extra">${fm.extra ? JSON.stringify(fm.extra, null, 2) : ''}</textarea></label></div>
  `;

  if (siteData) {
    if (siteData.tags) createAutocomplete(container.querySelector('#me-edit-tags'), siteData.tags);
    if (siteData.streams) createAutocomplete(container.querySelector('#me-edit-stream'), siteData.streams, (v) => { container.querySelector('#me-edit-stream').value = v; });
    if (siteData.series) createAutocomplete(container.querySelector('#me-edit-series'), siteData.series, (v) => { container.querySelector('#me-edit-series').value = v; });
    if (siteData.authors) createAutocomplete(container.querySelector('#me-edit-authors'), siteData.authors);
    const allLangs = siteData.iso_languages || siteData.languages || [];
    if (allLangs.length) createAutocomplete(container.querySelector('#me-edit-lang'), allLangs, (v) => { container.querySelector('#me-edit-lang').value = v; });
    if (siteData.slugs) createAutocomplete(container.querySelector('#me-edit-translates'), siteData.slugs, (v) => { container.querySelector('#me-edit-translates').value = v; });
    if (siteData.images) {
      createAutocomplete(container.querySelector('#me-edit-banner'), siteData.images, (v) => { container.querySelector('#me-edit-banner').value = v; });
      createAutocomplete(container.querySelector('#me-edit-card'), siteData.images, (v) => { container.querySelector('#me-edit-card').value = v; });
    }
  }
}

function renderActionsPanel() {
  const container = $('#me-panel-actions');
  if (!container) return;
  const fm = frontmatter || {};

  let translationsHtml = '';
  if (fm.translations && fm.translations.length) {
    translationsHtml = `<div style="margin-bottom:8px;font-size:12px;">Existing translations: ${fm.translations.map(t => t.lang || t.name).join(', ')}</div>`;
  }

  container.innerHTML = `
    <div class="me-section-title">Translation</div>
    ${translationsHtml}
    <div class="me-field"><label>Language code<input type="text" id="me-action-trans-lang" placeholder="e.g. pt, es, fr"></label></div>
    <div class="me-field"><label>Title<input type="text" id="me-action-trans-title" placeholder="Translated title"></label></div>
    <button class="me-btn me-btn-primary" style="width:100%;justify-content:center;margin-bottom:12px" id="me-action-translate">Add Translation</button>

    <div class="me-section-title">Clone / Copy</div>
    <div class="me-field"><label>New title<input type="text" id="me-action-clone-title" placeholder="Title for the copy"></label></div>
    <button class="me-btn" style="width:100%;justify-content:center;margin-bottom:12px" id="me-action-clone">Clone Content</button>

    <div class="me-section-title">Danger Zone</div>
    <button class="me-btn" style="width:100%;justify-content:center;border-color:#c0392b;color:#c0392b" id="me-action-delete">Delete Content</button>
  `;

  if (siteData) {
    const allLangs = siteData.iso_languages || siteData.languages || [];
    if (allLangs.length) createAutocomplete(container.querySelector('#me-action-trans-lang'), allLangs, (v) => {
      container.querySelector('#me-action-trans-lang').value = v;
    });
  }

  container.querySelector('#me-action-translate').addEventListener('click', async () => {
    const lang = container.querySelector('#me-action-trans-lang').value.trim();
    const title = container.querySelector('#me-action-trans-title').value.trim();
    if (!lang) { toast('Language code is required', true); return; }
    if (!title) { toast('Title is required', true); return; }
    try {
      const result = await api('POST', '/content', {
        title,
        lang,
        translates: slug,
        tags: (Array.isArray(fm.tags) ? fm.tags.join(', ') : (fm.tags || '')) || undefined,
      });
      toast('Translation created');
      window.location.href = `${API}/editor/${result.slug}`;
    } catch (e) {
      toast('Failed: ' + e.message, true);
    }
  });

  container.querySelector('#me-action-clone').addEventListener('click', async () => {
    const title = container.querySelector('#me-action-clone-title').value.trim();
    if (!title) { toast('Title is required', true); return; }
    try {
      const result = await api('POST', `/content/${slug}/clone`, { title });
      toast('Content cloned');
      window.location.href = `${API}/editor/${result.slug}`;
    } catch (e) {
      toast('Failed: ' + e.message, true);
    }
  });

  container.querySelector('#me-action-delete').addEventListener('click', async () => {
    const ok = await confirmDialog('Delete Content', `Are you sure you want to delete "${fm.title || slug}"? This cannot be undone.`);
    if (!ok) return;
    try {
      await api('DELETE', `/content/${slug}`);
      toast('Content deleted');
      window.location.href = '/';
    } catch (e) {
      toast('Failed: ' + e.message, true);
    }
  });
}

function renderHelpPanel() {
  const container = $('#me-panel-help');
  if (!container) return;

  const shortcodes = (siteData && siteData.shortcodes) ? siteData.shortcodes : [];

  container.innerHTML = `
    <div class="me-help-section">
      <h4>Markdown Syntax</h4>
      <table class="me-help-table">
        <tr><td># Heading 1</td><td>Heading level 1</td></tr>
        <tr><td>## Heading 2</td><td>Heading level 2</td></tr>
        <tr><td>**bold**</td><td>Bold text</td></tr>
        <tr><td>*italic*</td><td>Italic text</td></tr>
        <tr><td>~~strike~~</td><td>Strikethrough</td></tr>
        <tr><td>[text](url)</td><td>Link</td></tr>
        <tr><td>![alt](url)</td><td>Image</td></tr>
        <tr><td>\`code\`</td><td>Inline code</td></tr>
        <tr><td>\`\`\`lang</td><td>Code block</td></tr>
        <tr><td>> quote</td><td>Blockquote</td></tr>
        <tr><td>- item</td><td>Unordered list</td></tr>
        <tr><td>1. item</td><td>Ordered list</td></tr>
        <tr><td>- [ ] task</td><td>Task list</td></tr>
        <tr><td>---</td><td>Horizontal rule</td></tr>
        <tr><td>| a | b |</td><td>Table</td></tr>
        <tr><td>[[slug]]</td><td>Wikilink</td></tr>
        <tr><td>[[Text|slug]]</td><td>Wikilink with text</td></tr>
        <tr><td>> [!NOTE]</td><td>Alert/callout</td></tr>
        <tr><td>||spoiler||</td><td>Spoiler text</td></tr>
        <tr><td>[^1]</td><td>Footnote reference</td></tr>
        <tr><td>$math$</td><td>Inline math</td></tr>
        <tr><td>$$math$$</td><td>Display math</td></tr>
      </table>
    </div>
    <div class="me-help-section">
      <h4>Media</h4>
      <table class="me-help-table">
        <tr><td>@/file.jpg</td><td>Media from content subfolder</td></tr>
        <tr><td>media/file.jpg</td><td>Media from global media dir</td></tr>
      </table>
    </div>
    ${shortcodes.length ? `
    <div class="me-help-section">
      <h4>Available Shortcodes</h4>
      <p style="font-size:12px;margin:0 0 8px">Use: <code>&lt;!-- .name key=value --&gt;</code></p>
      <div class="me-shortcode-list">
        ${shortcodes.map(s => `<span class="me-shortcode-tag">${s}</span>`).join('')}
      </div>
    </div>` : ''}
  `;
}

// --- Sidebar tabs ---
function setupSidebarTabs() {
  $$('.me-sidebar-tab').forEach(tab => {
    tab.addEventListener('click', () => {
      $$('.me-sidebar-tab').forEach(t => t.classList.remove('me-active'));
      $$('.me-sidebar-panel').forEach(p => p.classList.remove('me-active'));
      tab.classList.add('me-active');
      const panel = $(`[data-panel="${tab.dataset.tab}"]`);
      if (panel) panel.classList.add('me-active');
    });
  });
}

// --- Insert menu ---
const INSERT_TEMPLATES = {
  heading: '## ',
  bold: '**text**',
  italic: '*text*',
  link: '[text](url)',
  image: '![alt](url)',
  code: '```\ncode\n```',
  table: '| Column 1 | Column 2 | Column 3 |\n|----------|----------|----------|\n| Cell 1   | Cell 2   | Cell 3   |\n| Cell 4   | Cell 5   | Cell 6   |',
  list: '- Item 1\n- Item 2\n- Item 3',
  quote: '> ',
  hr: '\n---\n',
};

function insertAtCursor(text) {
  if (!editorView) return;
  const cursor = editorView.state.selection.main.head;
  editorView.dispatch({
    changes: { from: cursor, insert: text },
    selection: { anchor: cursor + text.length },
  });
  editorView.focus();
}

function setupInsertMenu() {
  const btn = $('#me-btn-insert');
  const menu = $('#me-insert-menu');

  btn.addEventListener('click', (e) => {
    e.stopPropagation();
    menu.classList.toggle('me-open');
  });

  document.addEventListener('click', () => menu.classList.remove('me-open'));

  $$('.me-dropdown-item').forEach(item => {
    item.addEventListener('click', (e) => {
      e.stopPropagation();
      menu.classList.remove('me-open');
      const action = item.dataset.insert;
      if (action === 'media') {
        showMediaDialog();
      } else if (INSERT_TEMPLATES[action]) {
        insertAtCursor(INSERT_TEMPLATES[action]);
      }
    });
  });
}

// --- Media dialog ---
function showMediaDialog() {
  const images = (siteData && siteData.images) ? siteData.images : [];
  const overlay = document.createElement('div');
  overlay.className = 'me-media-overlay';
  overlay.innerHTML = `
    <div class="me-media-dialog">
      <div class="me-media-header">
        <h4>Insert Media</h4>
        <button class="me-btn me-btn-sm" id="me-media-close">Close</button>
      </div>
      <div class="me-media-search">
        <input type="text" id="me-media-filter" placeholder="Filter images...">
      </div>
      <div class="me-media-grid" id="me-media-grid"></div>
    </div>
  `;
  document.body.appendChild(overlay);

  const grid = overlay.querySelector('#me-media-grid');
  const filterInput = overlay.querySelector('#me-media-filter');

  function renderGrid(filter) {
    const filtered = filter
      ? images.filter(img => img.toLowerCase().includes(filter.toLowerCase()))
      : images;
    grid.innerHTML = filtered.map(img => `
      <div class="me-media-item" data-path="${img}">
        <img src="/${img}" alt="${img}" loading="lazy">
        <div class="me-media-item-name">${img}</div>
      </div>
    `).join('');

    if (filtered.length === 0) {
      grid.innerHTML = '<div style="padding:20px;text-align:center;opacity:0.6;grid-column:1/-1">No images found</div>';
    }
  }

  renderGrid('');

  filterInput.addEventListener('input', () => renderGrid(filterInput.value));
  filterInput.focus();

  grid.addEventListener('click', (e) => {
    const item = e.target.closest('.me-media-item');
    if (!item) return;
    const path = item.dataset.path;
    insertAtCursor(`![](${path})`);
    overlay.remove();
  });

  overlay.querySelector('#me-media-close').addEventListener('click', () => overlay.remove());
  overlay.addEventListener('click', (e) => {
    if (e.target === overlay) overlay.remove();
  });
}

// --- Divider resizing ---
function setupDividers() {
  // Left divider (sidebar | editor)
  setupDivider($('#me-divider-left'), $('#me-sidebar'), 'width', 200, 500);

  // Right divider (editor | preview)
  const rightDivider = $('#me-divider-right');
  const previewPanel = $('#me-preview-panel');

  if (rightDivider && previewPanel) {
    const prefs = getPrefs();
    if (prefs.previewWidth) {
      previewPanel.style.width = prefs.previewWidth + 'px';
      previewPanel.style.flex = 'none';
    }

    rightDivider.addEventListener('mousedown', (e) => {
      e.preventDefault();
      const startX = e.clientX;
      const startWidth = previewPanel.offsetWidth;

      const overlay = document.createElement('div');
      overlay.style.cssText = 'position:fixed;inset:0;z-index:9999;cursor:col-resize;';
      document.body.appendChild(overlay);

      function onMove(ev) {
        const newWidth = startWidth - (ev.clientX - startX);
        const clamped = Math.max(200, Math.min(window.innerWidth * 0.7, newWidth));
        previewPanel.style.width = clamped + 'px';
        previewPanel.style.flex = 'none';
      }

      function onUp() {
        overlay.remove();
        document.removeEventListener('mousemove', onMove);
        document.removeEventListener('mouseup', onUp);
        savePrefs({ previewWidth: previewPanel.offsetWidth });
      }

      document.addEventListener('mousemove', onMove);
      document.addEventListener('mouseup', onUp);
    });
  }
}

function setupDivider(divider, target, prop, min, max) {
  if (!divider || !target) return;

  divider.addEventListener('mousedown', (e) => {
    e.preventDefault();
    const startX = e.clientX;
    const startVal = target.offsetWidth;

    const overlay = document.createElement('div');
    overlay.style.cssText = 'position:fixed;inset:0;z-index:9999;cursor:col-resize;';
    document.body.appendChild(overlay);

    function onMove(ev) {
      const newVal = startVal + (ev.clientX - startX);
      target.style[prop] = Math.max(min, Math.min(max, newVal)) + 'px';
    }

    function onUp() {
      overlay.remove();
      document.removeEventListener('mousemove', onMove);
      document.removeEventListener('mouseup', onUp);
    }

    document.addEventListener('mousemove', onMove);
    document.addEventListener('mouseup', onUp);
  });
}

// --- Top bar ---
function setupTopBar() {
  // Back button
  const backUrl = rawMode ? '/' : `/${slug}.html`;
  $('#me-btn-back').addEventListener('click', () => {
    if (isDirty) {
      confirmDialog('Unsaved Changes', 'You have unsaved changes. Leave anyway?').then(ok => {
        if (ok) window.location.href = backUrl;
      });
    } else {
      window.location.href = backUrl;
    }
  });

  // Save
  $('#me-btn-save').addEventListener('click', saveContent);

  // Save As
  $('#me-btn-saveas').addEventListener('click', saveAs);

  // Sidebar toggle
  const sidebar = $('#me-sidebar');
  const prefs = getPrefs();
  if (prefs.sidebarOpen === false) {
    sidebar.classList.add('me-collapsed');
  }

  $('#me-btn-sidebar').addEventListener('click', () => {
    sidebar.classList.toggle('me-collapsed');
    savePrefs({ sidebarOpen: !sidebar.classList.contains('me-collapsed') });
  });

  // Theme toggle
  $('#me-btn-theme').addEventListener('click', () => {
    const current = document.documentElement.getAttribute('data-theme');
    const next = current === 'dark' ? 'light' : 'dark';
    document.documentElement.setAttribute('data-theme', next);
    localStorage.setItem('marmiteEditorTheme', next);
  });

  // Insert menu
  setupInsertMenu();

  // Preview toggle
  const previewPanel = $('#me-preview-panel');
  const rightDivider = $('#me-divider-right');
  const previewToggle = $('#me-btn-preview-toggle');
  const prefs2 = getPrefs();
  if (prefs2.previewHidden) {
    previewPanel.classList.add('me-hidden');
    rightDivider.classList.add('me-hidden');
  }
  previewToggle.addEventListener('click', () => {
    const hidden = previewPanel.classList.toggle('me-hidden');
    rightDivider.classList.toggle('me-hidden', hidden);
    savePrefs({ previewHidden: hidden });
  });

  // Preview buttons
  $('#me-btn-refresh-preview').addEventListener('click', () => {
    const iframe = $('#me-preview-frame');
    if (iframe) iframe.src = `/${slug}.html?t=${Date.now()}`;
  });

  $('#me-btn-popout').addEventListener('click', () => {
    window.open(`/${slug}.html`, '_blank');
  });

  // Config button
  $('#me-btn-config').addEventListener('click', showConfigDialog);

  // New content button
  $('#me-btn-new').addEventListener('click', showNewContentDialog);
}

// --- Editor controls ---
function setupEditorControls() {
  // Editor theme selector
  const themeSelect = $('#me-editor-theme');
  themeSelect.addEventListener('change', () => {
    const name = themeSelect.value;
    savePrefs({ editorTheme: name });
    if (editorView) {
      editorView.dispatch({
        effects: themeCompartment.reconfigure(getEditorThemeExt(name)),
      });
    }
  });

  // Font size control
  const fontRange = $('#me-font-size');
  const fontLabel = $('#me-font-size-label');
  fontRange.addEventListener('input', () => {
    const size = parseInt(fontRange.value, 10);
    fontLabel.textContent = size + 'px';
    savePrefs({ fontSize: size });
    if (editorView) {
      editorView.dispatch({
        effects: fontSizeCompartment.reconfigure(makeFontSizeTheme(size)),
      });
    }
  });
}

// --- File edit modal ---
function showFileEditModal(filePath) {
  const overlay = document.createElement('div');
  overlay.className = 'me-confirm-overlay';
  overlay.innerHTML = `
    <div class="me-config-dialog" style="max-width:700px">
      <div class="me-config-header">
        <h4 style="font-family:'SF Mono','Fira Code',Menlo,Consolas,monospace;font-size:13px">${filePath}</h4>
        <button class="me-btn me-btn-sm" id="me-file-modal-close">&times;</button>
      </div>
      <div class="me-config-body" style="padding:0;display:flex;flex-direction:column">
        <textarea id="me-file-modal-editor" style="flex:1;width:100%;min-height:400px;border:none;outline:none;resize:none;padding:12px;font-family:'SF Mono','Fira Code',Menlo,Consolas,monospace;font-size:13px;tab-size:2;white-space:pre;line-height:1.5" placeholder="Loading..."></textarea>
      </div>
      <div class="me-config-footer">
        <span style="font-size:11px;opacity:0.6;margin-right:auto" id="me-file-modal-status"></span>
        <button class="me-btn" id="me-file-modal-open-editor">Open in Editor</button>
        <button class="me-btn" id="me-file-modal-cancel">Cancel</button>
        <button class="me-btn me-btn-primary" id="me-file-modal-save">Save</button>
      </div>
    </div>
  `;
  document.body.appendChild(overlay);

  const textarea = overlay.querySelector('#me-file-modal-editor');
  const status = overlay.querySelector('#me-file-modal-status');

  // Load file content
  fetch(`${API}/file/${filePath}`)
    .then(r => r.json())
    .then(data => {
      if (data.error) { status.textContent = data.error; return; }
      textarea.value = data.content || '';
      textarea.placeholder = '';
    })
    .catch(e => { status.textContent = 'Failed to load'; });

  overlay.querySelector('#me-file-modal-close').addEventListener('click', () => overlay.remove());
  overlay.querySelector('#me-file-modal-cancel').addEventListener('click', () => overlay.remove());

  overlay.querySelector('#me-file-modal-open-editor').addEventListener('click', () => {
    overlay.remove();
    window.location.href = `${API}/editor/${filePath}`;
  });

  overlay.querySelector('#me-file-modal-save').addEventListener('click', async () => {
    const content = textarea.value;
    try {
      const resp = await fetch(`${API}/file/${filePath}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content }),
      });
      const data = await resp.json();
      if (!resp.ok) throw new Error(data.error || resp.statusText);
      toast('File saved');
      overlay.remove();
    } catch (e) {
      toast('Save failed: ' + e.message, true);
    }
  });

  // Ctrl+S in the textarea
  textarea.addEventListener('keydown', (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 's') {
      e.preventDefault();
      overlay.querySelector('#me-file-modal-save').click();
    }
  });
}

// --- New content dialog ---
function showNewContentDialog() {
  const overlay = document.createElement('div');
  overlay.className = 'me-confirm-overlay';
  overlay.innerHTML = `
    <div class="me-confirm-box" style="max-width:450px;text-align:left">
      <h4 style="margin-bottom:12px">Create New Content</h4>
      <div class="me-field"><label>Title<input type="text" id="me-new-title" placeholder="Content title"></label></div>
      <div class="me-field"><label>Tags (comma-separated)<input type="text" id="me-new-tags" placeholder="tag1, tag2"></label></div>
      <details style="margin-bottom:12px">
        <summary style="cursor:pointer;font-size:12px;font-weight:600;user-select:none">+ Advanced</summary>
        <div class="me-field" style="margin-top:8px"><label>Stream<input type="text" id="me-new-stream" placeholder="e.g. tutorial, news"></label></div>
        <div class="me-field"><label>Language<input type="text" id="me-new-lang" placeholder="e.g. en, pt, es"></label></div>
        <div class="me-field"><label>Directory<input type="text" id="me-new-directory" placeholder="e.g. tutorials/rust"></label></div>
      </details>
      <div class="me-confirm-actions" style="gap:6px">
        <button class="me-btn" id="me-new-cancel">Cancel</button>
        <button class="me-btn me-btn-primary" id="me-new-post">New Post</button>
        <button class="me-btn" id="me-new-page">New Page</button>
      </div>
    </div>
  `;
  document.body.appendChild(overlay);

  if (siteData && siteData.tags) {
    createAutocomplete(overlay.querySelector('#me-new-tags'), siteData.tags);
  }
  if (siteData && siteData.streams) {
    createAutocomplete(overlay.querySelector('#me-new-stream'), siteData.streams, (v) => {
      overlay.querySelector('#me-new-stream').value = v;
    });
  }

  overlay.querySelector('#me-new-title').focus();
  overlay.querySelector('#me-new-cancel').addEventListener('click', () => overlay.remove());

  async function createContent(isPage) {
    const title = overlay.querySelector('#me-new-title').value.trim();
    if (!title) { toast('Title is required', true); return; }
    const tags = overlay.querySelector('#me-new-tags').value.trim();
    const stream = overlay.querySelector('#me-new-stream').value.trim();
    const lang = overlay.querySelector('#me-new-lang').value.trim();
    const directory = overlay.querySelector('#me-new-directory').value.trim();
    const body = { title };
    if (tags) body.tags = tags;
    if (isPage) body.page = true;
    if (lang) body.lang = lang;
    if (directory) body.directory = directory;
    try {
      const result = await api('POST', '/content', body);
      if (stream) {
        await api('PATCH', `/content/${result.slug}`, { stream });
      }
      toast((isPage ? 'Page' : 'Post') + ' created');
      overlay.remove();
      window.location.href = `${API}/editor/${result.slug}`;
    } catch (e) {
      toast('Failed: ' + e.message, true);
    }
  }

  overlay.querySelector('#me-new-post').addEventListener('click', () => createContent(false));
  overlay.querySelector('#me-new-page').addEventListener('click', () => createContent(true));
}

// --- Config dialog ---
function getNestedValue(obj, keys) {
  let v = obj;
  for (const k of keys) {
    if (v == null || typeof v !== 'object') return undefined;
    v = v[k];
  }
  return v;
}

function showConfigDialog() {
  const cfg = (siteData && siteData.config) ? siteData.config : {};
  const extra = cfg.extra || {};
  const codeHighlight = cfg.code_highlight || {};
  const esc = (v) => v == null ? '' : String(v).replace(/"/g, '&quot;');
  const chk = (v, def) => (v != null ? !!v : !!def) ? ' checked' : '';
  const sel = (v, match) => v === match ? ' selected' : '';
  const menu = cfg.menu || [];

  const COLORSCHEMES = ['', 'catppuccin', 'clean', 'dracula', 'github', 'gruvbox', 'iceberg', 'minimal', 'minimal_wb', 'monokai', 'nord', 'one', 'solarized', 'typewriter'];

  const overlay = document.createElement('div');
  overlay.className = 'me-confirm-overlay';
  overlay.innerHTML = `
    <div class="me-config-dialog">
      <div class="me-config-header">
        <h4>Site Configuration</h4>
        <button class="me-btn me-btn-sm" id="me-config-close">&times;</button>
      </div>
      <div class="me-config-tabs">
        <button class="me-config-tab me-active" data-ctab="site">Site</button>
        <button class="me-config-tab" data-ctab="content">Content</button>
        <button class="me-config-tab" data-ctab="search">Search</button>
        <button class="me-config-tab" data-ctab="feeds">Feeds</button>
        <button class="me-config-tab" data-ctab="appearance">Appearance</button>
        <button class="me-config-tab" data-ctab="images">Images</button>
        <button class="me-config-tab" data-ctab="menu">Menu</button>
        <button class="me-config-tab" data-ctab="paths">Paths</button>
        <button class="me-config-tab" data-ctab="raw">Raw YAML</button>
      </div>
      <div class="me-config-body">
        <div class="me-config-pane me-active" data-cpanel="site">
          <div class="me-field"><label>Site Name<input type="text" data-key="name" value="${esc(cfg.name)}"></label></div>
          <div class="me-field"><label>Tagline<input type="text" data-key="tagline" value="${esc(cfg.tagline)}"></label></div>
          <div class="me-field"><label>Base URL<input type="text" data-key="url" value="${esc(cfg.url)}" placeholder="https://example.com"></label></div>
          <div class="me-field"><label>Language<input type="text" data-key="language" value="${esc(cfg.language)}" placeholder="en"></label></div>
          <div class="me-field"><label>Footer<textarea data-key="footer">${cfg.footer || ''}</textarea></label></div>
          <div class="me-field"><label>Logo Image<input type="text" data-key="logo_image" value="${esc(cfg.logo_image)}" placeholder="media/logo.png"></label></div>
        </div>

        <div class="me-config-pane" data-cpanel="content">
          <div class="me-field"><label>Posts per page<input type="number" data-key="pagination" data-type="number" value="${cfg.pagination || 10}" min="1"></label></div>
          <div class="me-field"><label>Default Author<input type="text" data-key="default_author" value="${esc(cfg.default_author)}"></label></div>
          <div class="me-field"><label>Date Format<input type="text" data-key="default_date_format" value="${esc(cfg.default_date_format)}" placeholder="%b %e, %Y"></label></div>
          <div class="me-field me-check"><label><input type="checkbox" data-key="toc" data-bool${chk(cfg.toc)}> Table of Contents</label></div>
          <div class="me-field me-check"><label><input type="checkbox" data-key="show_next_prev_links" data-bool${chk(cfg.show_next_prev_links, true)}> Next/Previous Links</label></div>
          <div class="me-field me-check"><label><input type="checkbox" data-key="enable_related_content" data-bool${chk(cfg.enable_related_content, true)}> Related Content</label></div>
        </div>

        <div class="me-config-pane" data-cpanel="search">
          <div class="me-field me-check"><label><input type="checkbox" data-key="enable_search" data-bool${chk(cfg.enable_search)}> Enable Search</label></div>
          <div class="me-field me-check"><label><input type="checkbox" data-key="search_show_matches" data-bool${chk(cfg.search_show_matches)}> Show Match Snippets</label></div>
          <div class="me-field"><label>Snippets per Result<input type="number" data-key="search_match_count" data-type="number" value="${cfg.search_match_count || 3}" min="1"></label></div>
          <div class="me-field"><label>Search Page Title<input type="text" data-key="search_title" value="${esc(cfg.search_title)}" placeholder="Search"></label></div>
        </div>

        <div class="me-config-pane" data-cpanel="feeds">
          <div class="me-field me-check"><label><input type="checkbox" data-key="json_feed" data-bool${chk(cfg.json_feed)}> JSON Feed</label></div>
          <div class="me-field me-check"><label><input type="checkbox" data-key="build_sitemap" data-bool${chk(cfg.build_sitemap, true)}> Sitemap</label></div>
          <div class="me-field me-check"><label><input type="checkbox" data-key="publish_urls_json" data-bool${chk(cfg.publish_urls_json, true)}> URLs JSON</label></div>
          <div class="me-field me-check"><label><input type="checkbox" data-key="publish_md" data-bool${chk(cfg.publish_md)}> Publish Markdown Source</label></div>
          <div class="me-field me-check"><label><input type="checkbox" data-key="enable_shortcodes" data-bool${chk(cfg.enable_shortcodes, true)}> Enable Shortcodes</label></div>
        </div>

        <div class="me-config-pane" data-cpanel="appearance">
          <div class="me-field"><label>Colorscheme<select data-key="extra.colorscheme">${COLORSCHEMES.map(c => `<option value="${c}"${sel(extra.colorscheme, c)}>${c || 'Default'}</option>`).join('')}</select></label></div>
          <div class="me-field me-check"><label><input type="checkbox" data-key="extra.colorscheme_toggle" data-bool${chk(extra.colorscheme_toggle)}> Colorscheme Picker</label></div>
          <div class="me-field"><label>Default Color Mode<select data-key="extra.colormode"><option value=""${sel(extra.colormode, undefined)}>Auto</option><option value="light"${sel(extra.colormode, 'light')}>Light</option><option value="dark"${sel(extra.colormode, 'dark')}>Dark</option></select></label></div>
          <div class="me-field me-check"><label><input type="checkbox" data-key="extra.colormodetoggle" data-bool${chk(extra.colormodetoggle)}> Light/Dark Toggle</label></div>
          <div class="me-field me-check"><label><input type="checkbox" data-key="code_highlight.enabled" data-bool${chk(codeHighlight.enabled, true)}> Code Highlighting</label></div>
        </div>

        <div class="me-config-pane" data-cpanel="images">
          <div class="me-field"><label>Card Image<input type="text" data-key="card_image" value="${esc(cfg.card_image)}" placeholder="media/og_image.jpg"></label></div>
          <div class="me-field"><label>Banner Image<input type="text" data-key="banner_image" value="${esc(cfg.banner_image)}" placeholder="media/banner.jpg"></label></div>
          <div class="me-field me-check"><label><input type="checkbox" data-key="skip_image_resize" data-bool${chk(cfg.skip_image_resize)}> Skip Image Resize</label></div>
          <div class="me-field"><label>Max Image Width<input type="number" data-key="extra.max_image_width" data-type="number" value="${extra.max_image_width || ''}" placeholder="1200" min="100"></label></div>
          <div class="me-field"><label>Banner Image Width<input type="number" data-key="extra.banner_image_width" data-type="number" value="${extra.banner_image_width || ''}" placeholder="1200" min="100"></label></div>
          <div class="me-field"><label>Resize Filter<select data-key="extra.resize_filter"><option value=""${sel(extra.resize_filter, undefined)}>Default</option><option value="fast"${sel(extra.resize_filter, 'fast')}>Fast</option><option value="balanced"${sel(extra.resize_filter, 'balanced')}>Balanced</option><option value="quality"${sel(extra.resize_filter, 'quality')}>Quality</option></select></label></div>
        </div>

        <div class="me-config-pane" data-cpanel="menu">
          <p style="font-size:12px;opacity:0.7;margin:0 0 8px">Navigation links in the site header.</p>
          <div id="me-config-menu-list"></div>
          <button class="me-btn me-btn-sm" id="me-config-menu-add" style="margin-top:6px">+ Add Menu Item</button>
        </div>

        <div class="me-config-pane" data-cpanel="paths">
          <div class="me-field"><label>Content Path<input type="text" data-key="content_path" value="${esc(cfg.content_path)}" placeholder="content"></label></div>
          <div class="me-field"><label>Site Path<input type="text" data-key="site_path" value="${esc(cfg.site_path)}" placeholder="site"></label></div>
          <div class="me-field"><label>Media Path<input type="text" data-key="media_path" value="${esc(cfg.media_path)}" placeholder="media"></label></div>
        </div>

        <div class="me-config-pane" data-cpanel="raw">
          <p style="font-size:12px;opacity:0.7;margin:0 0 8px">Edit marmite.yaml directly. Saving here overwrites the entire config file.</p>
          <textarea id="me-config-raw-yaml" style="width:100%;min-height:300px;font-family:'SF Mono','Fira Code',Menlo,Consolas,monospace;font-size:12px;tab-size:2;white-space:pre" placeholder="Loading..."></textarea>
          <button class="me-btn me-btn-primary" id="me-config-raw-save" style="margin-top:8px;width:100%;justify-content:center">Save Raw YAML</button>
        </div>
      </div>
      <div class="me-config-footer">
        <button class="me-btn" id="me-config-cancel">Cancel</button>
        <button class="me-btn me-btn-primary" id="me-config-save">Save Config</button>
      </div>
    </div>
  `;
  document.body.appendChild(overlay);

  // Tab switching
  overlay.querySelectorAll('.me-config-tab').forEach(tab => {
    tab.addEventListener('click', () => {
      overlay.querySelectorAll('.me-config-tab').forEach(t => t.classList.remove('me-active'));
      overlay.querySelectorAll('.me-config-pane').forEach(p => p.classList.remove('me-active'));
      tab.classList.add('me-active');
      overlay.querySelector(`[data-cpanel="${tab.dataset.ctab}"]`).classList.add('me-active');
    });
  });

  // Menu editor
  const menuList = overlay.querySelector('#me-config-menu-list');
  function addMenuRow(label, url) {
    const row = document.createElement('div');
    row.style.cssText = 'display:flex;gap:4px;align-items:center;margin-bottom:4px';
    row.innerHTML = `
      <input type="text" class="me-menu-label" value="${esc(label)}" placeholder="Label" style="flex:1">
      <input type="text" class="me-menu-url" value="${esc(url)}" placeholder="URL" style="flex:1">
      <button class="me-btn me-btn-sm me-menu-del" style="color:#c0392b;border-color:#c0392b">&times;</button>
    `;
    row.querySelector('.me-menu-del').addEventListener('click', () => row.remove());
    menuList.appendChild(row);
  }
  menu.forEach(item => addMenuRow(item[0] || '', item[1] || ''));
  overlay.querySelector('#me-config-menu-add').addEventListener('click', () => addMenuRow('', ''));

  // Raw YAML tab - lazy-load content when tab is first clicked
  let rawLoaded = false;
  const rawTextarea = overlay.querySelector('#me-config-raw-yaml');
  overlay.querySelector('.me-config-tab[data-ctab="raw"]').addEventListener('click', async () => {
    if (rawLoaded) return;
    rawLoaded = true;
    try {
      const resp = await fetch(`${API}/config`);
      const data = await resp.json();
      rawTextarea.value = data.yaml || '';
    } catch (e) {
      rawTextarea.value = '# Failed to load config';
    }
  });

  overlay.querySelector('#me-config-raw-save').addEventListener('click', async () => {
    const yamlContent = rawTextarea.value;
    try {
      const resp = await fetch(`${API}/config`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ yaml: yamlContent }),
      });
      const data = await resp.json();
      if (!resp.ok) throw new Error(data.error || 'Save failed');
      toast('Raw config saved');
      overlay.remove();
      try { siteData = await (await fetch(`${API}/data`)).json(); } catch {}
    } catch (e) {
      toast('Save failed: ' + e.message, true);
    }
  });

  // Image autocomplete
  if (siteData && siteData.images) {
    ['card_image', 'banner_image', 'logo_image'].forEach(key => {
      const input = overlay.querySelector(`[data-key="${key}"]`);
      if (input) createAutocomplete(input, siteData.images, (v) => { input.value = v; });
    });
  }

  // Close/cancel
  const closeDialog = () => overlay.remove();
  overlay.querySelector('#me-config-close').addEventListener('click', closeDialog);
  overlay.querySelector('#me-config-cancel').addEventListener('click', closeDialog);

  // Save
  overlay.querySelector('#me-config-save').addEventListener('click', async () => {
    const updates = {};

    // Simple and nested fields
    overlay.querySelectorAll('[data-key]').forEach(el => {
      const key = el.dataset.key;
      const isNested = key.includes('.');
      let val;

      if (el.type === 'checkbox') {
        val = el.checked;
      } else if (el.tagName === 'SELECT') {
        val = el.value || null;
        if (val === 'true') val = true;
        if (val === 'false') val = false;
      } else if (el.dataset.type === 'number') {
        val = el.value ? parseInt(el.value, 10) : null;
        if (val !== null && isNaN(val)) return;
      } else {
        val = el.value;
      }

      if (isNested) {
        const parts = key.split('.');
        const origVal = getNestedValue(cfg, parts);
        const changed = el.type === 'checkbox'
          ? val !== (origVal != null ? !!origVal : !!(el.dataset.bool !== undefined && el.defaultChecked))
          : (val || '') !== (origVal != null ? String(origVal) : '');
        if (changed) {
          if (!updates[parts[0]]) {
            updates[parts[0]] = Object.assign({}, cfg[parts[0]] || {});
          }
          if (val === null || val === '') {
            delete updates[parts[0]][parts[1]];
          } else {
            updates[parts[0]][parts[1]] = val;
          }
        }
      } else {
        const orig = cfg[key];
        if (el.type === 'checkbox') {
          if (val !== (orig || false)) updates[key] = val;
        } else if (typeof val === 'number') {
          if (val !== (orig || 0)) updates[key] = val;
        } else {
          if ((val || '') !== (orig || '')) updates[key] = val || null;
        }
      }
    });

    // Menu
    const menuItems = [];
    menuList.querySelectorAll('div').forEach(row => {
      const label = row.querySelector('.me-menu-label');
      const url = row.querySelector('.me-menu-url');
      if (label && url) {
        const l = label.value.trim();
        const u = url.value.trim();
        if (l && u) menuItems.push([l, u]);
      }
    });
    if (JSON.stringify(menuItems) !== JSON.stringify(menu)) {
      updates.menu = menuItems;
    }

    if (Object.keys(updates).length === 0) {
      toast('No changes to save');
      overlay.remove();
      return;
    }

    try {
      await api('PATCH', '/config', updates);
      toast('Config saved');
      overlay.remove();
      try { siteData = await (await fetch(`${API}/data`)).json(); } catch {}
    } catch (e) {
      toast('Save failed: ' + e.message, true);
    }
  });
}

// --- Init ---
async function init() {
  // Fetch site data
  try {
    siteData = await (await fetch(`${API}/data`)).json();
    if (siteData && siteData.marmite_version) {
      $('#me-version').textContent = `marmite v${siteData.marmite_version}`;
    }
  } catch (e) { /* ok */ }

  // Fetch file tree
  try {
    const ftData = await (await fetch(`${API}/files`)).json();
    fileTree = ftData.files || [];
  } catch { /* ok */ }

  // Fetch content - different paths for empty, raw, and content mode
  let body = '';
  if (emptyMode) {
    // No content to load
  } else if (rawMode) {
    try {
      const data = await (await fetch(`${API}/file/${slug}`)).json();
      if (data.error) throw new Error(data.error);
      body = data.content || '';
      sourcePath = slug;
      originalBody = body;
    } catch (e) {
      toast('Failed to load file: ' + e.message, true);
      return;
    }
  } else {
    try {
      const data = await api('GET', `/content/${slug}/body`);
      frontmatter = data.frontmatter || {};
      sourcePath = data.source_path || '';
      body = data.body || '';
      originalBody = body;
    } catch (e) {
      toast('Failed to load content: ' + e.message, true);
      return;
    }
  }

  // Check for auto-saved draft
  if (!emptyMode) {
    try {
      const saved = JSON.parse(localStorage.getItem(AUTOSAVE_KEY) || 'null');
      if (saved && saved.isDraft && saved.body && saved.body !== body) {
        const age = Date.now() - (saved.timestamp || 0);
        if (age < 86400000) {
          const restore = await confirmDialog(
            'Restore Draft',
            'A more recent unsaved draft was found in your browser. Restore it?'
          );
          if (restore) {
            body = saved.body;
            isDirty = true;
          }
        }
      }
    } catch { /* ignore */ }
  }

  // Set up the page
  if (emptyMode) {
    $('#me-content-type').textContent = '';
    $('#me-content-title').textContent = 'Marmite Editor';
    $('#me-preview-panel').classList.add('me-hidden');
    $('#me-divider-right').classList.add('me-hidden');
    $$('.me-sidebar-tab').forEach(t => {
      if (t.dataset.tab === 'edit' || t.dataset.tab === 'actions') {
        t.style.display = 'none';
      }
    });
    // Hide editor toolbar save/saveas/insert (nothing to save)
    $('#me-editor-toolbar').style.display = 'none';
  } else if (rawMode) {
    $('#me-content-type').textContent = 'file';
    $('#me-content-title').textContent = slug;
    $('#me-preview-panel').classList.add('me-hidden');
    $('#me-divider-right').classList.add('me-hidden');
    $$('.me-sidebar-tab').forEach(t => {
      if (t.dataset.tab === 'edit' || t.dataset.tab === 'actions') {
        t.style.display = 'none';
      }
    });
  } else {
    const isPost = !!frontmatter.date;
    $('#me-content-type').textContent = isPost ? 'post' : 'page';
    $('#me-content-title').textContent = frontmatter.title || slug;
    $('#me-preview-frame').src = `/${slug}.html`;
  }
  updateDirtyIndicator();
  connectLiveReload();

  // Setup UI
  setupSidebarTabs();
  setupTopBar();
  setupEditorControls();
  setupDividers();

  // Render sidebar panels
  renderInfoPanel();
  if (!rawMode && !emptyMode) {
    renderEditPanel();
    renderActionsPanel();
  }
  renderHelpPanel();

  // Delegate clicks on editable file tree items to open modal
  $('#me-panel-info').addEventListener('click', (e) => {
    const link = e.target.closest('.me-tree-editable');
    if (link) {
      e.preventDefault();
      showFileEditModal(link.dataset.filepath);
    }
  });

  // Create editor (skip in emptyMode - show placeholder instead)
  if (emptyMode) {
    $('#me-editor-container').innerHTML = '<div style="display:flex;align-items:center;justify-content:center;height:100%;opacity:0.5;font-size:14px">Select a file from the sidebar to start editing</div>';
  } else {
    createEditor(body);
  }
}

// Keyboard: Escape to close dropdowns
document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') {
    $$('.me-dropdown-menu.me-open').forEach(m => m.classList.remove('me-open'));
    $$('.me-media-overlay').forEach(o => o.remove());
  }
});

// Warn before leaving with unsaved changes
window.addEventListener('beforeunload', (e) => {
  if (isDirty) {
    e.preventDefault();
    e.returnValue = '';
  }
});

init();
