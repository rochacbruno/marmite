use log::{error, info};
use std::fs;
use std::path::Path;

// Copy or create robots.txt
const ROBOTS_SRC: &str = "robots.txt";
const DEFAULT_ROBOTS: &str = "User-agent: *
Disallow: /private
Allow: /public";

pub fn handle_robots(content_dir: &Path, output_path: &Path) {
    let robots_src = content_dir.join(ROBOTS_SRC);
    let robots_dst = output_path.join(ROBOTS_SRC);

    match robots_src.exists() {
        true => {
            if let Err(e) = fs::copy(&robots_src, &robots_dst) {
                error!("Failed to copy robots.txt: {}", e);
            } else {
                info!("Copied robots.txt to output folder");
            }
        }
        false => {
            if let Err(e) = fs::write(&robots_dst, DEFAULT_ROBOTS) {
                error!("Failed to create default robots.txt: {}", e);
            } else {
                info!("Generated default robots.txt in output folder");
            }
        }
    }
}
