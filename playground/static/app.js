import { EditorView, basicSetup } from "codemirror";
import { EditorState, Compartment } from "@codemirror/state";
import { markdown } from "@codemirror/lang-markdown";
import { yaml } from "@codemirror/lang-yaml";
import { oneDark } from "@codemirror/theme-one-dark";
import {
  solarizedLight, dracula, cobalt, coolGlow,
  amy, espresso, clouds, tomorrow,
  noctisLilac, rosePineDawn, ayuLight,
} from "thememirror";
import jsyaml from "js-yaml";

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

const $ = (sel) => document.querySelector(sel);
const statusEl = $("#status");
const themeToggle = $("#theme-toggle");
const editorThemeSelect = $("#editor-theme");
const newFileBtn = $("#new-file-btn");
const configBtn = $("#config-btn");
const downloadBtn = $("#download-btn");
const downloadMenu = $("#download-menu");
const downloadSourceBtn = $("#download-source");
const downloadSiteBtn = $("#download-site");
const uploadBtn = $("#upload-btn");
const uploadInput = $("#upload-input");
const newSessionBtn = $("#new-session-btn");
const tabsEl = $("#tabs");
const editorContainer = $("#editor-container");
const previewFrame = $("#preview-frame");
const previewPlaceholder = $("#preview-placeholder");
const outputPanel = $("#output-panel");
const outputContent = $("#output-content");
const outputClose = $("#output-close");
const viewOnlyBadge = $("#view-only-badge");
const editorFileInfo = $("#editor-file-info");
const newFileDialog = $("#new-file-dialog");
const newFileFolderSelect = $("#new-file-folder");
const newFileNameInput = $("#new-file-name");
const newFileAddBtn = $("#new-file-add");
const newFileCancelBtn = $("#new-file-cancel");

let sessionId = null;
let ownerToken = null;
let isOwner = false;
let files = {};
let currentFile = null;
let editorView = null;
let dirty = new Set();

const themeCompartment = new Compartment();
const readOnlyCompartment = new Compartment();

function getEditorThemeName() {
  return localStorage.getItem("playground_editor_theme") || "dracula";
}

function getEditorThemeExt(name) {
  const entry = editorThemes[name];
  return entry ? entry.ext : dracula;
}

function applyEditorTheme(name) {
  localStorage.setItem("playground_editor_theme", name);
  editorThemeSelect.value = name;
  if (editorView) {
    editorView.dispatch({
      effects: themeCompartment.reconfigure(getEditorThemeExt(name)),
    });
  }
}

editorThemeSelect.value = getEditorThemeName();
editorThemeSelect.addEventListener("change", () => {
  applyEditorTheme(editorThemeSelect.value);
});

function isDark() {
  return document.documentElement.getAttribute("data-theme") !== "light";
}

function updateThemeIcon() {
  themeToggle.textContent = isDark() ? "☀️" : "🌙";
}

function toggleTheme() {
  const next = isDark() ? "light" : "dark";
  document.documentElement.setAttribute("data-theme", next);
  localStorage.setItem("playground_theme", next);
  updateThemeIcon();
}

updateThemeIcon();
themeToggle.addEventListener("click", toggleTheme);

function setStatus(msg, color) {
  statusEl.textContent = msg;
  statusEl.style.color = color || "";
}

function updateOwnerUI() {
  newFileBtn.style.display = isOwner ? "" : "none";
  configBtn.style.display = isOwner ? "" : "none";
  uploadBtn.style.display = isOwner ? "" : "none";
  newSessionBtn.style.display = isOwner ? "none" : "";
  viewOnlyBadge.style.display = isOwner ? "none" : "";
  document.querySelectorAll(".tab-delete").forEach((btn) => {
    btn.style.display = isOwner ? "" : "none";
  });

  if (!isOwner) {
    previewPlaceholder.style.display = "none";
    previewFrame.classList.add("visible");
    previewFrame.src = `/preview/${sessionId}/`;
  }
}

async function api(method, path, body) {
  const opts = { method, headers: {} };
  if (ownerToken) {
    opts.headers["X-Owner-Token"] = ownerToken;
  }
  if (body !== undefined) {
    opts.headers["Content-Type"] = "application/json";
    opts.body = JSON.stringify(body);
  }
  const res = await fetch(path, opts);
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || res.statusText);
  }
  return res.json();
}

function getSessionFromHash() {
  const hash = location.hash.replace("#", "").trim();
  if (hash && hash.match(/^[0-9a-f-]{36}$/)) return hash;
  return null;
}

