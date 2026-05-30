use crate::error::AppError;
use crate::renderer;
use crate::session::{self, SessionStore};
use axum::body::Body;
use axum::extract::{Multipart, Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;

use crate::config::AppConfig;

pub struct AppState {
    pub config: AppConfig,
    pub sessions: SessionStore,
}

#[derive(Serialize)]
pub struct CreateSessionResponse {
    session_id: String,
    owner_token: String,
    files: Vec<String>,
}

#[derive(Serialize)]
pub struct SessionInfoResponse {
    session_id: String,
    files: Vec<String>,
}

#[derive(Serialize)]
pub struct FileResponse {
    path: String,
    content: String,
}

#[derive(Serialize)]
pub struct ListFilesResponse {
    files: Vec<String>,
}

#[derive(Deserialize)]
pub struct WriteFileRequest {
    content: String,
}

#[derive(Serialize)]
pub struct WriteFileResponse {
    path: String,
    saved: bool,
}

#[derive(Serialize)]
pub struct DeleteFileResponse {
    path: String,
    deleted: bool,
}

fn extract_owner_token(headers: &HeaderMap) -> Option<uuid::Uuid> {
    headers
        .get("x-owner-token")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
}

async fn verify_owner(
    state: &AppState,
    id: uuid::Uuid,
    headers: &HeaderMap,
) -> Result<(), AppError> {
    let token = extract_owner_token(headers).ok_or(AppError::Forbidden)?;
    let sessions = state.sessions.read().await;
    let (session, _, _) =
        session::get_session_sync(id, &sessions).ok_or(AppError::SessionNotFound(id))?;
    if session.owner_token != token {
        return Err(AppError::Forbidden);
    }
    Ok(())
}

pub async fn create_session_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<CreateSessionResponse>, AppError> {
    let session = session::create_session(&state.config, &state.sessions).await?;
    let files = session::list_files_in(&session.input_dir).await?;
    Ok(Json(CreateSessionResponse {
        session_id: session.id.to_string(),
        owner_token: session.owner_token.to_string(),
        files,
    }))
}

pub async fn clone_session_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<CreateSessionResponse>, AppError> {
    let session = session::clone_session(id, &state.config, &state.sessions).await?;
    let files = session::list_files_in(&session.input_dir).await?;
    Ok(Json(CreateSessionResponse {
        session_id: session.id.to_string(),
        owner_token: session.owner_token.to_string(),
        files,
    }))
}

pub async fn get_session_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<SessionInfoResponse>, AppError> {
    let sessions = state.sessions.read().await;
    let (_, input_dir, _) =
        session::get_session_sync(id, &sessions).ok_or(AppError::SessionNotFound(id))?;
    let files = session::list_files_in(&input_dir).await?;
    drop(sessions);

    Ok(Json(SessionInfoResponse {
        session_id: id.to_string(),
        files,
    }))
}

pub async fn list_files_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<ListFilesResponse>, AppError> {
    let sessions = state.sessions.read().await;
    let (_, input_dir, _) =
        session::get_session_sync(id, &sessions).ok_or(AppError::SessionNotFound(id))?;
    let files = session::list_files_in(&input_dir).await?;
    drop(sessions);

    Ok(Json(ListFilesResponse { files }))
}

pub async fn read_file_handler(
    State(state): State<Arc<AppState>>,
    Path((id, file_path)): Path<(uuid::Uuid, String)>,
) -> Result<Json<FileResponse>, AppError> {
    let sessions = state.sessions.read().await;
    let (_, input_dir, _) =
        session::get_session_sync(id, &sessions).ok_or(AppError::SessionNotFound(id))?;
    let resolved = validate_path(&input_dir, &file_path)?;
    let content = tokio::fs::read_to_string(&resolved)
        .await
        .map_err(|_| AppError::FileNotFound(file_path.clone()))?;
    drop(sessions);

    Ok(Json(FileResponse {
        path: file_path,
        content,
    }))
}

pub async fn write_file_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((id, file_path)): Path<(uuid::Uuid, String)>,
    Json(body): Json<WriteFileRequest>,
) -> Result<Json<WriteFileResponse>, AppError> {
    verify_owner(&state, id, &headers).await?;
    let sessions = state.sessions.read().await;
    let (_, input_dir, _) =
        session::get_session_sync(id, &sessions).ok_or(AppError::SessionNotFound(id))?;
    let resolved = validate_path_for_write(&input_dir, &file_path)?;
    if let Some(parent) = resolved.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(&resolved, &body.content).await?;
    drop(sessions);
    session::touch_session(id, &state.sessions).await;
    Ok(Json(WriteFileResponse {
        path: file_path,
        saved: true,
    }))
}

