use chrono::Utc;
use log::{error, info, warn};
use serde_json::json;
use std::fmt::Write as _;
use std::io::{Cursor, ErrorKind};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::{fs::File, thread};
use tiny_http::{Header, Method, Request, Response, Server};
use tungstenite::handshake::derive_accept_key;
use tungstenite::protocol::Role;
use tungstenite::Error as WsError;
use tungstenite::Message;
use urlencoding::decode;

pub struct ServerContext {
    pub output_folder: Arc<PathBuf>,
    pub input_folder: Arc<PathBuf>,
    pub config_path: Arc<PathBuf>,
}

const FALLBACK_BIND_ADDRESS: &str = "0.0.0.0:0";
const LIVE_RELOAD_SCRIPT_PATH: &str = "__marmite__/livereload.js";
const TOOLBAR_JS_PATH: &str = "__marmite__/toolbar.js";
const TOOLBAR_CSS_PATH: &str = "__marmite__/toolbar.css";
const LIVE_RELOAD_WS_PATH: &str = "/__marmite__/livereload";
const CONTENT_API_PATH: &str = "/__marmite__/content";
const CONFIG_API_PATH: &str = "/__marmite__/config";
const DATA_API_PATH: &str = "/__marmite__/data";
const LIVE_RELOAD_SCRIPT: &str = r#"(() => {
    const isHttps = window.location.protocol === "https:";
    const hostPart = window.location.hostname.includes(":") ? `[${window.location.hostname}]` : window.location.hostname;
    const wsProtocol = isHttps ? "wss" : "ws";
    const portSegment = window.location.port ? `:${window.location.port}` : "";
    const wsPath = "/__marmite__/livereload";
    const wsUrl = `${wsProtocol}://${hostPart}${portSegment}${wsPath}`;

    const connect = () => {
        const socket = new WebSocket(wsUrl);
        socket.addEventListener("message", (event) => {
            try {
                const payload = JSON.parse(event.data);
                if (payload.event === "reload") {
                    console.log("Live reload triggered, reloading page...");
                    window.location.reload();
                }
            } catch (err) {
                console.warn("Failed to parse live reload payload", err);
            }
        });
        socket.addEventListener("close", () => {
            setTimeout(connect, 2000);
        });
        socket.addEventListener("error", () => {
            socket.close();
        });
    };

    connect();
})();"#;

const TOOLBAR_CSS: &str = r#"
#mt-toolbar-btn {
  position: fixed;
  top: 12px;
  left: 12px;
  z-index: 999990;
  width: 36px;
  height: 36px;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  transition: background 0.15s, transform 0.15s;
  background: rgba(100, 100, 100, 0.15);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
}
#mt-toolbar-btn:hover {
  background: rgba(100, 100, 100, 0.3);
  transform: scale(1.08);
}
#mt-toolbar-btn svg { width: 20px; height: 20px; }

#mt-panel-overlay {
  position: fixed;
  inset: 0;
  z-index: 999991;
  background: rgba(0,0,0,0.25);
  opacity: 0;
  pointer-events: none;
  transition: opacity 0.2s;
}
#mt-panel-overlay.mt-open {
  opacity: 1;
  pointer-events: auto;
}

#mt-panel {
  position: fixed;
  top: 0;
  left: 0;
  bottom: 0;
  width: 380px;
  max-width: 95vw;
  z-index: 999992;
  transform: translateX(-100%);
  transition: transform 0.25s cubic-bezier(0.4,0,0.2,1);
  display: flex;
  flex-direction: column;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  font-size: 14px;
  line-height: 1.5;
}
#mt-panel.mt-open { transform: translateX(0); }

#mt-panel,
#mt-panel * { box-sizing: border-box; }

.mt-panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid;
  flex-shrink: 0;
}
.mt-panel-header h3 {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
}
.mt-close-btn {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 20px;
  line-height: 1;
  padding: 4px;
  border-radius: 4px;
  transition: background 0.15s;
}

.mt-tabs {
  display: flex;
  border-bottom: 1px solid;
  flex-shrink: 0;
  overflow-x: auto;
}
.mt-tab {
  flex: 1;
  padding: 8px 4px;
  text-align: center;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  border: none;
  background: none;
  border-bottom: 2px solid transparent;
  transition: border-color 0.15s, color 0.15s;
  white-space: nowrap;
}
.mt-tab:hover { opacity: 0.8; }

.mt-tab-content {
  display: none;
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}
.mt-tab-content.mt-active { display: block; }

.mt-field {
  margin-bottom: 8px;
}
.mt-field label {
  display: block;
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 2px;
}
#mt-panel .mt-field input,
#mt-panel .mt-field select,
#mt-panel .mt-field textarea,
#mt-panel input,
#mt-panel select,
#mt-panel textarea {
  width: 100%;
  padding: 4px 8px !important;
  border: 1px solid !important;
  border-radius: 4px !important;
  font-size: 12px !important;
  font-family: inherit;
  outline: none;
  transition: border-color 0.15s;
  margin-bottom: 0 !important;
  height: auto !important;
  line-height: 1.4 !important;
}
#mt-panel .mt-field input:focus,
#mt-panel .mt-field select:focus,
#mt-panel .mt-field textarea:focus {
  border-color: #0172ad;
  box-shadow: 0 0 0 2px rgba(1,114,173,0.15);
}
#mt-panel .mt-field textarea { resize: vertical; min-height: 40px; }

.mt-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 5px 12px;
  border: 1px solid;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.15s, transform 0.1s;
  font-family: inherit;
  text-decoration: none;
}
.mt-btn:hover { transform: translateY(-1px); }
.mt-btn:active { transform: translateY(0); }
.mt-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
  transform: none;
}
.mt-btn-primary { border-color: #0172ad; }
.mt-btn-danger { border-color: #c0392b; }
.mt-btn-block { width: 100%; justify-content: center; margin-bottom: 6px; }

.mt-meta-grid {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 4px 12px;
  font-size: 13px;
}
.mt-meta-key {
  font-weight: 600;
  white-space: nowrap;
}
.mt-meta-val {
  word-break: break-word;
}

.mt-section-title {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin: 12px 0 6px;
  padding-bottom: 3px;
  border-bottom: 1px solid;
}
.mt-section-title:first-child { margin-top: 0; }

.mt-stats {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: 8px;
  margin-bottom: 8px;
}
a.mt-stat-card {
  display: block;
  transition: transform 0.1s, box-shadow 0.15s;
}
a.mt-stat-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
}
.mt-stat-card {
  padding: 12px;
  border-radius: 8px;
  text-align: center;
  border: 1px solid;
}
.mt-stat-num {
  font-size: 24px;
  font-weight: 700;
  line-height: 1;
}
.mt-stat-label {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-top: 4px;
}

.mt-toast {
  position: fixed;
  bottom: 20px;
  right: 20px;
  z-index: 999999;
  padding: 10px 18px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  opacity: 0;
  transform: translateY(10px);
  transition: opacity 0.2s, transform 0.2s;
  pointer-events: none;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
}
.mt-toast.mt-show {
  opacity: 1;
  transform: translateY(0);
}

.mt-confirm-overlay {
  position: fixed;
  inset: 0;
  z-index: 999995;
  background: rgba(0,0,0,0.4);
  display: flex;
  align-items: center;
  justify-content: center;
}
.mt-confirm-box {
  padding: 24px;
  border-radius: 12px;
  max-width: 400px;
  width: 90%;
  text-align: center;
}
.mt-confirm-box h4 { margin: 0 0 8px; font-size: 16px; }
.mt-confirm-box p { margin: 0 0 20px; font-size: 14px; }
.mt-confirm-actions { display: flex; gap: 8px; justify-content: center; }

.mt-autocomplete-wrap { position: relative; }
.mt-autocomplete-list {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  z-index: 10;
  border: 1px solid;
  border-radius: 6px;
  max-height: 150px;
  overflow-y: auto;
  display: none;
}
.mt-autocomplete-list.mt-show { display: block; }
.mt-autocomplete-item {
  padding: 6px 10px;
  cursor: pointer;
  font-size: 13px;
  transition: background 0.1s;
}

.mt-tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-top: 4px;
}
.mt-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 12px;
  border: 1px solid;
}
.mt-tag-remove {
  cursor: pointer;
  font-size: 14px;
  line-height: 1;
  opacity: 0.6;
}
.mt-tag-remove:hover { opacity: 1; }

