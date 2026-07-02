use crate::atproto::client;
use crate::atproto::credentials;
use crate::content::Content;
use crate::re;
use crate::site::{get_content_folder, Data};
use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize, Default)]
struct StateEntry {
    content_hash: String,
    at_uri: String,
    last_published: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct PublishState {
    posts: HashMap<String, StateEntry>,
}

fn state_path(input_folder: &Path) -> PathBuf {
    input_folder.join(".marmite-atproto-state.json")
}

fn load_state(input_folder: &Path) -> PublishState {
    let path = state_path(input_folder);
    if !path.exists() {
        return PublishState::default();
    }
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_state(input_folder: &Path, state: &PublishState) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(state)?;
    fs::write(state_path(input_folder), json)?;
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    result.iter().fold(String::new(), |mut output, b| {
        let _ = write!(output, "{b:02x}");
        output
    })
}

fn strip_html(html: &str) -> String {
    let re = Regex::new(re::MATCH_HTML_TAGS).unwrap_or_else(|_| Regex::new(r"<[^>]*>").unwrap());
    re.replace_all(html, "").to_string()
}

/// Extract the rkey from an AT URI string (`at://did/.../rkey`).
fn rkey_from_at_uri(at_uri: &str) -> Option<String> {
    at_uri
        .parse::<shrike::syntax::AtUri>()
        .ok()
        .and_then(|u| u.rkey().map(std::string::ToString::to_string))
}

enum PostAction {
    Published,
    Updated,
    Skipped,
    NoSource,
}

fn build_record(
    post: &Content,
    marmite: &crate::config::Marmite,
    publication_uri: &str,
    publish_content: bool,
) -> serde_json::Value {
    let published_at = post
        .date
        .map_or_else(|| Utc::now().to_rfc3339(), |d| d.and_utc().to_rfc3339());

    let canonical_url = format!("{}/{}.html", marmite.url.trim_end_matches('/'), post.slug);

    let mut record = serde_json::json!({
        "$type": "site.standard.document",
        "title": post.title,
        "site": publication_uri,
        "path": format!("/{}", post.slug),
        "canonicalUrl": canonical_url,
        "publishedAt": published_at,
    });

    if let Some(desc) = &post.description {
        record["description"] = serde_json::Value::String(desc.clone());
    }
    if !post.tags.is_empty() {
        record["tags"] = serde_json::json!(post.tags);
    }
    if publish_content {
        let text: String = strip_html(&post.html).chars().take(10_000).collect();
        record["textContent"] = serde_json::Value::String(text);
    }

    record
}

struct PublishContext<'a> {
    pds_url: &'a str,
    session: &'a client::Session,
    marmite: &'a crate::config::Marmite,
    publication_uri: &'a str,
    publish_content: bool,
    force: bool,
    dry_run: bool,
}

fn process_post(
    post: &Content,
    state: &mut PublishState,
    ctx: &PublishContext<'_>,
) -> Result<PostAction, Box<dyn std::error::Error>> {
    let source_path = match &post.source_path {
        Some(p) => p.clone(),
        None => return Ok(PostAction::NoSource),
    };

    let raw_bytes = match fs::read(&source_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Warning: could not read {}: {e}", source_path.display());
            return Ok(PostAction::NoSource);
        }
    };
    let hash = sha256_hex(&raw_bytes);

    let existing = state.posts.get(&post.slug);
    if !ctx.force {
        if let Some(entry) = existing {
            if entry.content_hash == hash {
                return Ok(PostAction::Skipped);
            }
        }
    }

    let record = build_record(post, ctx.marmite, ctx.publication_uri, ctx.publish_content);

    if ctx.dry_run {
        let action = if existing.is_some() {
            "update"
        } else {
            "publish"
        };
        eprintln!("[dry-run] {action}: {}", post.slug);
        return Ok(PostAction::Skipped);
    }

    let (at_uri, action) = if let Some(entry) = existing {
        match rkey_from_at_uri(&entry.at_uri) {
            Some(rkey) => {
                let result = client::put_record(
                    ctx.pds_url,
                    &ctx.session.access_jwt,
                    &ctx.session.did,
                    "site.standard.document",
                    &rkey,
                    &record,
                )
                .map_err(|e| format!("Failed to update '{}': {e}", post.slug))?;
                (result.uri, PostAction::Updated)
            }
            None => {
                return Err(format!(
                    "Could not parse record key (rkey) from AT-URI '{}' for post '{}'",
                    entry.at_uri, post.slug
                )
                .into());
            }
        }
    } else {
        let result = client::create_record(
            ctx.pds_url,
            &ctx.session.access_jwt,
            &ctx.session.did,
            "site.standard.document",
            &record,
        )
        .map_err(|e| format!("Failed to publish '{}': {e}", post.slug))?;
        (result.uri, PostAction::Published)
    };

    state.posts.insert(
        post.slug.clone(),
        StateEntry {
            content_hash: hash,
            at_uri,
            last_published: Utc::now().to_rfc3339(),
        },
    );

    Ok(action)
}

