use chrono::Utc;
use log::{error, info, warn};
use serde_json::json;
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
const LIVE_RELOAD_WS_PATH: &str = "/__marmite__/livereload";
const CONTENT_API_PATH: &str = "/__marmite__/content";
const CONFIG_API_PATH: &str = "/__marmite__/config";
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

#[allow(clippy::case_sensitive_file_extension_comparisons)]
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
                if live_reload_enabled && request_path.ends_with(".html") {
                    let original_buffer = buffer.clone();
                    if let Ok(mut html) = String::from_utf8(buffer) {
                        if !html.contains(LIVE_RELOAD_SCRIPT_PATH) {
                            let snippet =
                                format!("\n<script src=\"/{LIVE_RELOAD_SCRIPT_PATH}\"></script>\n");
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
        render_not_found(&error_path)
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
    match *request.method() {
        Method::Post if url == CONTENT_API_PATH => handle_create_content(request, ctx),
        Method::Patch => {
            let slug = url
                .strip_prefix(CONTENT_API_PATH)
                .and_then(|s| s.strip_prefix('/'))
                .unwrap_or("");
            if slug.is_empty() {
                return json_response(400, &json!({"error": "slug is required in URL path"}));
            }
            handle_patch_content(request, slug, ctx)
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

fn render_not_found(error_path: &PathBuf) -> Result<Response<Cursor<Vec<u8>>>, String> {
    match File::open(error_path) {
        Ok(mut file) => {
            let mut buffer = Vec::new();
            std::io::copy(&mut file, &mut buffer).map_err(|e| e.to_string())?;
            let resp = Response::from_data(buffer);
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