.mt-404-create {
  margin-top: 20px;
  text-align: center;
}
.mt-404-create-btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 12px 24px;
  border: 2px solid #0172ad;
  border-radius: 8px;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.15s, transform 0.1s;
  font-family: inherit;
  text-decoration: none;
}
.mt-404-create-btn:hover { transform: translateY(-1px); }

/* Light theme defaults */
#mt-panel {
  background: #fff;
  color: #1a1a1a;
  box-shadow: 4px 0 24px rgba(0,0,0,0.12);
}
.mt-panel-header { border-color: #e5e5e5; }
.mt-close-btn { color: #666; }
.mt-close-btn:hover { background: #f0f0f0; }
.mt-tabs { border-color: #e5e5e5; }
.mt-tab { color: #666; }
.mt-tab.mt-active { color: #0172ad; border-bottom-color: #0172ad; }
.mt-field input,
.mt-field select,
.mt-field textarea { background: #fff; border-color: #ddd; color: #1a1a1a; }
.mt-btn { background: #fff; color: #1a1a1a; border-color: #ddd; }
.mt-btn-primary { background: #0172ad; color: #fff; }
.mt-btn-danger { background: #fff; color: #c0392b; }
.mt-stat-card { background: #f8f9fa; border-color: #e5e5e5; }
.mt-toast { background: #1a1a1a; color: #fff; }
.mt-toast.mt-error { background: #c0392b; }
.mt-confirm-box { background: #fff; }
.mt-autocomplete-list { background: #fff; border-color: #ddd; }
.mt-autocomplete-item:hover { background: #f0f0f0; }
.mt-tag { background: #e9f2fc; border-color: #9bccfd; color: #0172ad; }
.mt-section-title { border-color: #e5e5e5; color: #888; }
.mt-meta-key { color: #555; }
.mt-404-create-btn { background: #fff; color: #0172ad; }
.mt-404-create-btn:hover { background: #e9f2fc; }
#mt-toolbar-btn svg { fill: #333; stroke: #333; }

/* Dark theme */
@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) #mt-panel { background: #1e1e2e; color: #cdd6f4; box-shadow: 4px 0 24px rgba(0,0,0,0.4); }
  :root:not([data-theme="light"]) .mt-panel-header { border-color: #313244; }
  :root:not([data-theme="light"]) .mt-close-btn { color: #a6adc8; }
  :root:not([data-theme="light"]) .mt-close-btn:hover { background: #313244; }
  :root:not([data-theme="light"]) .mt-tabs { border-color: #313244; }
  :root:not([data-theme="light"]) .mt-tab { color: #a6adc8; }
  :root:not([data-theme="light"]) .mt-tab.mt-active { color: #89b4fa; border-bottom-color: #89b4fa; }
  :root:not([data-theme="light"]) .mt-field input,
  :root:not([data-theme="light"]) .mt-field select,
  :root:not([data-theme="light"]) .mt-field textarea { background: #181825; border-color: #45475a; color: #cdd6f4; }
  :root:not([data-theme="light"]) .mt-btn { background: #313244; color: #cdd6f4; border-color: #45475a; }
  :root:not([data-theme="light"]) .mt-btn-primary { background: #0172ad; color: #fff; border-color: #0172ad; }
  :root:not([data-theme="light"]) .mt-btn-danger { background: #313244; color: #f38ba8; border-color: #f38ba8; }
  :root:not([data-theme="light"]) .mt-stat-card { background: #181825; border-color: #313244; }
  :root:not([data-theme="light"]) .mt-toast { background: #cdd6f4; color: #1e1e2e; }
  :root:not([data-theme="light"]) .mt-toast.mt-error { background: #f38ba8; color: #1e1e2e; }
  :root:not([data-theme="light"]) .mt-confirm-box { background: #1e1e2e; color: #cdd6f4; }
  :root:not([data-theme="light"]) .mt-autocomplete-list { background: #1e1e2e; border-color: #45475a; }
  :root:not([data-theme="light"]) .mt-autocomplete-item:hover { background: #313244; }
  :root:not([data-theme="light"]) .mt-tag { background: #1e3a5f; border-color: #2a5a8f; color: #89b4fa; }
  :root:not([data-theme="light"]) .mt-section-title { border-color: #313244; color: #6c7086; }
  :root:not([data-theme="light"]) .mt-meta-key { color: #a6adc8; }
  :root:not([data-theme="light"]) .mt-404-create-btn { background: #1e1e2e; color: #89b4fa; border-color: #89b4fa; }
  :root:not([data-theme="light"]) .mt-404-create-btn:hover { background: #1e3a5f; }
  :root:not([data-theme="light"]) #mt-toolbar-btn svg { fill: #cdd6f4; stroke: #cdd6f4; }
}
[data-theme="dark"] #mt-panel { background: #1e1e2e; color: #cdd6f4; box-shadow: 4px 0 24px rgba(0,0,0,0.4); }
[data-theme="dark"] .mt-panel-header { border-color: #313244; }
[data-theme="dark"] .mt-close-btn { color: #a6adc8; }
[data-theme="dark"] .mt-close-btn:hover { background: #313244; }
[data-theme="dark"] .mt-tabs { border-color: #313244; }
[data-theme="dark"] .mt-tab { color: #a6adc8; }
[data-theme="dark"] .mt-tab.mt-active { color: #89b4fa; border-bottom-color: #89b4fa; }
[data-theme="dark"] .mt-field input,
[data-theme="dark"] .mt-field select,
[data-theme="dark"] .mt-field textarea { background: #181825; border-color: #45475a; color: #cdd6f4; }
[data-theme="dark"] .mt-btn { background: #313244; color: #cdd6f4; border-color: #45475a; }
[data-theme="dark"] .mt-btn-primary { background: #0172ad; color: #fff; border-color: #0172ad; }
[data-theme="dark"] .mt-btn-danger { background: #313244; color: #f38ba8; border-color: #f38ba8; }
[data-theme="dark"] .mt-stat-card { background: #181825; border-color: #313244; }
[data-theme="dark"] .mt-toast { background: #cdd6f4; color: #1e1e2e; }
[data-theme="dark"] .mt-toast.mt-error { background: #f38ba8; color: #1e1e2e; }
[data-theme="dark"] .mt-confirm-box { background: #1e1e2e; color: #cdd6f4; }
[data-theme="dark"] .mt-autocomplete-list { background: #1e1e2e; border-color: #45475a; }
[data-theme="dark"] .mt-autocomplete-item:hover { background: #313244; }
[data-theme="dark"] .mt-tag { background: #1e3a5f; border-color: #2a5a8f; color: #89b4fa; }
[data-theme="dark"] .mt-section-title { border-color: #313244; color: #6c7086; }
[data-theme="dark"] .mt-meta-key { color: #a6adc8; }
[data-theme="dark"] .mt-404-create-btn { background: #1e1e2e; color: #89b4fa; border-color: #89b4fa; }
[data-theme="dark"] .mt-404-create-btn:hover { background: #1e3a5f; }
[data-theme="dark"] #mt-toolbar-btn svg { fill: #cdd6f4; stroke: #cdd6f4; }
"#;

const TOOLBAR_JS: &str = r#"(() => {
  if (document.getElementById('mt-toolbar-btn')) return;

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
        <button class="mt-close-btn" id="mt-panel-close">&times;</button>
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
        <label>Title</label>
        <input type="text" id="mt-edit-title" value="${(fm.title || '').replace(/"/g, '&quot;')}">
      </div>
      <div class="mt-field">
        <label>Slug</label>
        <input type="text" id="mt-edit-slug" value="${(fm.slug || '').replace(/"/g, '&quot;')}">
      </div>
      <div class="mt-field">
        <label>Description</label>
        <textarea id="mt-edit-desc">${fm.description || ''}</textarea>
      </div>
      <div class="mt-field">
        <label>Date</label>
        <input type="datetime-local" id="mt-edit-date" value="${fm.date ? fm.date.replace(' ', 'T').slice(0, 19) : ''}"
          ${!hasDate ? 'placeholder="No date (page)"' : ''}>
      </div>
      <div class="mt-field">
        <label>Tags (comma-separated)</label>
        <input type="text" id="mt-edit-tags" value="${(fm.tags || []).join(', ')}">
      </div>
      <div class="mt-field">
        <label>Stream</label>
        <input type="text" id="mt-edit-stream" value="${fm.stream || ''}">
      </div>
      <div class="mt-field">
        <label>Series</label>
        <input type="text" id="mt-edit-series" value="${fm.series || ''}">
      </div>
      <div class="mt-field">
        <label>Authors (comma-separated)</label>
        <input type="text" id="mt-edit-authors" value="${(fm.authors || []).join(', ')}">
      </div>
      <div class="mt-field">
        <label>Language</label>
        <input type="text" id="mt-edit-lang" value="${fm.language || ''}">
      </div>
      <div class="mt-field">
        <label>Translates (slug)</label>
        <input type="text" id="mt-edit-translates" value="${fm.translates || ''}">
      </div>
      <div class="mt-field">
        <label>Banner Image</label>
        <input type="text" id="mt-edit-banner" value="${(fm.banner_image || '').replace(/"/g, '&quot;')}" placeholder="media/banner.jpg">
      </div>
      <div class="mt-field">
        <label>Card Image</label>
        <input type="text" id="mt-edit-card" value="${(fm.card_image || '').replace(/"/g, '&quot;')}" placeholder="media/card.jpg">
      </div>
      <div class="mt-field">
        <label>Pinned</label>
        <select id="mt-edit-pinned">
          <option value=""${!fm.pinned ? ' selected' : ''}>No</option>
          <option value="true"${fm.pinned ? ' selected' : ''}>Yes</option>
        </select>
      </div>
      <div class="mt-field">
        <label>Comments</label>
        <select id="mt-edit-comments">
          <option value=""${fm.comments === null || fm.comments === undefined ? ' selected' : ''}>Default</option>
          <option value="true"${fm.comments === true ? ' selected' : ''}>Enabled</option>
          <option value="false"${fm.comments === false ? ' selected' : ''}>Disabled</option>
        </select>
      </div>
      <div class="mt-field">
        <label>Extra (JSON)</label>
        <textarea id="mt-edit-extra">${fm.extra ? JSON.stringify(fm.extra, null, 2) : ''}</textarea>
      </div>
      <button class="mt-btn mt-btn-primary mt-btn-block" id="mt-edit-save">Save Frontmatter</button>
      <button class="mt-btn mt-btn-block" disabled title="Coming soon - content editing will be added in a future release">Edit Content</button>
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
      if (newSlug !== (fm.slug || '')) updates.slug = newSlug || null;
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

      if (Object.keys(updates).length === 0) {
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
        <label>Language code</label>
        <input type="text" id="mt-action-trans-lang" placeholder="e.g. pt, es, fr">
      </div>
      <div class="mt-field">
        <label>Title</label>
        <input type="text" id="mt-action-trans-title" placeholder="Translated title">
      </div>
      <button class="mt-btn mt-btn-primary mt-btn-block" id="mt-action-translate">Add Translation</button>

      <div class="mt-section-title">Clone / Copy</div>
      <div class="mt-field">
        <label>New title</label>
        <input type="text" id="mt-action-clone-title" placeholder="Title for the copy">
      </div>
      <button class="mt-btn mt-btn-block" id="mt-action-clone">Clone Content</button>

      <div class="mt-section-title">Move / Rename</div>
      <div class="mt-field">
        <label>New filename</label>
        <input type="text" id="mt-action-move-name" value="${metadata && metadata.source_path ? metadata.source_path.split('/').pop() : ''}" placeholder="new-name.md">
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
        <label>Title</label>
        <input type="text" id="mt-site-new-title" placeholder="New content title">
      </div>
      <div class="mt-field">
        <label>Tags (comma-separated)</label>
        <input type="text" id="mt-site-new-tags" placeholder="tag1, tag2">
      </div>
      <details class="mt-advanced">
        <summary style="cursor:pointer;font-size:12px;font-weight:600;margin-bottom:8px;user-select:none">+ Advanced</summary>
        <div class="mt-field">
          <label>Stream</label>
          <input type="text" id="mt-site-new-stream" placeholder="e.g. tutorial, news">
        </div>
        <div class="mt-field">
          <label>Series</label>
          <input type="text" id="mt-site-new-series" placeholder="e.g. python-tutorial">
        </div>
        <div class="mt-field">
          <label>Language</label>
          <input type="text" id="mt-site-new-lang" placeholder="e.g. en, pt, es">
        </div>
        <div class="mt-field">
          <label>Translates (slug of original post)</label>
          <input type="text" id="mt-site-new-translates" placeholder="original-post-slug">
        </div>
        <div class="mt-field">
          <label>Directory</label>
          <input type="text" id="mt-site-new-directory" placeholder="e.g. tutorials/rust">
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
        <input type="text" class="mt-menu-name" value="${esc(item[0])}" placeholder="Label" style="flex:1">
        <input type="text" class="mt-menu-url" value="${esc(item[1])}" placeholder="URL" style="flex:1">
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
        <input type="text" class="mt-dn-field" data-section="${title}" data-key="${key}" data-field="display_name" value="${esc(dn)}" placeholder="Display name" style="flex:1">
        ${title === 'series' ? `<input type="text" class="mt-dn-field" data-section="${title}" data-key="${key}" data-field="description" value="${esc(desc)}" placeholder="Description" style="flex:1">` : ''}
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
          <input type="text" class="mt-author-field" data-author="${key}" data-field="name" value="${esc(v.name)}" placeholder="Display name">
          <input type="text" class="mt-author-field" data-author="${key}" data-field="avatar" value="${esc(v.avatar)}" placeholder="Avatar URL">
          <input type="text" class="mt-author-field" data-author="${key}" data-field="bio" value="${esc(v.bio)}" placeholder="Bio">
          <input type="text" class="mt-author-field" data-author="${key}" data-field="links" value="${esc(linksStr)}" placeholder="Label=URL, Label=URL">
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
        <label>Pages Title</label>
        <input type="text" class="mt-layout-title" data-key="pages_title" value="${esc(cfg.pages_title)}">
      </div>
      <div class="mt-field">
        <label>Tags Title</label>
        <input type="text" class="mt-layout-title" data-key="tags_title" value="${esc(cfg.tags_title)}">
      </div>
      <div class="mt-field">
        <label>Archives Title</label>
        <input type="text" class="mt-layout-title" data-key="archives_title" value="${esc(cfg.archives_title)}">
      </div>
      <div class="mt-field">
        <label>Authors Title</label>
        <input type="text" class="mt-layout-title" data-key="authors_title" value="${esc(cfg.authors_title)}">
      </div>
      <div class="mt-field">
        <label>Streams Title</label>
        <input type="text" class="mt-layout-title" data-key="streams_title" value="${esc(cfg.streams_title)}">
      </div>
      <div class="mt-field">
        <label>Series Title</label>
        <input type="text" class="mt-layout-title" data-key="series_title" value="${esc(cfg.series_title)}">
      </div>
      <div class="mt-field">
        <label>Languages Title</label>
        <input type="text" class="mt-layout-title" data-key="languages_title" value="${esc(cfg.languages_title)}">
      </div>
      <div class="mt-field">
        <label>Search Title</label>
        <input type="text" class="mt-layout-title" data-key="search_title" value="${esc(cfg.search_title)}">
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
        <input type="text" class="mt-menu-name" value="${esc(name)}" placeholder="Label" style="flex:1">
        <input type="text" class="mt-menu-url" value="${esc(url)}" placeholder="URL" style="flex:1">
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
    const boolSel = (val) => {
      const v = val === true || val === 'true';
      return `<select class="mt-cfg-field">
        <option value="true"${v ? ' selected' : ''}>Yes</option>
        <option value="false"${!v ? ' selected' : ''}>No</option>
      </select>`;
    };

    container.innerHTML = `
      <div class="mt-section-title">General</div>
      <div class="mt-field">
        <label>Site Name</label>
        <input type="text" class="mt-cfg-field" data-key="name" value="${esc(cfg.name)}">
      </div>
      <div class="mt-field">
        <label>Tagline</label>
        <input type="text" class="mt-cfg-field" data-key="tagline" value="${esc(cfg.tagline)}">
      </div>
      <div class="mt-field">
        <label>URL</label>
        <input type="text" class="mt-cfg-field" data-key="url" value="${esc(cfg.url)}" placeholder="https://example.com">
      </div>
      <div class="mt-field">
        <label>Language</label>
        <input type="text" class="mt-cfg-field" data-key="language" value="${esc(cfg.language)}">
      </div>
      <div class="mt-field">
        <label>Footer</label>
        <textarea class="mt-cfg-field" data-key="footer">${cfg.footer || ''}</textarea>
      </div>
      <div class="mt-field">
        <label>Default Author</label>
        <input type="text" class="mt-cfg-field" data-key="default_author" value="${esc(cfg.default_author)}">
      </div>
      <div class="mt-field">
        <label>Default Date Format</label>
        <input type="text" class="mt-cfg-field" data-key="default_date_format" value="${esc(cfg.default_date_format)}" placeholder="%B %d, %Y">
      </div>

      <div class="mt-section-title">Display</div>
      <div class="mt-field">
        <label>Pagination</label>
        <input type="number" class="mt-cfg-field" data-key="pagination" data-type="number" value="${cfg.pagination || 10}" min="1">
      </div>
      <div class="mt-field">
        <label>Enable Search</label>
        <select class="mt-cfg-field" data-key="enable_search">
          <option value="true"${cfg.enable_search ? ' selected' : ''}>Yes</option>
          <option value="false"${!cfg.enable_search ? ' selected' : ''}>No</option>
        </select>
      </div>
      <div class="mt-field">
        <label>Show Related Content</label>
        <select class="mt-cfg-field" data-key="enable_related_content">
          <option value="true"${cfg.enable_related_content ? ' selected' : ''}>Yes</option>
          <option value="false"${!cfg.enable_related_content ? ' selected' : ''}>No</option>
        </select>
      </div>
      <div class="mt-field">
        <label>Show Next/Prev Links</label>
        <select class="mt-cfg-field" data-key="show_next_prev_links">
          <option value="true"${cfg.show_next_prev_links ? ' selected' : ''}>Yes</option>
          <option value="false"${!cfg.show_next_prev_links ? ' selected' : ''}>No</option>
        </select>
      </div>

      <div class="mt-section-title">Images</div>
      <div class="mt-field">
        <label>Card Image</label>
        <input type="text" class="mt-cfg-field" data-key="card_image" value="${esc(cfg.card_image)}">
      </div>
      <div class="mt-field">
        <label>Banner Image</label>
        <input type="text" class="mt-cfg-field" data-key="banner_image" value="${esc(cfg.banner_image)}">
      </div>
      <div class="mt-field">
        <label>Logo Image</label>
        <input type="text" class="mt-cfg-field" data-key="logo_image" value="${esc(cfg.logo_image)}">
      </div>

      <div class="mt-section-title">Paths</div>
      <div class="mt-field">
        <label>Content Path</label>
        <input type="text" class="mt-cfg-field" data-key="content_path" value="${esc(cfg.content_path)}" placeholder="content">
      </div>
      <div class="mt-field">
        <label>Site Path</label>
        <input type="text" class="mt-cfg-field" data-key="site_path" value="${esc(cfg.site_path)}" placeholder="site">
      </div>
      <div class="mt-field">
        <label>Media Path</label>
        <input type="text" class="mt-cfg-field" data-key="media_path" value="${esc(cfg.media_path)}" placeholder="media">
      </div>

      <div class="mt-section-title">Extra (JSON)</div>
      <div class="mt-field">
        <textarea class="mt-cfg-field" data-key="extra" id="mt-cfg-extra" style="min-height:100px">${cfg.extra ? JSON.stringify(cfg.extra, null, 2) : '{}'}</textarea>
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
})();"#;

pub fn start(bind_address: &str, ctx: &ServerContext, live_reload: Option<&LiveReload>) {
    let server = match Server::http(bind_address) {
        Ok(server) => server,
        Err(e) => {
            warn!(
                "Failed to start server on address {bind_address}: {e:?}. Falling back to OS-assigned port."
            );
            match Server::http(FALLBACK_BIND_ADDRESS) {
                Ok(server) => server,
                Err(e) => {
                    error!("Failed to start server on fallback address: {e:?}");
                    return;
                }
            }
        }
    };

    let Some(server_addr) = server.server_addr().to_ip() else {
        warn!("Failed to get server IP address, using fallback display");
        // Use a fallback approach for display purposes
        let raw_addr = server.server_addr();
        let server_bind_address = format!("{raw_addr}");
        info!("Server started at http://{server_bind_address}/ - Type ^C to stop.");
        if live_reload.is_some() {
            info!("Live reload WebSocket available at ws://{server_bind_address}{LIVE_RELOAD_WS_PATH}");
        }
        // Continue with request handling
        for mut request in server.incoming_requests() {
            if let Some(live_reload_handler) = live_reload {
                if is_live_reload_ws_request(&request) {
                    live_reload_handler.accept(request);
                    continue;
                }
            }

            let response = match handle_request(&mut request, ctx, live_reload.is_some()) {
                Ok(response) => response,
                Err(err) => {
                    error!("Error handling request: {err:?}");
                    Response::from_string("Internal Server Error").with_status_code(500)
                }
            };

            if let Err(err) = request.respond(response) {
                error!("Error sending response: {err:?}");
            }
        }
        return;
    };
    let server_port = server_addr.port();
    let server_bind_address = format!("{}:{}", server_addr.ip(), server_port);

    if live_reload.is_some() {
        info!("Live reload WebSocket available at ws://{server_bind_address}{LIVE_RELOAD_WS_PATH}");
    }

    info!("Server started at http://{server_bind_address}/ - Type ^C to stop.");

    for mut request in server.incoming_requests() {
        if let Some(live_reload_handler) = live_reload {
            if is_live_reload_ws_request(&request) {
                live_reload_handler.accept(request);
                continue;
            }
        }

        let response = match handle_request(&mut request, ctx, live_reload.is_some()) {
            Ok(response) => response,
            Err(err) => {
                error!("Error handling request: {err:?}");
                Response::from_string("Internal Server Error").with_status_code(500)
            }
        };

        if let Err(err) = request.respond(response) {
            error!("Failed to send response: {err:?}");
        }
    }
}

#[allow(
    clippy::case_sensitive_file_extension_comparisons,
    clippy::too_many_lines
)]
fn handle_request(
    request: &mut tiny_http::Request,
    ctx: &ServerContext,
    live_reload_enabled: bool,
) -> Result<Response<Cursor<Vec<u8>>>, String> {
    let output_folder = ctx.output_folder.as_path();
    let decoded_url = match decode(request.url()) {
        Ok(decoded) => decoded.into_owned(),
        Err(err) => {
            error!("Error decoding url {}: {err:?}", request.url());
            return Err(format!("Error decoding url: {err}"));
        }
    };

    if decoded_url.starts_with(CONTENT_API_PATH) {
        return Ok(handle_content_api(request, &decoded_url, ctx));
    }

    if decoded_url == CONFIG_API_PATH {
        return Ok(handle_config_api(request, ctx));
    }

    if decoded_url == DATA_API_PATH {
        return Ok(handle_data_api(ctx));
    }

    if live_reload_enabled && decoded_url == format!("/{LIVE_RELOAD_SCRIPT_PATH}") {
        let mut response = Response::from_string(LIVE_RELOAD_SCRIPT);
        let js_header = Header::from_bytes("Content-Type", "application/javascript")
            .map_err(|()| "invalid live reload header".to_string())?;
        response.add_header(js_header);
        if let Ok(cache_header) = Header::from_bytes("Cache-Control", "no-store") {
            response.add_header(cache_header);
        }
        return Ok(response);
    }

    if decoded_url == format!("/{TOOLBAR_JS_PATH}") {
        let mut response = Response::from_string(TOOLBAR_JS);
        if let Ok(h) = Header::from_bytes("Content-Type", "application/javascript; charset=utf-8") {
            response.add_header(h);
        }
        if let Ok(h) = Header::from_bytes("Cache-Control", "no-store") {
            response.add_header(h);
        }
        return Ok(response);
    }

    if decoded_url == format!("/{TOOLBAR_CSS_PATH}") {
        let mut response = Response::from_string(TOOLBAR_CSS);
        if let Ok(h) = Header::from_bytes("Content-Type", "text/css; charset=utf-8") {
            response.add_header(h);
        }
        if let Ok(h) = Header::from_bytes("Cache-Control", "no-store") {
            response.add_header(h);
        }
        return Ok(response);
    }

    let request_path = match decoded_url.as_str() {
        "/" => "index.html".to_string(),
        url if url.ends_with('/') => format!("{}index.html", &url[1..]),
        url => url[1..].to_string(),
    };

    let file_path = output_folder.join(&request_path);
    let error_path = output_folder.join("404.html");

    if file_path.is_file() {
        match File::open(&file_path) {
            Ok(mut file) => {
                let mut buffer = Vec::new();
                std::io::copy(&mut file, &mut buffer).map_err(|e| e.to_string())?;
                if request_path.ends_with(".html") {
                    let original_buffer = buffer.clone();
                    if let Ok(mut html) = String::from_utf8(buffer) {
                        let mut snippet = String::new();
                        if live_reload_enabled && !html.contains(LIVE_RELOAD_SCRIPT_PATH) {
                            let _ = write!(
                                snippet,
                                "\n<script src=\"/{LIVE_RELOAD_SCRIPT_PATH}\"></script>\n"
                            );
                        }
                        if !html.contains(TOOLBAR_CSS_PATH) {
                            let _ = write!(
                                snippet,
                                "<link rel=\"stylesheet\" href=\"/{TOOLBAR_CSS_PATH}\">\n\
                                 <script src=\"/{TOOLBAR_JS_PATH}\"></script>\n"
                            );
                        }
                        if !snippet.is_empty() {
                            if let Some(pos) = html.rfind("</body>") {
                                html.insert_str(pos, &snippet);
                            } else {
                                html.push_str(&snippet);
                            }
                        }
                        buffer = html.into_bytes();
                    } else {
                        buffer = original_buffer;
                    }
                }
                info!(
                    "\"{} {} HTTP/{}\" 200 -",
                    request.method(),
                    request_path,
                    request.http_version()
                );
                let mut resp = Response::from_data(buffer);
                if let Some(content_type) = content_type_for(&request_path) {
                    match Header::from_bytes("Content-Type", content_type) {
                        Ok(header) => resp.add_header(header),
                        Err(e) => error!("Failed to create Content-Type header: {e:?}"),
                    }
                }
                Ok(resp)
            }
            Err(err) => {
                error!("Failed to read file {}: {err:?}", file_path.display());
                Err(format!("Error reading file: {err}"))
            }
        }
    } else {
        error!(
            "\"{} {} HTTP/{}\" 404 -",
            request.method(),
            request_path,
            request.http_version()
        );
        render_not_found(&error_path, &request_path, live_reload_enabled)
    }
}

fn json_response(status: u16, body: &serde_json::Value) -> Response<Cursor<Vec<u8>>> {
    let json_bytes = serde_json::to_string_pretty(body)
        .unwrap_or_else(|_| r#"{"error":"serialization failed"}"#.to_string())
        .into_bytes();
    let mut resp = Response::from_data(json_bytes).with_status_code(status);
    if let Ok(header) = Header::from_bytes("Content-Type", "application/json; charset=utf-8") {
        resp.add_header(header);
    }
    resp
}

fn read_request_body(request: &mut Request) -> Result<String, String> {
    let mut body = String::new();
    request
        .as_reader()
        .read_to_string(&mut body)
        .map_err(|e| format!("Failed to read request body: {e}"))?;
    Ok(body)
}

fn handle_content_api(
    request: &mut Request,
    url: &str,
    ctx: &ServerContext,
) -> Response<Cursor<Vec<u8>>> {
    let rest = url
        .strip_prefix(CONTENT_API_PATH)
        .and_then(|s| s.strip_prefix('/'))
        .unwrap_or("");

    match *request.method() {
        Method::Post if rest.is_empty() => handle_create_content(request, ctx),
        Method::Post if rest.ends_with("/move") => {
            let slug = rest.strip_suffix("/move").unwrap_or("");
            if slug.is_empty() {
                return json_response(400, &json!({"error": "slug is required in URL path"}));
            }
            handle_move_content(request, slug, ctx)
        }
        Method::Post if rest.ends_with("/clone") => {
            let slug = rest.strip_suffix("/clone").unwrap_or("");
            if slug.is_empty() {
                return json_response(400, &json!({"error": "slug is required in URL path"}));
            }
            handle_clone_content(request, slug, ctx)
        }
        Method::Patch => {
            if rest.is_empty() {
                return json_response(400, &json!({"error": "slug is required in URL path"}));
            }
            handle_patch_content(request, rest, ctx)
        }
        Method::Delete => {
            if rest.is_empty() {
                return json_response(400, &json!({"error": "slug is required in URL path"}));
            }
            handle_delete_content(rest, ctx)
        }
        _ => json_response(405, &json!({"error": "method not allowed"})),
    }
}

fn handle_create_content(request: &mut Request, ctx: &ServerContext) -> Response<Cursor<Vec<u8>>> {
    let body = match read_request_body(request) {
        Ok(b) => b,
        Err(e) => return json_response(400, &json!({"error": e})),
    };

    let parsed: serde_json::Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(e) => return json_response(400, &json!({"error": format!("Invalid JSON: {e}")})),
    };

    let title = match parsed.get("title").and_then(|t| t.as_str()) {
        Some(t) => t.to_string(),
        None => return json_response(400, &json!({"error": "title is required"})),
    };

    let tags = parsed.get("tags").and_then(|v| {
        if let Some(s) = v.as_str() {
            Some(s.to_string())
        } else if let Some(arr) = v.as_array() {
            let items: Vec<String> = arr
                .iter()
                .filter_map(|i| i.as_str().map(String::from))
                .collect();
            if items.is_empty() {
                None
            } else {
                Some(items.join(", "))
            }
        } else {
            None
        }
    });
    let directory = parsed
        .get("directory")
        .and_then(|v| v.as_str())
        .map(String::from);
    let page = parsed
        .get("page")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let lang = parsed
        .get("lang")
        .and_then(|v| v.as_str())
        .map(String::from);
    let translates = parsed
        .get("translates")
        .and_then(|v| v.as_str())
        .map(String::from);

    let params = crate::content::CreateContentParams {
        title,
        tags,
        directory,
        page,
        lang,
        translates,
    };

    match crate::content::create_content(&ctx.input_folder, &ctx.config_path, &params) {
        Ok(result) => {
            let mut output = serde_json::Map::new();
            output.insert("file".into(), json!(result.file_path.display().to_string()));
            output.insert("title".into(), json!(result.title));
            output.insert("slug".into(), json!(result.slug));
            output.insert("is_page".into(), json!(result.is_page));
            if let Some(ref date) = result.date {
                output.insert("date".into(), json!(date));
            }
            if let Some(ref tags) = result.tags {
                output.insert("tags".into(), json!(tags));
            }
            if let Some(ref lang) = result.lang {
                output.insert("language".into(), json!(lang));
            }
            if let Some(ref translates) = result.translates {
                output.insert("translates".into(), json!(translates));
            }
            json_response(201, &serde_json::Value::Object(output))
        }
        Err(e) => json_response(400, &json!({"error": e})),
    }
}

fn handle_patch_content(
    request: &mut Request,
    slug: &str,
    ctx: &ServerContext,
) -> Response<Cursor<Vec<u8>>> {
    let body = match read_request_body(request) {
        Ok(b) => b,
        Err(e) => return json_response(400, &json!({"error": e})),
    };

    let patch_fields: serde_json::Map<String, serde_json::Value> = match serde_json::from_str(&body)
    {
        Ok(v) => v,
        Err(e) => return json_response(400, &json!({"error": format!("Invalid JSON: {e}")})),
    };

    if patch_fields.is_empty() {
        return json_response(400, &json!({"error": "no fields to update"}));
    }

    let site_data = crate::site::Data::from_file(&ctx.config_path);
    let content_folder = crate::site::get_content_folder(&site_data.site, &ctx.input_folder);

    let Some(file_path) = crate::content::find_file_by_slug(&content_folder, slug) else {
        return json_response(
            404,
            &json!({"error": format!("Content with slug '{slug}' not found")}),
        );
    };

    match crate::content::update_frontmatter(&file_path, &patch_fields) {
        Ok(frontmatter) => json_response(
            200,
            &json!({
                "slug": slug,
                "file": file_path.display().to_string(),
                "frontmatter": frontmatter,
            }),
        ),
        Err(e) => json_response(500, &json!({"error": e})),
    }
}

fn handle_delete_content(slug: &str, ctx: &ServerContext) -> Response<Cursor<Vec<u8>>> {
    let site_data = crate::site::Data::from_file(&ctx.config_path);
    let content_folder = crate::site::get_content_folder(&site_data.site, &ctx.input_folder);

    match crate::content::delete_content(&content_folder, slug) {
        Ok(file_path) => json_response(
            200,
            &json!({
                "slug": slug,
                "file": file_path.display().to_string(),
                "deleted": true,
            }),
        ),
        Err(e) if e.contains("not found") => json_response(404, &json!({"error": e})),
        Err(e) => json_response(500, &json!({"error": e})),
    }
}

fn handle_move_content(
    request: &mut Request,
    slug: &str,
    ctx: &ServerContext,
) -> Response<Cursor<Vec<u8>>> {
    let body = match read_request_body(request) {
        Ok(b) => b,
        Err(e) => return json_response(400, &json!({"error": e})),
    };

    let parsed: serde_json::Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(e) => return json_response(400, &json!({"error": format!("Invalid JSON: {e}")})),
    };

    let Some(new_filename) = parsed.get("filename").and_then(|v| v.as_str()) else {
        return json_response(400, &json!({"error": "filename is required"}));
    };

    let site_data = crate::site::Data::from_file(&ctx.config_path);
    let content_folder = crate::site::get_content_folder(&site_data.site, &ctx.input_folder);

    match crate::content::move_content(&content_folder, slug, new_filename) {
        Ok((old_path, new_path)) => {
            let mut new_slug = new_path.file_stem().and_then(|s| s.to_str()).map_or_else(
                || slug.to_string(),
                crate::content::remove_date_from_filename,
            );
            if let Ok(file_content) = std::fs::read_to_string(&new_path) {
                if let Ok((fm, _)) = crate::parser::parse_front_matter(&file_content) {
                    if let Some(s) = fm
                        .get("slug")
                        .and_then(|v| v.as_str().map(|s| s.trim_matches('"').to_string()))
                    {
                        new_slug = s;
                    }
                }
            }
            json_response(
                200,
                &json!({
                    "slug": new_slug,
                    "old_file": old_path.display().to_string(),
                    "new_file": new_path.display().to_string(),
                }),
            )
        }
        Err(e) if e.contains("not found") => json_response(404, &json!({"error": e})),
        Err(e) => json_response(400, &json!({"error": e})),
    }
}

fn handle_clone_content(
    request: &mut Request,
    slug: &str,
    ctx: &ServerContext,
) -> Response<Cursor<Vec<u8>>> {
    let body = match read_request_body(request) {
        Ok(b) => b,
        Err(e) => return json_response(400, &json!({"error": e})),
    };

    let parsed: serde_json::Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(e) => return json_response(400, &json!({"error": format!("Invalid JSON: {e}")})),
    };

    let Some(title) = parsed.get("title").and_then(|v| v.as_str()) else {
        return json_response(400, &json!({"error": "title is required"}));
    };

    let new_slug = parsed.get("slug").and_then(|v| v.as_str());

    let site_data = crate::site::Data::from_file(&ctx.config_path);
    let content_folder = crate::site::get_content_folder(&site_data.site, &ctx.input_folder);

    match crate::content::clone_content(&content_folder, slug, title, new_slug) {
        Ok((file_path, result_slug)) => json_response(
            201,
            &json!({
                "slug": result_slug,
                "file": file_path.display().to_string(),
                "source": slug,
            }),
        ),
        Err(e) if e.contains("not found") => json_response(404, &json!({"error": e})),
        Err(e) => json_response(400, &json!({"error": e})),
    }
}

#[allow(clippy::too_many_lines)]
fn handle_data_api(ctx: &ServerContext) -> Response<Cursor<Vec<u8>>> {
    let marmite_json_path = ctx.output_folder.join("marmite.json");
    let build_info: serde_json::Value = if marmite_json_path.exists() {
        match std::fs::read_to_string(&marmite_json_path) {
            Ok(s) => serde_json::from_str(&s).unwrap_or(json!({})),
            Err(_) => json!({}),
        }
    } else {
        json!({})
    };

    let mut tags: Vec<String> = Vec::new();
    let mut streams: Vec<String> = Vec::new();
    let mut series: Vec<String> = Vec::new();
    let mut authors: Vec<String> = Vec::new();
    let mut languages: Vec<String> = Vec::new();

    let all_content = build_info
        .get("posts")
        .into_iter()
        .chain(build_info.get("pages"))
        .filter_map(|v| v.as_array());

    for content_list in all_content {
        for item in content_list {
            if let Some(item_tags) = item.get("tags").and_then(|v| v.as_array()) {
                for t in item_tags {
                    if let Some(s) = t.as_str() {
                        let val = s.to_string();
                        if !tags.contains(&val) {
                            tags.push(val);
                        }
                    }
                }
            }
            if let Some(s) = item.get("stream").and_then(|v| v.as_str()) {
                let val = s.to_string();
                if !streams.contains(&val) {
                    streams.push(val);
                }
            }
            if let Some(s) = item.get("series").and_then(|v| v.as_str()) {
                let val = s.to_string();
                if !series.contains(&val) {
                    series.push(val);
                }
            }
            if let Some(item_authors) = item.get("authors").and_then(|v| v.as_array()) {
                for a in item_authors {
                    if let Some(s) = a.as_str() {
                        let val = s.to_string();
                        if !authors.contains(&val) {
                            authors.push(val);
                        }
                    }
                }
            }
        }
    }

    if let Some(config) = build_info.get("config") {
        if let Some(langs) = config.get("languages").and_then(|v| v.as_object()) {
            for key in langs.keys() {
                if !languages.contains(key) {
                    languages.push(key.clone());
                }
            }
        }
    }

    tags.sort();
    streams.sort();
    series.sort();
    authors.sort();
    languages.sort();

    let config = build_info.get("config").cloned().unwrap_or(json!({}));

    let mut slugs: Vec<String> = Vec::new();
    for key in &["posts", "pages"] {
        if let Some(items) = build_info.get(key).and_then(|v| v.as_array()) {
            for item in items {
                if let Some(s) = item.get("slug").and_then(|v| v.as_str()) {
                    slugs.push(s.to_string());
                }
            }
        }
    }
    slugs.sort();

    let iso_languages: Vec<&str> = crate::content::ISO_639_1_CODES.to_vec();

    let mut images: Vec<String> = Vec::new();
    let image_extensions = [
        "jpg", "jpeg", "png", "gif", "webp", "svg", "avif", "bmp", "tiff",
    ];
    let output_folder = ctx.output_folder.as_path();
    for entry in walkdir::WalkDir::new(output_folder)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if !image_extensions.contains(&ext.as_str()) {
            continue;
        }
        if let Ok(rel) = path.strip_prefix(output_folder) {
            images.push(rel.to_string_lossy().to_string());
        }
    }
    images.sort();

    json_response(
        200,
        &json!({
            "tags": tags,
            "streams": streams,
            "series": series,
            "authors": authors,
            "languages": languages,
            "iso_languages": iso_languages,
            "slugs": slugs,
            "images": images,
            "post_count": build_info.get("posts").and_then(|v| v.as_array()).map_or(0, Vec::len),
            "page_count": build_info.get("pages").and_then(|v| v.as_array()).map_or(0, Vec::len),
            "elapsed_time": build_info.get("elapsed_time").and_then(serde_json::Value::as_f64).unwrap_or(0.0),
            "marmite_version": build_info.get("marmite_version").and_then(|v| v.as_str()).unwrap_or(""),
            "config": config,
        }),
    )
}

fn handle_config_api(request: &mut Request, ctx: &ServerContext) -> Response<Cursor<Vec<u8>>> {
    match *request.method() {
        Method::Post => handle_create_config(ctx),
        Method::Patch => handle_patch_config(request, ctx),
        _ => json_response(405, &json!({"error": "method not allowed"})),
    }
}

fn handle_create_config(ctx: &ServerContext) -> Response<Cursor<Vec<u8>>> {
    let config_path = ctx.config_path.as_path();
    if config_path.exists() {
        return json_response(
            409,
            &json!({"error": "config file already exists", "file": config_path.display().to_string()}),
        );
    }

    let config = crate::config::Marmite::new();
    let yaml = match serde_yaml::to_string(&config) {
        Ok(s) => s,
        Err(e) => {
            return json_response(
                500,
                &json!({"error": format!("Failed to serialize config: {e}")}),
            )
        }
    };

    if let Err(e) = std::fs::write(config_path, &yaml) {
        return json_response(
            500,
            &json!({"error": format!("Failed to write config: {e}")}),
        );
    }

    json_response(
        201,
        &json!({"file": config_path.display().to_string(), "config": config}),
    )
}

fn handle_patch_config(request: &mut Request, ctx: &ServerContext) -> Response<Cursor<Vec<u8>>> {
    let body = match read_request_body(request) {
        Ok(b) => b,
        Err(e) => return json_response(400, &json!({"error": e})),
    };

    let patch: serde_yaml::Mapping = match serde_json::from_str::<serde_json::Value>(&body) {
        Ok(json_val) => match serde_yaml::to_value(&json_val) {
            Ok(serde_yaml::Value::Mapping(m)) => m,
            _ => {
                return json_response(400, &json!({"error": "request body must be a JSON object"}))
            }
        },
        Err(e) => return json_response(400, &json!({"error": format!("Invalid JSON: {e}")})),
    };

    if patch.is_empty() {
        return json_response(400, &json!({"error": "no fields to update"}));
    }

    let config_path = ctx.config_path.as_path();

    let mut existing: serde_yaml::Mapping = if config_path.exists() {
        match std::fs::read_to_string(config_path) {
            Ok(content) if !content.trim().is_empty() => {
                serde_yaml::from_str(&content).unwrap_or_default()
            }
            _ => serde_yaml::Mapping::new(),
        }
    } else {
        let config = crate::config::Marmite::new();
        match serde_yaml::to_value(&config) {
            Ok(serde_yaml::Value::Mapping(m)) => m,
            _ => serde_yaml::Mapping::new(),
        }
    };

    for (key, value) in &patch {
        if value.is_null() {
            existing.remove(key);
        } else {
            existing.insert(key.clone(), value.clone());
        }
    }

    let yaml = match serde_yaml::to_string(&existing) {
        Ok(s) => s,
        Err(e) => {
            return json_response(
                500,
                &json!({"error": format!("Failed to serialize config: {e}")}),
            )
        }
    };

    if let Err(e) = std::fs::write(config_path, &yaml) {
        return json_response(
            500,
            &json!({"error": format!("Failed to write config: {e}")}),
        );
    }

    let result: serde_json::Value =
        serde_json::to_value(&existing).unwrap_or(serde_json::Value::Null);

    json_response(
        200,
        &json!({
            "file": config_path.display().to_string(),
            "config": result,
        }),
    )
}

fn content_type_for(path: &str) -> Option<&'static str> {
    let ext = path.rsplit('.').next()?;
    Some(match ext {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "xml" => "application/xml; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "avif" => "image/avif",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "txt" => "text/plain; charset=utf-8",
        "pdf" => "application/pdf",
        "wasm" => "application/wasm",
        _ => return None,
    })
}

fn render_not_found(
    error_path: &PathBuf,
    request_path: &str,
    live_reload_enabled: bool,
) -> Result<Response<Cursor<Vec<u8>>>, String> {
    match File::open(error_path) {
        Ok(mut file) => {
            let mut buffer = Vec::new();
            std::io::copy(&mut file, &mut buffer).map_err(|e| e.to_string())?;
            if let Ok(mut html) = String::from_utf8(buffer.clone()) {
                let slug = request_path.trim_end_matches(".html");
                let live_reload_tag = if live_reload_enabled {
                    format!("<script src=\"/{LIVE_RELOAD_SCRIPT_PATH}\"></script>\n")
                } else {
                    String::new()
                };
                let inject = format!(
                    "<script>window.__marmite_404_slug__={slug};</script>\n\
                     <link rel=\"stylesheet\" href=\"/{TOOLBAR_CSS_PATH}\">\n\
                     <script src=\"/{TOOLBAR_JS_PATH}\"></script>\n\
                     {live_reload_tag}",
                    slug = serde_json::to_string(slug).unwrap_or_else(|_| "null".into()),
                );
                if let Some(pos) = html.rfind("</body>") {
                    html.insert_str(pos, &inject);
                } else {
                    html.push_str(&inject);
                }
                buffer = html.into_bytes();
            }
            let mut resp = Response::from_data(buffer).with_status_code(404);
            if let Ok(h) = Header::from_bytes("Content-Type", "text/html; charset=utf-8") {
                resp.add_header(h);
            }
            Ok(resp)
        }
        Err(err) => {
            error!("Error on rendering page 404 - {err:?}");
            Ok(Response::from_string("404 Not Found").with_status_code(404))
        }
    }
}

fn is_live_reload_ws_request(request: &Request) -> bool {
    if request.method() != &Method::Get {
        return false;
    }

    if request.url() != LIVE_RELOAD_WS_PATH {
        return false;
    }

    let mut has_upgrade = false;
    let mut has_connection_upgrade = false;
    let mut has_key = false;

    for header in request.headers() {
        if header.field.equiv("Upgrade") && header.value.as_str().eq_ignore_ascii_case("websocket")
        {
            has_upgrade = true;
        } else if header.field.equiv("Connection") {
            if header
                .value
                .as_str()
                .to_ascii_lowercase()
                .split(',')
                .any(|segment| segment.trim() == "upgrade")
            {
                has_connection_upgrade = true;
            }
        } else if header.field.equiv("Sec-WebSocket-Key") {
            has_key = true;
        }
    }

    has_upgrade && has_connection_upgrade && has_key
}

#[cfg(test)]
#[path = "tests/server.rs"]
mod tests;

#[derive(Clone)]
pub struct LiveReload {
    clients: Arc<Mutex<Vec<ClientSender>>>,
    next_id: Arc<AtomicUsize>,
}

impl LiveReload {
    pub fn new() -> Self {
        LiveReload {
            clients: Arc::new(Mutex::new(Vec::new())),
            next_id: Arc::new(AtomicUsize::new(1)),
        }
    }

    pub fn accept(&self, request: Request) {
        if let Err(err) = self.accept_internal(request) {
            error!("Live reload WebSocket upgrade failed: {err}");
        }
    }

    pub fn notify_reload(&self) {
        let payload = json!({
            "event": "reload",
            "timestamp": Utc::now().timestamp_millis(),
        })
        .to_string();
        self.broadcast(&payload);
    }

    #[allow(clippy::useless_conversion)]
    fn accept_internal(&self, request: Request) -> Result<(), String> {
        let key_value = request.headers().iter().find_map(|header| {
            if header.field.equiv("Sec-WebSocket-Key") {
                Some(header.value.as_str().trim().to_owned())
            } else {
                None
            }
        });

        let Some(key_value) = key_value else {
            Self::respond_bad_request(request, "Missing Sec-WebSocket-Key header")?;
            return Ok(());
        };

        let accept_key = derive_accept_key(key_value.as_bytes());
        let upgrade_header = Header::from_bytes("Upgrade", "websocket")
            .map_err(|()| "Failed to build Upgrade header".to_string())?;
        let connection_header = Header::from_bytes("Connection", "Upgrade")
            .map_err(|()| "Failed to build Connection header".to_string())?;
        let accept_header = Header::from_bytes("Sec-WebSocket-Accept", accept_key.as_str())
            .map_err(|()| "Failed to build Sec-WebSocket-Accept header".to_string())?;

        let response = Response::empty(101)
            .with_header(upgrade_header)
            .with_header(connection_header)
            .with_header(accept_header);

        let stream = request.upgrade("websocket", response);
        let (tx, rx) = mpsc::channel::<String>();
        let client_id = self.register(tx);
        let live_reload = self.clone();

        thread::spawn(move || {
            let mut websocket = tungstenite::WebSocket::from_raw_socket(stream, Role::Server, None);
            while let Ok(message) = rx.recv() {
                match websocket.send(Message::Text(message.into())) {
                    Ok(()) => {}
                    Err(WsError::ConnectionClosed | WsError::AlreadyClosed) => break,
                    Err(WsError::Io(err))
                        if matches!(
                            err.kind(),
                            ErrorKind::BrokenPipe
                                | ErrorKind::ConnectionReset
                                | ErrorKind::ConnectionAborted
                                | ErrorKind::NotConnected
                        ) =>
                    {
                        break;
                    }
                    Err(err) => {
                        warn!("Live reload WebSocket send error: {err:?}");
                        break;
                    }
                }
            }
            live_reload.unregister(client_id);
        });

        Ok(())
    }

    fn respond_bad_request(request: Request, message: &str) -> Result<(), String> {
        let response = Response::from_string(message).with_status_code(400);
        request
            .respond(response)
            .map_err(|err| format!("Failed to send bad request response: {err}"))
    }

    fn register(&self, sender: mpsc::Sender<String>) -> usize {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut clients) = self.clients.lock() {
            clients.push(ClientSender { id, sender });
        }
        id
    }

    fn unregister(&self, id: usize) {
        if let Ok(mut clients) = self.clients.lock() {
            clients.retain(|client| client.id != id);
        }
    }

    fn broadcast(&self, message: &str) {
        if let Ok(mut clients) = self.clients.lock() {
            clients.retain(|client| client.sender.send(message.to_string()).is_ok());
        }
    }
}

struct ClientSender {
    id: usize,
    sender: mpsc::Sender<String>,
}
