use chrono::Utc;
use log::{error, info, warn};
use serde_json::json;
use std::fmt::Write as _;
use std::io::{Cursor, ErrorKind};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, LazyLock, Mutex};
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
    pub enable_toolbar: bool,
}

const FALLBACK_BIND_ADDRESS: &str = "0.0.0.0:0";
const LIVE_RELOAD_SCRIPT_PATH: &str = "__marmite__/livereload.js";
const TOOLBAR_JS_PATH: &str = "__marmite__/toolbar.js";
const TOOLBAR_CSS_PATH: &str = "__marmite__/toolbar.css";
const LIVE_RELOAD_WS_PATH: &str = "/__marmite__/livereload";
const CONTENT_API_PATH: &str = "/__marmite__/content";
const CONFIG_API_PATH: &str = "/__marmite__/config";
const DATA_API_PATH: &str = "/__marmite__/data";
const FILES_API_PATH: &str = "/__marmite__/files";
const FILE_API_PATH: &str = "/__marmite__/file/";
const EDITOR_PAGE_PATH: &str = "/__marmite__/editor/";
const EDITOR_JS_PATH: &str = "__marmite__/editor.js";
const EDITOR_CSS_PATH: &str = "__marmite__/editor.css";
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

static TOOLBAR_CSS: LazyLock<String> = LazyLock::new(|| {
    crate::embedded::get_toolbar_asset("toolbar.css")
        .expect("embedded toolbar.css missing - this is a build-time error")
});

static TOOLBAR_JS: LazyLock<String> = LazyLock::new(|| {
    crate::embedded::get_toolbar_asset("toolbar.js")
        .expect("embedded toolbar.js missing - this is a build-time error")
});

static EDITOR_CSS: LazyLock<String> = LazyLock::new(|| {
    crate::embedded::get_toolbar_asset("editor.css")
        .expect("embedded editor.css missing - this is a build-time error")
});

static EDITOR_JS: LazyLock<String> = LazyLock::new(|| {
    crate::embedded::get_toolbar_asset("editor.js")
        .expect("embedded editor.js missing - this is a build-time error")
});