async function initSession() {
  const hashId = getSessionFromHash();

  if (hashId) {
    try {
      const data = await api("GET", `/api/sessions/${hashId}`);
      sessionId = data.session_id;
      ownerToken = localStorage.getItem(`playground_owner_${sessionId}`);
      isOwner = !!ownerToken;
      return data.files;
    } catch {
      // Session expired or invalid, fall through to create new
    }
  }

  // Check localStorage for a recent session
  const stored = localStorage.getItem("playground_session");
  if (stored && !hashId) {
    try {
      const data = await api("GET", `/api/sessions/${stored}`);
      sessionId = data.session_id;
      ownerToken = localStorage.getItem(`playground_owner_${sessionId}`);
      isOwner = !!ownerToken;
      location.hash = sessionId;
      return data.files;
    } catch {
      localStorage.removeItem("playground_session");
    }
  }

  return await createNewSession();
}

async function createNewSession() {
  setStatus("Creating session...");
  const data = await api("POST", "/api/sessions");
  sessionId = data.session_id;
  ownerToken = data.owner_token;
  isOwner = true;
  localStorage.setItem("playground_session", sessionId);
  localStorage.setItem(`playground_owner_${sessionId}`, ownerToken);
  location.hash = sessionId;
  return data.files;
}

async function loadFile(path) {
  const data = await api("GET", `/api/sessions/${sessionId}/files/${path}`);
  return data.content;
}

async function saveFile(path, content) {
  await api("PUT", `/api/sessions/${sessionId}/files/${path}`, { content });
}

async function deleteFile(path) {
  await api("DELETE", `/api/sessions/${sessionId}/files/${path}`);
}

function langExtension(path) {
  if (path.endsWith(".yaml") || path.endsWith(".yml")) return yaml();
  return markdown();
}

function createEditor(content, path) {
  if (editorView) {
    editorView.destroy();
  }

  const state = EditorState.create({
    doc: content,
    extensions: [
      basicSetup,
      langExtension(path),
      themeCompartment.of(getEditorThemeExt(getEditorThemeName())),
      readOnlyCompartment.of([
        EditorView.editable.of(isOwner),
        EditorState.readOnly.of(!isOwner),
      ]),
      EditorView.updateListener.of((update) => {
        if (update.docChanged && currentFile && isOwner) {
          dirty.add(currentFile);
          files[currentFile] = update.state.doc.toString();
          updateTabStates();
          scheduleAutoRender();
        }
      }),
    ],
  });

  editorView = new EditorView({
    state,
    parent: editorContainer,
  });
}

function updateTabStates() {
  tabsEl.querySelectorAll(".tab-btn").forEach((btn) => {
    const path = btn.dataset.path;
    btn.classList.toggle("active", path === currentFile);
    btn.classList.toggle("dirty", dirty.has(path));
  });
}

function switchTab(path) {
  if (currentFile && editorView) {
    files[currentFile] = editorView.state.doc.toString();
  }
  currentFile = path;
  createEditor(files[path] || "", path);
  updateTabStates();
  editorFileInfo.textContent = path;
}

function buildTabs(filePaths) {
  tabsEl.innerHTML = "";
  for (const path of filePaths) {
    const tab = document.createElement("div");
    tab.className = "tab";

    const btn = document.createElement("button");
    btn.className = "tab-btn";
    btn.textContent = path;
    btn.dataset.path = path;
    btn.addEventListener("click", () => switchTab(path));
    tab.appendChild(btn);

    if (isOwner) {
      const del = document.createElement("button");
      del.className = "tab-delete";
      del.textContent = "×";
      del.title = `Delete ${path}`;
      del.addEventListener("click", async (e) => {
        e.stopPropagation();
        if (!confirm(`Delete ${path}?`)) return;
        try {
          await deleteFile(path);
          delete files[path];
          dirty.delete(path);
          await refreshFileList();
          doRender();
        } catch (err) {
          setStatus(`Delete failed: ${err.message}`, "var(--red)");
        }
      });
      tab.appendChild(del);
    }

    tabsEl.appendChild(tab);
  }
}

async function refreshFileList() {
  const data = await api("GET", `/api/sessions/${sessionId}/files`);
  const filePaths = data.files;

  // Load any new files we don't have yet
  for (const path of filePaths) {
    if (!(path in files)) {
      files[path] = await loadFile(path);
    }
  }

  // Remove files that no longer exist
  for (const path of Object.keys(files)) {
    if (!filePaths.includes(path)) {
      delete files[path];
    }
  }

  buildTabs(filePaths);

  if (!filePaths.includes(currentFile) && filePaths.length > 0) {
    switchTab(filePaths[0]);
  } else {
    updateTabStates();
  }
}

