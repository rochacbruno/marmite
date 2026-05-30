use crate::config::{AppConfig, RenderMode};
use crate::error::AppError;
use serde::Serialize;
use std::path::Path;
use std::time::Duration;

#[derive(Serialize)]
pub struct RenderResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

static RENDER_SEMAPHORE: std::sync::LazyLock<tokio::sync::Semaphore> =
    std::sync::LazyLock::new(|| tokio::sync::Semaphore::new(4));

pub async fn render(
    input_dir: &Path,
    output_dir: &Path,
    config: &AppConfig,
) -> Result<RenderResult, AppError> {
    let _permit = RENDER_SEMAPHORE
        .acquire()
        .await
        .map_err(|e| AppError::RenderFailed(e.to_string()))?;

    let mode = effective_mode(config).await;
    match mode {
        RenderMode::Docker => render_docker(input_dir, &config.docker_image).await,
        RenderMode::Binary => {
            let cmd = config.marmite_cmd.as_deref().unwrap_or("marmite");
            render_binary(input_dir, output_dir, cmd).await
        }
        RenderMode::Auto => unreachable!(),
    }
}

pub fn output_dir_for_mode(
    config: &AppConfig,
    input_dir: &Path,
    fallback_output: &Path,
) -> std::path::PathBuf {
    match config.render_mode {
        RenderMode::Docker => input_dir.join("site"),
        RenderMode::Binary | RenderMode::Auto => fallback_output.to_path_buf(),
    }
}

async fn effective_mode(config: &AppConfig) -> RenderMode {
    match config.render_mode {
        RenderMode::Docker => RenderMode::Docker,
        RenderMode::Binary => RenderMode::Binary,
        RenderMode::Auto => {
            if docker_available().await {
                RenderMode::Docker
            } else {
                RenderMode::Binary
            }
        }
    }
}

async fn docker_available() -> bool {
    tokio::process::Command::new("docker")
        .arg("info")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}

async fn render_binary(
    input_dir: &Path,
    output_dir: &Path,
    cmd: &str,
) -> Result<RenderResult, AppError> {
    let start = tokio::time::Instant::now();

    let result = tokio::time::timeout(
        Duration::from_secs(30),
        tokio::process::Command::new(cmd)
            .arg(input_dir)
            .arg(output_dir)
            .output(),
    )
    .await;

    match result {
        Ok(Ok(output)) => Ok(RenderResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
        }),
        Ok(Err(e)) => Err(AppError::RenderFailed(format!("failed to run {cmd}: {e}"))),
        Err(_) => Err(AppError::RenderFailed("render timed out (30s)".to_string())),
    }
}

async fn render_docker(input_dir: &Path, image: &str) -> Result<RenderResult, AppError> {
    let start = tokio::time::Instant::now();
    let input_str = input_dir.to_string_lossy();

    let result = tokio::time::timeout(
        Duration::from_secs(60),
        tokio::process::Command::new("docker")
            .args(["run", "--rm", "-v", &format!("{input_str}:/input"), image])
            .output(),
    )
    .await;

    match result {
        Ok(Ok(output)) => Ok(RenderResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
        }),
        Ok(Err(e)) => Err(AppError::RenderFailed(format!("docker run failed: {e}"))),
        Err(_) => Err(AppError::RenderFailed("render timed out (60s)".to_string())),
    }
}
