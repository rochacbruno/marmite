use log::{error, info, warn};
use std::path::Path;
use std::process::Command;

/// Check all links in the generated website using Lychee binary
pub fn check_links(output_folder: &Path, base_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting link checking with Lychee...");

    // Check if lychee is available
    if !is_lychee_installed() {
        warn!("Lychee is not installed. Install it with: cargo install lychee");
        warn!("Skipping link checking...");
        return Ok(());
    }

    // Run lychee on the output folder
    let mut cmd = Command::new("lychee");
    cmd.arg("--verbose")
        .arg("--no-progress")
        .arg("--format")
        .arg("json")
        .arg(format!("{}/**/*.html", output_folder.display()));

    // Add base URL to check internal links
    if !base_url.is_empty() {
        cmd.arg("--base").arg(base_url);
    }

    info!("Running: {:?}", cmd);

    let output = cmd.output()?;

    if output.status.success() {
        info!("Link checking completed successfully!");
        info!("Output: {}", String::from_utf8_lossy(&output.stdout));
    } else {
        error!("Link checking found broken links!");
        error!("Error output: {}", String::from_utf8_lossy(&output.stderr));
        error!("Output: {}", String::from_utf8_lossy(&output.stdout));
        return Err("Link checking failed with broken links".into());
    }

    Ok(())
}

/// Check if lychee binary is installed
fn is_lychee_installed() -> bool {
    Command::new("lychee").arg("--version").output().is_ok()
}