async function saveAllDirty() {
  if (currentFile && editorView) {
    files[currentFile] = editorView.state.doc.toString();
  }

  const saves = [...dirty].map((path) =>
    saveFile(path, files[path]).then(() => dirty.delete(path))
  );
  await Promise.all(saves);
  updateTabStates();
}

let autoRenderTimer = null;
let rendering = false;

function scheduleAutoRender() {
  if (autoRenderTimer) clearTimeout(autoRenderTimer);
  autoRenderTimer = setTimeout(() => doRender(), 1500);
}

async function doRender() {
  if (rendering) return;
  rendering = true;
  setStatus("Saving...");

  try {
    await saveAllDirty();
    setStatus("Rendering...");

    const result = await api("POST", `/api/sessions/${sessionId}/render`);

    if (result.success) {
      setStatus(`Rendered in ${result.duration_ms}ms`, "var(--green)");
      previewPlaceholder.style.display = "none";
      previewFrame.classList.add("visible");
      previewFrame.src = `/preview/${sessionId}/?t=${Date.now()}`;
    } else {
      setStatus("Render failed", "var(--red)");
      showOutput(result.stderr || result.stdout);
    }
  } catch (err) {
    setStatus(`Error: ${err.message}`, "var(--red)");
  } finally {
    rendering = false;
  }
}

function showOutput(text) {
  outputContent.textContent = text;
  outputPanel.classList.remove("hidden");
}

outputClose.addEventListener("click", () => {
  outputPanel.classList.add("hidden");
});

// New file dialog
const newFileHelp = $("#new-file-help");

const helpTexts = {
  content:
    "<b>Post:</b> <code>2024-06-15-my-post.md</code> - dated, appears in listings<br>" +
    "<b>Page:</b> <code>about.md</code> - standalone, no date<br>" +
    "<b>Fragment:</b> <code>_hero.md</code> - reusable snippet, no frontmatter needed<br>" +
    "<a href='https://marmite.blog/using-markdown-to-customize-layout.html' target='_blank'>Content reference</a>",
  static:
    "<b>CSS:</b> <code>custom.css</code> - custom styles<br>" +
    "<b>JS:</b> <code>custom.js</code> - custom scripts<br>" +
    "Files are served at <code>/static/filename</code>",
  templates:
    "<b>Override:</b> <code>base.html</code>, <code>list.html</code>, <code>content.html</code><br>" +
    "<b>Partials:</b> <code>custom_header.html</code>, <code>custom_footer.html</code><br>" +
    "<a href='https://marmite.blog/template-reference.html' target='_blank'>Template reference</a>",
};

function updateHelp() {
  newFileHelp.innerHTML = helpTexts[newFileFolderSelect.value] || "";
}

newFileBtn.addEventListener("click", () => {
  newFileDialog.classList.remove("hidden");
  newFileNameInput.value = "";
  updateHelp();
  newFileNameInput.focus();
});

newFileFolderSelect.addEventListener("change", updateHelp);

newFileCancelBtn.addEventListener("click", () => {
  newFileDialog.classList.add("hidden");
});

function generateContent(folder, name) {
  if (folder !== "content") return "";
  if (name.startsWith("_")) return "";

  const title = name
    .replace(/\.md$/, "")
    .replace(/^\d{4}-\d{2}-\d{2}-?/, "")
    .replace(/[-_]/g, " ")
    .replace(/\b\w/g, (c) => c.toUpperCase());

  const hasDate = /^\d{4}-\d{2}-\d{2}/.test(name);
  const lines = ["---", `title: ${title}`];
  if (hasDate) {
    lines.push(`date: ${name.slice(0, 10)}`);
    lines.push("tags: ");
  }
  lines.push("---", "", "");
  return lines.join("\n");
}

newFileAddBtn.addEventListener("click", async () => {
  const folder = newFileFolderSelect.value;
  const name = newFileNameInput.value.trim();
  if (!name) return;

  const path = folder ? `${folder}/${name}` : name;
  const content = generateContent(folder, name);
  newFileDialog.classList.add("hidden");

  try {
    await saveFile(path, content);
    files[path] = content;
    await refreshFileList();
    switchTab(path);
    doRender();
  } catch (err) {
    setStatus(`Failed to create file: ${err.message}`, "var(--red)");
  }
});