static EDITOR_HTML: LazyLock<String> = LazyLock::new(|| {
    crate::embedded::get_toolbar_asset("editor.html")
        .expect("embedded editor.html missing - this is a build-time error")
});

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

    if decoded_url == FILES_API_PATH {
        return Ok(handle_files_api(ctx));
    }

    if let Some(file_path) = decoded_url.strip_prefix(FILE_API_PATH) {
        if !file_path.is_empty() {
            return Ok(handle_file_api(request, file_path, ctx));
        }
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
        let mut response = Response::from_string(TOOLBAR_JS.as_str());
        if let Ok(h) = Header::from_bytes("Content-Type", "application/javascript; charset=utf-8") {
            response.add_header(h);
        }
        if let Ok(h) = Header::from_bytes("Cache-Control", "no-store") {
            response.add_header(h);
        }
        return Ok(response);
    }

    if decoded_url == format!("/{TOOLBAR_CSS_PATH}") {
        let mut response = Response::from_string(TOOLBAR_CSS.as_str());
        if let Ok(h) = Header::from_bytes("Content-Type", "text/css; charset=utf-8") {
            response.add_header(h);
        }
        if let Ok(h) = Header::from_bytes("Cache-Control", "no-store") {
            response.add_header(h);
        }
        return Ok(response);
    }

    if decoded_url == format!("/{EDITOR_JS_PATH}") {
        let mut response = Response::from_string(EDITOR_JS.as_str());
        if let Ok(h) = Header::from_bytes("Content-Type", "application/javascript; charset=utf-8") {
            response.add_header(h);
        }
        if let Ok(h) = Header::from_bytes("Cache-Control", "no-store") {
            response.add_header(h);
        }
        return Ok(response);
    }

    if decoded_url == format!("/{EDITOR_CSS_PATH}") {
        let mut response = Response::from_string(EDITOR_CSS.as_str());
        if let Ok(h) = Header::from_bytes("Content-Type", "text/css; charset=utf-8") {
            response.add_header(h);
        }
        if let Ok(h) = Header::from_bytes("Cache-Control", "no-store") {
            response.add_header(h);
        }
        return Ok(response);
    }

    if let Some(slug) = decoded_url.strip_prefix(EDITOR_PAGE_PATH) {
        return Ok(handle_editor_page(slug, live_reload_enabled));
    }
    if decoded_url == "/__marmite__/editor" {
        return Ok(handle_editor_page("", live_reload_enabled));
    }

    let url_without_query = decoded_url.split('?').next().unwrap_or(&decoded_url);
    let request_path = match url_without_query {
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
                        if ctx.enable_toolbar && !html.contains(TOOLBAR_CSS_PATH) {
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
        render_not_found(
            &error_path,
            &request_path,
            live_reload_enabled,
            ctx.enable_toolbar,
        )
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
        Method::Get if rest.ends_with("/body") => {
            let slug = rest.strip_suffix("/body").unwrap_or("");
            if slug.is_empty() {
                return json_response(400, &json!({"error": "slug is required in URL path"}));
            }
            handle_get_content_body(slug, ctx)
        }
        Method::Put if rest.ends_with("/body") => {
            let slug = rest.strip_suffix("/body").unwrap_or("");
            if slug.is_empty() {
                return json_response(400, &json!({"error": "slug is required in URL path"}));
            }
            handle_put_content_body(request, slug, ctx)
        }
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

    let shortcodes: Vec<String> = build_info
        .get("shortcodes")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let mut content_items: Vec<serde_json::Value> = Vec::new();
    for key in &["posts", "pages"] {
        if let Some(items) = build_info.get(key).and_then(|v| v.as_array()) {
            for item in items {
                let title = item
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let item_slug = item
                    .get("slug")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if !item_slug.is_empty() {
                    content_items.push(json!({"title": title, "slug": item_slug}));
                }
            }
        }
    }

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
            "shortcodes": shortcodes,
            "content_items": content_items,
            "post_count": build_info.get("posts").and_then(|v| v.as_array()).map_or(0, Vec::len),
            "page_count": build_info.get("pages").and_then(|v| v.as_array()).map_or(0, Vec::len),
            "elapsed_time": build_info.get("elapsed_time").and_then(serde_json::Value::as_f64).unwrap_or(0.0),
            "marmite_version": build_info.get("marmite_version").and_then(|v| v.as_str()).unwrap_or(""),
            "config": config,
        }),
    )
}

fn handle_file_api(
    request: &mut Request,
    rel_path: &str,
    ctx: &ServerContext,
) -> Response<Cursor<Vec<u8>>> {
    let input_folder = ctx.input_folder.as_path();
    let file_path = input_folder.join(rel_path);

    // Safety: ensure the resolved path is inside the input folder
    let canonical_input = input_folder
        .canonicalize()
        .unwrap_or_else(|_| input_folder.to_path_buf());
    let canonical_file = file_path
        .canonicalize()
        .unwrap_or_else(|_| file_path.clone());
    if !canonical_file.starts_with(&canonical_input) {
        return json_response(403, &json!({"error": "path traversal not allowed"}));
    }

    // Block editing binary/image files
    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let binary_exts = [
        "png", "jpg", "jpeg", "gif", "webp", "avif", "bmp", "tiff", "ico", "svg", "woff", "woff2",
        "ttf", "otf", "pdf", "zip", "tar", "gz",
    ];
    if binary_exts.contains(&ext.as_str()) {
        return json_response(400, &json!({"error": "binary files cannot be edited"}));
    }

    match *request.method() {
        Method::Get => {
            if !file_path.is_file() {
                return json_response(
                    404,
                    &json!({"error": format!("file not found: {rel_path}")}),
                );
            }
            match std::fs::read_to_string(&file_path) {
                Ok(content) => json_response(200, &json!({ "path": rel_path, "content": content })),
                Err(e) => {
                    json_response(500, &json!({"error": format!("failed to read file: {e}")}))
                }
            }
        }
        Method::Put => {
            let body = match read_request_body(request) {
                Ok(b) => b,
                Err(e) => return json_response(400, &json!({"error": e})),
            };
            let parsed: serde_json::Value = match serde_json::from_str(&body) {
                Ok(v) => v,
                Err(e) => {
                    return json_response(400, &json!({"error": format!("Invalid JSON: {e}")}))
                }
            };
            let Some(content) = parsed.get("content").and_then(|v| v.as_str()) else {
                return json_response(400, &json!({"error": "content field is required"}));
            };
            if let Some(parent) = file_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match std::fs::write(&file_path, content) {
                Ok(()) => json_response(200, &json!({"path": rel_path})),
                Err(e) => {
                    json_response(500, &json!({"error": format!("failed to write file: {e}")}))
                }
            }
        }
        _ => json_response(405, &json!({"error": "method not allowed"})),
    }
}

