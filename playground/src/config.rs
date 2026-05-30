use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum RenderMode {
    Docker,
    Binary,
    Auto,
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub bind_addr: String,
    pub render_mode: RenderMode,
    pub marmite_cmd: Option<String>,
    pub docker_image: String,
    pub sessions_dir: PathBuf,
    pub session_ttl_secs: u64,
    pub cleanup_interval_secs: u64,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let render_mode = match std::env::var("MARMITE_RENDER_MODE")
            .unwrap_or_default()
            .as_str()
        {
            "docker" => RenderMode::Docker,
            "binary" => RenderMode::Binary,
            _ => RenderMode::Auto,
        };

        let sessions_dir = std::env::var("PLAYGROUND_SESSIONS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir().join("marmite-playground"));

        Self {
            bind_addr: std::env::var("PLAYGROUND_BIND")
                .unwrap_or_else(|_| "0.0.0.0:3000".to_string()),
            render_mode,
            marmite_cmd: std::env::var("MARMITE_CMD").ok(),
            docker_image: std::env::var("MARMITE_DOCKER_IMAGE")
                .unwrap_or_else(|_| "ghcr.io/rochacbruno/marmite:latest".to_string()),
            sessions_dir,
            session_ttl_secs: std::env::var("PLAYGROUND_SESSION_TTL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3600),
            cleanup_interval_secs: std::env::var("PLAYGROUND_CLEANUP_INTERVAL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(300),
        }
    }
}