newFileNameInput.addEventListener("keydown", (e) => {
  if (e.key === "Enter") {
    e.preventDefault();
    newFileAddBtn.click();
  }
  if (e.key === "Escape") {
    newFileDialog.classList.add("hidden");
  }
});

// Clone session button
newSessionBtn.addEventListener("click", async () => {
  try {
    setStatus("Cloning session...");
    const data = await api("POST", `/api/sessions/${sessionId}/clone`);
    sessionId = data.session_id;
    ownerToken = data.owner_token;
    isOwner = true;
    localStorage.setItem("playground_session", sessionId);
    localStorage.setItem(`playground_owner_${sessionId}`, ownerToken);
    location.hash = sessionId;

    files = {};
    dirty.clear();
    const filePaths = data.files;
    for (const path of filePaths) {
      files[path] = await loadFile(path);
    }
    updateOwnerUI();
    buildTabs(filePaths);
    if (filePaths.length > 0) switchTab(filePaths[0]);
    previewFrame.classList.remove("visible");
    previewPlaceholder.style.display = "";
    doRender();
  } catch (err) {
    setStatus(`Clone failed: ${err.message}`, "var(--red)");
  }
});

// Mobile panel toggle
const showEditorBtn = $("#show-editor");
const showPreviewBtn = $("#show-preview");
const appEl = $("#app");

showEditorBtn.addEventListener("click", () => {
  appEl.classList.remove("show-preview");
  showEditorBtn.classList.add("active");
  showPreviewBtn.classList.remove("active");
});

showPreviewBtn.addEventListener("click", () => {
  appEl.classList.add("show-preview");
  showPreviewBtn.classList.add("active");
  showEditorBtn.classList.remove("active");
});

// Download
downloadBtn.addEventListener("click", (e) => {
  e.stopPropagation();
  downloadMenu.classList.toggle("hidden");
});

document.addEventListener("click", () => {
  downloadMenu.classList.add("hidden");
});

async function triggerDownload(url, filename) {
  setStatus("Preparing download...");
  try {
    const res = await fetch(url, {
      headers: ownerToken ? { "X-Owner-Token": ownerToken } : {},
    });
    if (!res.ok) throw new Error("Download failed");
    const blob = await res.blob();
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = filename;
    a.click();
    URL.revokeObjectURL(a.href);
    setStatus("Downloaded", "var(--green)");
  } catch (err) {
    setStatus(`Download failed: ${err.message}`, "var(--red)");
  }
}

downloadSourceBtn.addEventListener("click", (e) => {
  e.stopPropagation();
  downloadMenu.classList.add("hidden");
  triggerDownload(
    `/api/sessions/${sessionId}/download/source`,
    "marmite-source.tar.gz"
  );
});

downloadSiteBtn.addEventListener("click", (e) => {
  e.stopPropagation();
  downloadMenu.classList.add("hidden");
  triggerDownload(
    `/api/sessions/${sessionId}/download/site`,
    "marmite-site.tar.gz"
  );
});

// Upload
uploadBtn.addEventListener("click", () => {
  uploadInput.click();
});

uploadInput.addEventListener("change", async () => {
  const file = uploadInput.files[0];
  if (!file) return;
  uploadInput.value = "";

  setStatus("Uploading...");
  try {
    const formData = new FormData();
    formData.append("file", file);

    const res = await fetch(`/api/sessions/${sessionId}/upload`, {
      method: "POST",
      headers: ownerToken ? { "X-Owner-Token": ownerToken } : {},
      body: formData,
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: res.statusText }));
      throw new Error(err.error || res.statusText);
    }

    const data = await res.json();
    files = {};
    dirty.clear();
    for (const path of data.files) {
      files[path] = await loadFile(path);
    }
    buildTabs(data.files);
    if (data.files.length > 0) switchTab(data.files[0]);
    doRender();
  } catch (err) {
    setStatus(`Upload failed: ${err.message}`, "var(--red)");
  }
});

// Config form
const configDialog = $("#config-dialog");
const configClose = $("#config-close");
const configCancel = $("#config-cancel");
const configSave = $("#config-save");
const cfgMenuList = $("#cfg-menu-list");
const cfgMenuAdd = $("#cfg-menu-add");

const CONFIG_FILE = "marmite.yaml";