fn handle_files_api(ctx: &ServerContext) -> Response<Cursor<Vec<u8>>> {
    let input_folder = ctx.input_folder.as_path();
    let site_data = crate::site::Data::from_file(&ctx.config_path);
    let content_path = &site_data.site.content_path;
    let site_path = &site_data.site.site_path;
    let output_rel = ctx
        .output_folder
        .strip_prefix(input_folder)
        .ok()
        .and_then(|p| p.to_str())
        .map(String::from);

    let mut files: Vec<serde_json::Value> = Vec::new();

    for entry in walkdir::WalkDir::new(input_folder)
        .sort_by_file_name()
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Ok(rel) = path.strip_prefix(input_folder) else {
            continue;
        };
        let rel_str = rel.to_string_lossy().to_string();

        // Skip output folder and hidden directories
        if (!site_path.is_empty() && rel_str.starts_with(site_path))
            || output_rel
                .as_ref()
                .is_some_and(|op| rel_str.starts_with(op.as_str()))
            || rel_str.starts_with('.')
            || rel
                .components()
                .any(|c| c.as_os_str().to_str().is_some_and(|s| s.starts_with('.')))
        {
            continue;
        }

        let is_md = path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"));
        let is_content = rel_str.starts_with(content_path) && is_md;
        let file_ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let editable_exts = [
            "md", "txt", "css", "js", "json", "yaml", "yml", "toml", "html", "xml", "svg", "csv",
            "sh",
        ];
        let editable = editable_exts.contains(&file_ext.as_str());
        let is_fragment = is_md
            && path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with('_'));

        let mut entry_json = json!({ "path": rel_str });

        if is_content {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                let slug = crate::content::remove_date_from_filename(stem);
                if !is_fragment {
                    entry_json["slug"] = json!(slug);
                }
            }
        }
        if editable {
            entry_json["editable"] = json!(true);
        }
        if is_fragment {
            entry_json["fragment"] = json!(true);
        }

        files.push(entry_json);
    }

    json_response(200, &json!({ "files": files }))
}

fn handle_config_api(request: &mut Request, ctx: &ServerContext) -> Response<Cursor<Vec<u8>>> {
    match *request.method() {
        Method::Get => handle_get_config_raw(ctx),
        Method::Post => handle_create_config(ctx),
        Method::Patch => handle_patch_config(request, ctx),
        Method::Put => handle_put_config_raw(request, ctx),
        _ => json_response(405, &json!({"error": "method not allowed"})),
    }
}

fn handle_get_config_raw(ctx: &ServerContext) -> Response<Cursor<Vec<u8>>> {
    let config_path = ctx.config_path.as_path();
    let content = if config_path.exists() {
        std::fs::read_to_string(config_path).unwrap_or_default()
    } else {
        String::new()
    };
    json_response(
        200,
        &json!({ "file": config_path.display().to_string(), "yaml": content }),
    )
}