pub async fn delete_file_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((id, file_path)): Path<(uuid::Uuid, String)>,
) -> Result<Json<DeleteFileResponse>, AppError> {
    verify_owner(&state, id, &headers).await?;
    let sessions = state.sessions.read().await;
    let (_, input_dir, _) =
        session::get_session_sync(id, &sessions).ok_or(AppError::SessionNotFound(id))?;
    let resolved = validate_path(&input_dir, &file_path)?;
    tokio::fs::remove_file(&resolved)
        .await
        .map_err(|_| AppError::FileNotFound(file_path.clone()))?;
    drop(sessions);
    session::touch_session(id, &state.sessions).await;
    Ok(Json(DeleteFileResponse {
        path: file_path,
        deleted: true,
    }))
}

pub async fn render_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<renderer::RenderResult>, AppError> {
    verify_owner(&state, id, &headers).await?;
    let (input_dir, output_dir) = {
        let sessions = state.sessions.read().await;
        let (_, input_dir, output_dir) =
            session::get_session_sync(id, &sessions).ok_or(AppError::SessionNotFound(id))?;
        (input_dir, output_dir)
    };

    let effective_output = renderer::output_dir_for_mode(&state.config, &input_dir, &output_dir);
    let result = renderer::render(&input_dir, &effective_output, &state.config).await?;
    session::touch_session(id, &state.sessions).await;
    Ok(Json(result))
}

pub async fn download_source_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Response, AppError> {
    let input_dir = {
        let sessions = state.sessions.read().await;
        let (_, input_dir, _) =
            session::get_session_sync(id, &sessions).ok_or(AppError::SessionNotFound(id))?;
        input_dir
    };

    let tarball = create_tarball(&input_dir)?;
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/gzip"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"marmite-source.tar.gz\"",
            ),
        ],
        Body::from(tarball),
    )
        .into_response())
}

pub async fn download_site_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Response, AppError> {
    let (input_dir, output_dir) = {
        let sessions = state.sessions.read().await;
        let (_, input_dir, output_dir) =
            session::get_session_sync(id, &sessions).ok_or(AppError::SessionNotFound(id))?;
        (input_dir, output_dir)
    };
    let effective_output = renderer::output_dir_for_mode(&state.config, &input_dir, &output_dir);
    if !effective_output.exists() {
        return Err(AppError::RenderFailed(
            "site has not been rendered yet".to_string(),
        ));
    }

    let tarball = create_tarball(&effective_output)?;
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/gzip"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"marmite-site.tar.gz\"",
            ),
        ],
        Body::from(tarball),
    )
        .into_response())
}

pub async fn upload_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<uuid::Uuid>,
    mut multipart: Multipart,
) -> Result<Json<ListFilesResponse>, AppError> {
    verify_owner(&state, id, &headers).await?;

    let input_dir = {
        let sessions = state.sessions.read().await;
        let (_, input_dir, _) =
            session::get_session_sync(id, &sessions).ok_or(AppError::SessionNotFound(id))?;
        input_dir
    };

    let field = multipart
        .next_field()
        .await
        .map_err(|e| AppError::RenderFailed(format!("multipart error: {e}")))?
        .ok_or_else(|| AppError::RenderFailed("no file uploaded".to_string()))?;

    let filename = field.file_name().unwrap_or("upload").to_string();
    let data = field
        .bytes()
        .await
        .map_err(|e| AppError::RenderFailed(format!("failed to read upload: {e}")))?;

    // Clear input dir
    let mut entries = tokio::fs::read_dir(&input_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            tokio::fs::remove_dir_all(&path).await?;
        } else {
            tokio::fs::remove_file(&path).await?;
        }
    }

    // Extract based on format
    if filename.ends_with(".zip") {
        extract_zip(&data, &input_dir)?;
    } else {
        extract_tarball(&data, &input_dir)?;
    }

    // Ensure standard dirs exist
    tokio::fs::create_dir_all(input_dir.join("content")).await?;
    tokio::fs::create_dir_all(input_dir.join("static")).await?;
    tokio::fs::create_dir_all(input_dir.join("templates")).await?;

    let files = session::list_files_in(&input_dir).await?;
    session::touch_session(id, &state.sessions).await;
    Ok(Json(ListFilesResponse { files }))
}

pub async fn preview_handler(
    State(state): State<Arc<AppState>>,
    Path((id, path)): Path<(uuid::Uuid, String)>,
) -> Result<Response, AppError> {
    serve_preview(&state, id, &path).await
}

pub async fn preview_root_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Response, AppError> {
    serve_preview(&state, id, "").await
}