const SIMPLE_FIELDS = [
  "name", "tagline", "url", "language", "footer", "logo_image",
  "pagination", "default_author", "default_date_format",
  "toc", "show_next_prev_links", "enable_related_content",
  "enable_search", "search_show_matches", "search_match_count", "search_title",
  "json_feed", "build_sitemap", "publish_urls_json", "publish_md", "enable_shortcodes",
  "card_image", "banner_image", "skip_image_resize",
];

const NESTED_FIELDS = {
  "extra.colorscheme": ["extra", "colorscheme"],
  "extra.colorscheme_toggle": ["extra", "colorscheme_toggle"],
  "extra.colormode": ["extra", "colormode"],
  "extra.colormodetoggle": ["extra", "colormodetoggle"],
  "extra.max_image_width": ["extra", "max_image_width"],
  "extra.banner_image_width": ["extra", "banner_image_width"],
  "extra.resize_filter": ["extra", "resize_filter"],
  "code_highlight.enabled": ["code_highlight", "enabled"],
};

function getNestedValue(obj, keys) {
  let v = obj;
  for (const k of keys) {
    if (v == null || typeof v !== "object") return undefined;
    v = v[k];
  }
  return v;
}

function setNestedValue(obj, keys, value) {
  for (let i = 0; i < keys.length - 1; i++) {
    if (obj[keys[i]] == null || typeof obj[keys[i]] !== "object") {
      obj[keys[i]] = {};
    }
    obj = obj[keys[i]];
  }
  obj[keys[keys.length - 1]] = value;
}

function deleteNestedValue(obj, keys) {
  for (let i = 0; i < keys.length - 1; i++) {
    if (obj[keys[i]] == null) return;
    obj = obj[keys[i]];
  }
  delete obj[keys[keys.length - 1]];
}

function populateConfigForm(cfg) {
  for (const key of SIMPLE_FIELDS) {
    const el = configDialog.querySelector(`[data-key="${key}"]`);
    if (!el) continue;
    const val = cfg[key];
    if (el.type === "checkbox") {
      el.checked = val === true;
    } else {
      el.value = val != null ? val : "";
    }
  }

  for (const [dataKey, path] of Object.entries(NESTED_FIELDS)) {
    const el = configDialog.querySelector(`[data-key="${dataKey}"]`);
    if (!el) continue;
    const val = getNestedValue(cfg, path);
    if (el.type === "checkbox") {
      el.checked = val === true;
    } else {
      el.value = val != null ? val : "";
    }
  }

  cfgMenuList.innerHTML = "";
  const menu = cfg.menu || [];
  for (const item of menu) {
    addMenuRow(item[0] || "", item[1] || "");
  }
}

function addMenuRow(label, url) {
  const row = document.createElement("div");
  row.className = "cfg-menu-row";
  row.innerHTML = `<input type="text" placeholder="Label" value="${label.replace(/"/g, "&quot;")}">` +
    `<input type="text" placeholder="URL" value="${url.replace(/"/g, "&quot;")}">` +
    `<button class="cfg-menu-del" title="Remove">&times;</button>`;
  row.querySelector(".cfg-menu-del").addEventListener("click", () => row.remove());
  cfgMenuList.appendChild(row);
}

cfgMenuAdd.addEventListener("click", () => addMenuRow("", ""));

function collectConfigForm(original) {
  const cfg = {};

  for (const key of SIMPLE_FIELDS) {
    const el = configDialog.querySelector(`[data-key="${key}"]`);
    if (!el) continue;
    let val;
    if (el.type === "checkbox") {
      val = el.checked;
    } else if (el.type === "number") {
      val = el.value ? Number(el.value) : undefined;
    } else {
      val = el.value || undefined;
    }
    if (val !== undefined) cfg[key] = val;
  }

  for (const [dataKey, path] of Object.entries(NESTED_FIELDS)) {
    const el = configDialog.querySelector(`[data-key="${dataKey}"]`);
    if (!el) continue;
    let val;
    if (el.type === "checkbox") {
      val = el.checked;
    } else if (el.type === "number") {
      val = el.value ? Number(el.value) : undefined;
    } else {
      val = el.value || undefined;
    }
    if (val !== undefined) {
      setNestedValue(cfg, path, val);
    }
  }

  const menuRows = cfgMenuList.querySelectorAll(".cfg-menu-row");
  if (menuRows.length > 0) {
    cfg.menu = [];
    menuRows.forEach((row) => {
      const inputs = row.querySelectorAll("input");
      const label = inputs[0].value.trim();
      const url = inputs[1].value.trim();
      if (label || url) cfg.menu.push([label, url]);
    });
  }

  // Preserve fields not in the form (authors, streams, series, markdown_parser, etc.)
  const preserved = ["authors", "streams", "series", "file_mapping", "galleries",
    "theme", "source_repository", "shortcode_pattern", "markdown_parser",
    "pages_title", "tags_title", "tags_content_title", "archives_title",
    "archives_content_title", "authors_title", "streams_title",
    "streams_content_title", "series_title", "series_content_title",
    "content_path", "templates_path", "static_path", "media_path",
    "gallery_path", "site_path", "gallery_create_thumbnails", "gallery_thumb_size",
    "image_provider", "https"];
  for (const key of preserved) {
    if (original[key] !== undefined && cfg[key] === undefined) {
      cfg[key] = original[key];
    }
  }

  // Preserve extra keys not in the form
  if (original.extra) {
    if (!cfg.extra) cfg.extra = {};
    const formExtraKeys = ["colorscheme", "colorscheme_toggle", "colormode",
      "colormodetoggle", "max_image_width", "banner_image_width", "resize_filter"];
    for (const [k, v] of Object.entries(original.extra)) {
      if (!formExtraKeys.includes(k) && cfg.extra[k] === undefined) {
        cfg.extra[k] = v;
      }
    }
  }

  return cfg;
}

