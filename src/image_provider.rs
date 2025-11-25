use frontmatter_gen::Frontmatter;
use log::{info, warn};
use std::{fs, path::Path};

use crate::config::{ImageProvider, Marmite};

pub fn download_banner_image(
    config: &Marmite,
    frontmatter: &Frontmatter,
    content_path: &Path,
    slug: &str,
    tags: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    match &config.image_provider {
        Some(ImageProvider::Picsum) => {
            download_picsum_image(config, frontmatter, content_path, slug, tags)
        }
        None => Ok(()),
    }
}

fn download_picsum_image(
    config: &Marmite,
    frontmatter: &Frontmatter,
    content_path: &Path,
    slug: &str,
    tags: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if frontmatter contains banner_image
    if frontmatter.get("banner_image").is_some() {
        return Ok(());
    }

    // Check if media/{slug}.banner.jpg exists
    let media_path = content_path.join(&config.media_path);
    let banner_file = media_path.join(format!("{slug}.banner.jpg"));

    if banner_file.exists() {
        return Ok(());
    }

    // Create media directory if it doesn't exist
    if !media_path.exists() {
        fs::create_dir_all(&media_path)?;
    }

    // Build URL: https://picsum.photos/seed/{slugified-site-name-post-slug-tags}/1200/300
    let slugified_site_name = slug::slugify(&config.name);
    let tags_string = tags.join("-");
    let slugified_tags = if tags_string.is_empty() {
        String::new()
    } else {
        format!("-{}", slug::slugify(&tags_string))
    };
    let seed = format!("{slugified_site_name}-{slug}{slugified_tags}");
    let url = format!("https://picsum.photos/seed/{seed}/1200/300");

    info!("Downloading banner image from: {url}");

    // Download the image
    // Note: ureq 3.0 doesn't support per-request timeout, but has default timeouts
    let response = ureq::get(&url).call()?;

    if response.status() == 200 {
        let mut file = fs::File::create(&banner_file)?;
        let mut body = response.into_body();
        std::io::copy(&mut body.as_reader(), &mut file)?;
        info!("Downloaded banner image to: {}", banner_file.display());
    } else {
        warn!("Failed to download image: HTTP {}", response.status());
    }

    Ok(())
}

#[cfg(test)]
#[path = "tests/image_provider.rs"]
mod tests;