async fn serve_preview(state: &AppState, id: uuid::Uuid, path: &str) -> Result<Response, AppError> {
    let (input_dir, output_dir) = {
        let sessions = state.sessions.read().await;
        let (_, input_dir, output_dir) =
            session::get_session_sync(id, &sessions).ok_or(AppError::SessionNotFound(id))?;
        (input_dir, output_dir)
    };

    let effective_output = renderer::output_dir_for_mode(&state.config, &input_dir, &output_dir);

    let file_path = if path.is_empty() || path == "/" {
        "index.html"
    } else {
        path
    };

    let resolved = effective_output.join(file_path);
    if !resolved.starts_with(&effective_output) {
        return Err(AppError::PathTraversal(file_path.to_string()));
    }

    let content = tokio::fs::read(&resolved)
        .await
        .map_err(|_| AppError::FileNotFound(file_path.to_string()))?;

    let content_type = mime_for_path(&resolved);

    if content_type.starts_with("text/html") {
        let html = String::from_utf8_lossy(&content);
        let base_path = format!("/preview/{id}/");
        let rewritten = rewrite_absolute_urls(&html, &base_path);
        Ok((
            StatusCode::OK,
            [(header::CONTENT_TYPE, content_type)],
            Body::from(rewritten),
        )
            .into_response())
    } else {
        Ok((
            StatusCode::OK,
            [(header::CONTENT_TYPE, content_type)],
            Body::from(content),
        )
            .into_response())
    }
}

fn rewrite_absolute_urls(html: &str, base_path: &str) -> String {
    html.replace("href=\"/", &format!("href=\"{base_path}"))
        .replace("src=\"/", &format!("src=\"{base_path}"))
        .replace("action=\"/", &format!("action=\"{base_path}"))
}

fn validate_path(base: &std::path::Path, relative: &str) -> Result<PathBuf, AppError> {
    if relative.contains("..") {
        return Err(AppError::PathTraversal(relative.to_string()));
    }
    let target = base.join(relative);
    if !target.starts_with(base) {
        return Err(AppError::PathTraversal(relative.to_string()));
    }
    Ok(target)
}

fn validate_path_for_write(base: &std::path::Path, relative: &str) -> Result<PathBuf, AppError> {
    if relative.contains("..") {
        return Err(AppError::PathTraversal(relative.to_string()));
    }
    let target = base.join(relative);
    if !target.starts_with(base) {
        return Err(AppError::PathTraversal(relative.to_string()));
    }
    Ok(target)
}

fn mime_for_path(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("xml") => "application/xml; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("txt") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn create_tarball(dir: &std::path::Path) -> Result<Vec<u8>, AppError> {
    let buf = Vec::new();
    let enc = GzEncoder::new(buf, Compression::default());
    let mut tar = tar::Builder::new(enc);

    fn add_dir(
        tar: &mut tar::Builder<GzEncoder<Vec<u8>>>,
        dir: &std::path::Path,
        base: &std::path::Path,
    ) -> Result<(), AppError> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let rel = path
                .strip_prefix(base)
                .map_err(|e| AppError::RenderFailed(e.to_string()))?;
            if path.is_dir() {
                add_dir(tar, &path, base)?;
            } else {
                tar.append_path_with_name(&path, rel)
                    .map_err(|e| AppError::RenderFailed(e.to_string()))?;
            }
        }
        Ok(())
    }

    add_dir(&mut tar, dir, dir)?;
    let enc = tar
        .into_inner()
        .map_err(|e| AppError::RenderFailed(e.to_string()))?;
    enc.finish().map_err(AppError::Io)
}

fn extract_tarball(data: &[u8], dest: &std::path::Path) -> Result<(), AppError> {
    let decoder = GzDecoder::new(Cursor::new(data));
    let mut archive = tar::Archive::new(decoder);

    for entry in archive
        .entries()
        .map_err(|e| AppError::RenderFailed(format!("invalid tar.gz: {e}")))?
    {
        let mut entry =
            entry.map_err(|e| AppError::RenderFailed(format!("tar entry error: {e}")))?;
        let path = entry
            .path()
            .map_err(|e| AppError::RenderFailed(format!("tar path error: {e}")))?
            .to_path_buf();

        if path
            .components()
            .any(|c| c == std::path::Component::ParentDir)
        {
            continue;
        }

        let target = dest.join(&path);
        if entry.header().entry_type().is_dir() {
            std::fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut file = std::fs::File::create(&target)?;
            std::io::copy(&mut entry, &mut file)?;
        }
    }
    Ok(())
}

fn extract_zip(data: &[u8], dest: &std::path::Path) -> Result<(), AppError> {
    let cursor = Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| AppError::RenderFailed(format!("invalid zip: {e}")))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| AppError::RenderFailed(format!("zip entry error: {e}")))?;

        let Some(path) = file.enclosed_name() else {
            continue;
        };
        let path = path.to_path_buf();

        let target = dest.join(&path);
        if file.is_dir() {
            std::fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut out = std::fs::File::create(&target)?;
            std::io::copy(&mut file, &mut out)?;
        }
    }
    Ok(())
}