fn collect_publishable_posts(input_folder: &Path, config_path: &Path) -> Vec<Content> {
    let site_data = Data::from_file(config_path);
    let content_dir = get_content_folder(&site_data.site, input_folder);
    let fragments = HashMap::new();

    WalkDir::new(&content_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            let name = e.path().file_name().and_then(|n| n.to_str()).unwrap_or("");
            let ext = e.path().extension().and_then(|x| x.to_str());
            e.path().is_file() && ext == Some("md") && !name.starts_with('_')
        })
        .filter_map(|entry| {
            Content::from_markdown(entry.path(), Some(&fragments), &site_data.site, None, None).ok()
        })
        .filter(|post| post.date.is_some() && post.stream.as_deref() != Some("draft"))
        .collect()
}

pub fn publish(
    input_folder: &Path,
    force: bool,
    dry_run: bool,
    config_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = input_folder.join(config_file);
    let site_data = Data::from_file(&config_path);
    let marmite = &site_data.site;

    // 1. Validate atproto config
    let atproto = marmite.atproto.as_ref().ok_or({
        concat!(
            "No atproto configuration found in marmite.yaml.\n",
            "Add the following to get started:\n\n",
            "  atproto:\n",
            "    handle: yourhandle.bsky.social\n\n",
            "Then run: marmite atproto auth"
        )
    })?;

    let handle = atproto
        .handle
        .as_deref()
        .ok_or("Add atproto.handle: yourhandle.bsky.social to marmite.yaml")?;

    // Check publication_uri
    let publication_uri = atproto
        .publication_uri
        .as_deref()
        .ok_or("No publication found. Add atproto.publication_uri to marmite.yaml.")?;

    // 2. Load credentials
    let cred = credentials::load(handle)
        .or_else(credentials::load_any)
        .ok_or_else(|| {
            format!(
                "No credentials found for '{handle}'.\n\
                 Run: marmite atproto auth\n\
                 (set ATPROTO_APP_PASSWORD env var first)"
            )
        })?;

    // 3. Authenticate
    let pds_url = env::var("ATPROTO_PDS_URL").unwrap_or(cred.pds_url.clone());
    let session = client::create_session(&pds_url, &cred.identifier, &cred.password)
        .map_err(|e| format!("Authentication failed: {e}"))?;

    // 4. Load state
    let mut state = load_state(input_folder);

    // 5. Collect publishable posts
    let posts = collect_publishable_posts(input_folder, &config_path);

    let mut published = 0usize;
    let mut updated = 0usize;
    let mut skipped = 0usize;

    let ctx = PublishContext {
        pds_url: &pds_url,
        session: &session,
        marmite,
        publication_uri,
        publish_content: atproto.publish_content,
        force,
        dry_run,
    };

    for post in &posts {
        match process_post(post, &mut state, &ctx)? {
            PostAction::Published => published += 1,
            PostAction::Updated => updated += 1,
            PostAction::Skipped | PostAction::NoSource => skipped += 1,
        }
    }

    if !dry_run {
        save_state(input_folder, &state)?;
    }

    if dry_run {
        eprintln!("[dry-run] done — no changes made");
    } else {
        eprintln!("Published {published}, updated {updated}, skipped {skipped} posts");
    }

    Ok(())
}