configBtn.addEventListener("click", () => {
  const yamlContent = files[CONFIG_FILE] || "";
  let cfg = {};
  try {
    cfg = jsyaml.load(yamlContent) || {};
  } catch {
    cfg = {};
  }
  populateConfigForm(cfg);
  configDialog.classList.remove("hidden");

  // Activate first tab
  configDialog.querySelectorAll(".config-tab").forEach((t) => t.classList.remove("active"));
  configDialog.querySelectorAll(".config-pane").forEach((p) => p.classList.remove("active"));
  configDialog.querySelector('.config-tab[data-tab="site"]').classList.add("active");
  configDialog.querySelector('.config-pane[data-tab="site"]').classList.add("active");
});

configDialog.querySelectorAll(".config-tab").forEach((tab) => {
  tab.addEventListener("click", () => {
    configDialog.querySelectorAll(".config-tab").forEach((t) => t.classList.remove("active"));
    configDialog.querySelectorAll(".config-pane").forEach((p) => p.classList.remove("active"));
    tab.classList.add("active");
    configDialog.querySelector(`.config-pane[data-tab="${tab.dataset.tab}"]`).classList.add("active");
  });
});

configClose.addEventListener("click", () => configDialog.classList.add("hidden"));
configCancel.addEventListener("click", () => configDialog.classList.add("hidden"));

configSave.addEventListener("click", async () => {
  const originalYaml = files[CONFIG_FILE] || "";
  let original = {};
  try {
    original = jsyaml.load(originalYaml) || {};
  } catch {
    original = {};
  }

  const cfg = collectConfigForm(original);
  const newYaml = jsyaml.dump(cfg, { lineWidth: -1, quotingType: '"', forceQuotes: false });

  files[CONFIG_FILE] = newYaml;
  dirty.add(CONFIG_FILE);

  configDialog.classList.add("hidden");

  if (currentFile === CONFIG_FILE && editorView) {
    editorView.dispatch({
      changes: { from: 0, to: editorView.state.doc.length, insert: newYaml },
    });
  }

  scheduleAutoRender();
  setStatus("Config saved", "var(--green)");
});

// Resizable divider
const divider = $("#divider");
const editorPanel = $("#editor-panel");

divider.addEventListener("mousedown", (e) => {
  e.preventDefault();
  const startX = e.clientX;
  const startWidth = editorPanel.offsetWidth;

  function onMove(e) {
    const newWidth = startWidth + (e.clientX - startX);
    editorPanel.style.width = `${Math.max(200, newWidth)}px`;
  }

  function onUp() {
    document.removeEventListener("mousemove", onMove);
    document.removeEventListener("mouseup", onUp);
  }

  document.addEventListener("mousemove", onMove);
  document.addEventListener("mouseup", onUp);
});

// Init
(async () => {
  try {
    setStatus("Loading...");
    const filePaths = await initSession();
    updateOwnerUI();
    buildTabs(filePaths);

    for (const path of filePaths) {
      files[path] = await loadFile(path);
    }

    if (filePaths.length > 0) {
      switchTab(filePaths[0]);
    }

    if (isOwner) {
      doRender();
    } else {
      setStatus("Ready");
    }
  } catch (err) {
    setStatus(`Failed to initialize: ${err.message}`, "var(--red)");
  }
})();
