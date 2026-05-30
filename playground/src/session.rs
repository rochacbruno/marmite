use crate::config::AppConfig;
use crate::error::AppError;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Instant;

pub struct Session {
    pub id: uuid::Uuid,
    pub owner_token: uuid::Uuid,
    pub root_dir: PathBuf,
    pub input_dir: PathBuf,
    pub output_dir: PathBuf,
    pub last_activity: Instant,
}

pub type SessionStore = Arc<RwLock<HashMap<uuid::Uuid, Session>>>;

const SEED_MARMITE_YAML: &str = include_str!("../seed/marmite.yaml");
const SEED_ABOUT: &str = include_str!("../seed/content/about.md");
const SEED_HELLO: &str = include_str!("../seed/content/2024-01-01-hello-world.md");
const SEED_GOODBYE: &str = include_str!("../seed/content/2024-01-02-goodbye-world.md");

pub async fn create_session(config: &AppConfig, store: &SessionStore) -> Result<Session, AppError> {
    let id = uuid::Uuid::new_v4();
    let owner_token = uuid::Uuid::new_v4();
    let root_dir = config.sessions_dir.join(id.to_string());
    let input_dir = root_dir.join("input");
    let output_dir = root_dir.join("output");

    tokio::fs::create_dir_all(&input_dir.join("content")).await?;
    tokio::fs::create_dir_all(&input_dir.join("static")).await?;
    tokio::fs::create_dir_all(&input_dir.join("templates")).await?;
    tokio::fs::create_dir_all(&output_dir).await?;

    seed_session(&input_dir).await?;

    let session = Session {
        id,
        owner_token,
        root_dir,
        input_dir,
        output_dir,
        last_activity: Instant::now(),
    };

    store.write().await.insert(
        id,
        Session {
            id: session.id,
            owner_token: session.owner_token,
            root_dir: session.root_dir.clone(),
            input_dir: session.input_dir.clone(),
            output_dir: session.output_dir.clone(),
            last_activity: session.last_activity,
        },
    );

    Ok(session)
}

pub async fn clone_session(
    source_id: uuid::Uuid,
    config: &AppConfig,
    store: &SessionStore,
) -> Result<Session, AppError> {
    let source_input = {
        let sessions = store.read().await;
        let (_, input_dir, _) =
            get_session_sync(source_id, &sessions).ok_or(AppError::SessionNotFound(source_id))?;
        input_dir
    };

    let id = uuid::Uuid::new_v4();
    let owner_token = uuid::Uuid::new_v4();
    let root_dir = config.sessions_dir.join(id.to_string());
    let input_dir = root_dir.join("input");
    let output_dir = root_dir.join("output");

    tokio::fs::create_dir_all(&output_dir).await?;
    copy_dir_recursive(&source_input, &input_dir).await?;

    let session = Session {
        id,
        owner_token,
        root_dir,
        input_dir,
        output_dir,
        last_activity: Instant::now(),
    };

    store.write().await.insert(
        id,
        Session {
            id: session.id,
            owner_token: session.owner_token,
            root_dir: session.root_dir.clone(),
            input_dir: session.input_dir.clone(),
            output_dir: session.output_dir.clone(),
            last_activity: session.last_activity,
        },
    );

    Ok(session)
}

async fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    tokio::fs::create_dir_all(dst).await?;
    let mut entries = tokio::fs::read_dir(src).await?;
    while let Some(entry) = entries.next_entry().await? {
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            Box::pin(copy_dir_recursive(&src_path, &dst_path)).await?;
        } else {
            tokio::fs::copy(&src_path, &dst_path).await?;
        }
    }
    Ok(())
}

pub async fn touch_session(id: uuid::Uuid, store: &SessionStore) {
    if let Some(session) = store.write().await.get_mut(&id) {
        session.last_activity = Instant::now();
    }
}

pub fn get_session_sync(
    id: uuid::Uuid,
    sessions: &HashMap<uuid::Uuid, Session>,
) -> Option<(&Session, PathBuf, PathBuf)> {
    let s = sessions.get(&id)?;
    Some((s, s.input_dir.clone(), s.output_dir.clone()))
}

async fn seed_session(input_dir: &Path) -> Result<(), std::io::Error> {
    let content_dir = input_dir.join("content");
    tokio::fs::write(input_dir.join("marmite.yaml"), SEED_MARMITE_YAML).await?;
    tokio::fs::write(content_dir.join("about.md"), SEED_ABOUT).await?;
    tokio::fs::write(content_dir.join("2024-01-01-hello-world.md"), SEED_HELLO).await?;
    tokio::fs::write(
        content_dir.join("2024-01-02-goodbye-world.md"),
        SEED_GOODBYE,
    )
    .await?;
    Ok(())
}

pub async fn cleanup_task(config: AppConfig, store: SessionStore) {
    let interval = std::time::Duration::from_secs(config.cleanup_interval_secs);
    let ttl = std::time::Duration::from_secs(config.session_ttl_secs);

    loop {
        tokio::time::sleep(interval).await;

        let mut sessions = store.write().await;
        let expired: Vec<uuid::Uuid> = sessions
            .iter()
            .filter(|(_, s)| s.last_activity.elapsed() > ttl)
            .map(|(id, _)| *id)
            .collect();

        for id in expired {
            if let Some(session) = sessions.remove(&id) {
                tracing::info!("cleaning up expired session {id}");
                if let Err(e) = tokio::fs::remove_dir_all(&session.root_dir).await {
                    tracing::warn!(
                        "failed to remove session dir {}: {e}",
                        session.root_dir.display()
                    );
                }
            }
        }
    }
}

pub async fn list_files_in(input_dir: &Path) -> Result<Vec<String>, std::io::Error> {
    let mut files = Vec::new();
    collect_files(input_dir, input_dir, &mut files).await?;
    files.sort();
    Ok(files)
}

async fn collect_files(
    base: &Path,
    dir: &Path,
    out: &mut Vec<String>,
) -> Result<(), std::io::Error> {
    let mut entries = tokio::fs::read_dir(dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            Box::pin(collect_files(base, &path, out)).await?;
        } else if let Ok(rel) = path.strip_prefix(base) {
            out.push(rel.to_string_lossy().to_string());
        }
    }
    Ok(())
}