fn handle_put_config_raw(request: &mut Request, ctx: &ServerContext) -> Response<Cursor<Vec<u8>>> {
    let body = match read_request_body(request) {
        Ok(b) => b,
        Err(e) => return json_response(400, &json!({"error": e})),
    };

    let parsed: serde_json::Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(e) => return json_response(400, &json!({"error": format!("Invalid JSON: {e}")})),
    };

    let Some(yaml_content) = parsed.get("yaml").and_then(|v| v.as_str()) else {
        return json_response(400, &json!({"error": "yaml field is required"}));
    };

    // Validate that the YAML is parseable
    if !yaml_content.trim().is_empty() {
        if let Err(e) = serde_yaml::from_str::<serde_yaml::Value>(yaml_content) {
            return json_response(400, &json!({"error": format!("Invalid YAML: {e}")}));
        }
    }

    let config_path = ctx.config_path.as_path();
    if let Err(e) = std::fs::write(config_path, yaml_content) {
        return json_response(
            500,
            &json!({"error": format!("Failed to write config: {e}")}),
        );
    }

    json_response(200, &json!({ "file": config_path.display().to_string() }))
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

fn handle_get_content_body(slug: &str, ctx: &ServerContext) -> Response<Cursor<Vec<u8>>> {
    let site_data = crate::site::Data::from_file(&ctx.config_path);
    let content_folder = crate::site::get_content_folder(&site_data.site, &ctx.input_folder);

    match crate::content::get_raw_content(&content_folder, slug) {
        Ok((frontmatter, body, file_path)) => {
            let source_path = file_path
                .strip_prefix(&*ctx.input_folder)
                .unwrap_or(&file_path)
                .display()
                .to_string();
            json_response(
                200,
                &json!({
                    "slug": slug,
                    "frontmatter": frontmatter,
                    "body": body,
                    "source_path": source_path,
                }),
            )
        }
        Err(e) if e.contains("not found") => json_response(404, &json!({"error": e})),
        Err(e) => json_response(500, &json!({"error": e})),
    }
}

fn handle_put_content_body(
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

    let Some(new_body) = parsed.get("body").and_then(|v| v.as_str()) else {
        return json_response(400, &json!({"error": "body field is required"}));
    };

    let fm_updates = parsed
        .get("frontmatter")
        .and_then(|v| v.as_object())
        .cloned();

    let site_data = crate::site::Data::from_file(&ctx.config_path);
    let content_folder = crate::site::get_content_folder(&site_data.site, &ctx.input_folder);

    match crate::content::update_content_body(&content_folder, slug, new_body, fm_updates.as_ref())
    {
        Ok(frontmatter) => json_response(
            200,
            &json!({
                "slug": slug,
                "frontmatter": frontmatter,
            }),
        ),
        Err(e) if e.contains("not found") => json_response(404, &json!({"error": e})),
        Err(e) => json_response(500, &json!({"error": e})),
    }
}

fn handle_editor_page(slug: &str, _live_reload_enabled: bool) -> Response<Cursor<Vec<u8>>> {
    let escaped_slug = slug
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('<', "&lt;")
        .replace('>', "&gt;");

    let html = EDITOR_HTML.replace("{slug}", &escaped_slug);

    let mut resp = Response::from_string(html).with_status_code(200);
    if let Ok(h) = Header::from_bytes("Content-Type", "text/html; charset=utf-8") {
        resp.add_header(h);
    }
    if let Ok(h) = Header::from_bytes("Cache-Control", "no-store") {
        resp.add_header(h);
    }
    resp
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
    enable_toolbar: bool,
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
                let toolbar_tag = if enable_toolbar {
                    format!(
                        "<script>window.__marmite_404_slug__={slug};</script>\n\
                         <link rel=\"stylesheet\" href=\"/{TOOLBAR_CSS_PATH}\">\n\
                         <script src=\"/{TOOLBAR_JS_PATH}\"></script>\n",
                        slug = serde_json::to_string(slug).unwrap_or_else(|_| "null".into()),
                    )
                } else {
                    String::new()
                };
                let inject = format!("{toolbar_tag}{live_reload_tag}");
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
