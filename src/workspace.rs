use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{fs, process};

use crate::cli::Cli;
use crate::config::Marmite;
use crate::site::{self, Data};

pub const WORKSPACE_CONFIG_FILENAME: &str = "marmite-workspace.yaml";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorkspaceSiteEntry {
    pub name: String,
    #[serde(default)]
    pub output_path: Option<String>,
}

impl WorkspaceSiteEntry {
    pub fn resolved_output_path(&self) -> &str {
        self.output_path.as_deref().unwrap_or(&self.name)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorkspaceConfig {
    pub sites: Vec<WorkspaceSiteEntry>,
    #[serde(default)]
    pub default_site: Option<String>,
    #[serde(default)]
    pub redirect: bool,
    #[serde(default)]
    pub defaults: Option<Marmite>,
    #[serde(default = "default_separator")]
    pub separator: String,
}

fn default_separator() -> String {
    "::".to_string()
}

impl WorkspaceConfig {
    pub fn resolved_default_site(&self) -> Option<&str> {
        self.default_site
            .as_deref()
            .or(self.sites.first().map(|s| s.name.as_str()))
    }
}

#[derive(Debug, Clone)]
pub struct SiteData {
    #[allow(dead_code)]
    pub name: String,
    pub output_path: String,
    pub data: Data,
}

#[derive(Debug, Clone, Default)]
pub struct CrossSiteData {
    pub sites: HashMap<String, SiteData>,
    pub separator: String,
}

pub fn detect_workspace(input_folder: &Path) -> Option<PathBuf> {
    let config_path = input_folder.join(WORKSPACE_CONFIG_FILENAME);
    if config_path.exists() {
        Some(config_path)
    } else {
        None
    }
}

pub fn load_workspace_config(
    config_path: &Path,
) -> Result<WorkspaceConfig, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(config_path)?;
    let config: WorkspaceConfig = serde_yaml::from_str(&content)?;
    if config.sites.is_empty() {
        return Err("Workspace config must define at least one site in 'sites'".into());
    }
    Ok(config)
}

pub(crate) fn deep_merge_yaml(
    base: serde_yaml::Value,
    overlay: serde_yaml::Value,
) -> serde_yaml::Value {
    match (base, overlay) {
        (serde_yaml::Value::Mapping(mut base_map), serde_yaml::Value::Mapping(overlay_map)) => {
            for (key, overlay_val) in overlay_map {
                let merged = if let Some(base_val) = base_map.remove(&key) {
                    deep_merge_yaml(base_val, overlay_val)
                } else {
                    overlay_val
                };
                base_map.insert(key, merged);
            }
            serde_yaml::Value::Mapping(base_map)
        }
        (_, overlay) => overlay,
    }
}

pub fn merge_site_config(
    workspace_defaults: Option<&Marmite>,
    site_config_path: &Path,
    cli_args: &Arc<Cli>,
) -> Marmite {
    let mut base = workspace_defaults.cloned().unwrap_or_else(Marmite::new);
    base.override_from_cli_args(cli_args);

    let site_str = fs::read_to_string(site_config_path).unwrap_or_default();
    if site_str.is_empty() {
        return base;
    }

    let site_value: serde_yaml::Value = match serde_yaml::from_str(&site_str) {
        Ok(v) => v,
        Err(e) => {
            warn!(
                "Failed to parse site config '{}': {e}, using workspace defaults",
                site_config_path.display()
            );
            return base;
        }
    };

    let Ok(base_value) = serde_yaml::to_value(&base) else {
        return base;
    };

    let merged_value = deep_merge_yaml(base_value, site_value);
    serde_yaml::from_value(merged_value).unwrap_or(base)
}

fn preprocess_all_sites(
    ws_config: &WorkspaceConfig,
    workspace_root: &Path,
    cli_args: &Arc<Cli>,
) -> Result<CrossSiteData, Box<dyn std::error::Error>> {
    let mut cross_site = CrossSiteData {
        sites: HashMap::new(),
        separator: ws_config.separator.clone(),
    };

    let default_site_name = ws_config.resolved_default_site();

    for site_entry in &ws_config.sites {
        let site_input = workspace_root.join(&site_entry.name);
        if !site_input.exists() {
            return Err(format!("Site directory does not exist: {}", site_input.display()).into());
        }

        let site_config_path = site_input.join(&cli_args.config);
        let merged_config =
            merge_site_config(ws_config.defaults.as_ref(), &site_config_path, cli_args);

        let config_str = serde_yaml::to_string(&merged_config).unwrap_or_default();
        let mut site_data = Data::new(&config_str, &site_config_path);
        let content_folder = site::get_content_folder(&site_data.site, &site_input);
        let fragments = site::collect_content_fragments(&content_folder);
        let folder_defaults = site::load_folder_frontmatter(&content_folder);
        site::collect_content(
            &content_folder,
            &mut site_data,
            &fragments,
            None,
            &folder_defaults,
        );
        site_data.sort_all();
        site_data.collect_all_urls();

        let is_default = default_site_name == Some(site_entry.name.as_str());
        let output_path = if is_default && !ws_config.redirect {
            String::new()
        } else {
            site_entry.resolved_output_path().to_string()
        };

        cross_site.sites.insert(
            site_entry.name.clone(),
            SiteData {
                name: site_entry.name.clone(),
                output_path,
                data: site_data,
            },
        );
    }

    Ok(cross_site)
}

pub fn run_workspace(
    ws_config_path: &Path,
    workspace_root: &Path,
    output_folder_override: Option<PathBuf>,
    watch: bool,
    serve: bool,
    bind_address: &str,
    cli_args: &Arc<Cli>,
) -> Result<(), Box<dyn std::error::Error>> {
    let ws_config = load_workspace_config(ws_config_path)?;
    let output_root = output_folder_override.unwrap_or_else(|| workspace_root.join("site"));

    info!("Workspace mode: {} site(s) detected", ws_config.sites.len());

    let cross_site_data = preprocess_all_sites(&ws_config, workspace_root, cli_args)?;

    let default_site_name = ws_config.resolved_default_site().map(String::from);

    for site_entry in &ws_config.sites {
        let site_input = workspace_root.join(&site_entry.name);
        let site_config_path = site_input.join(&cli_args.config);
        let mut merged_config =
            merge_site_config(ws_config.defaults.as_ref(), &site_config_path, cli_args);

        let is_default = default_site_name.as_deref() == Some(&site_entry.name);
        let site_output = if is_default && !ws_config.redirect {
            output_root.clone()
        } else {
            output_root.join(site_entry.resolved_output_path())
        };

        if is_default && !ws_config.redirect {
            if let Some(url) = resolve_site_url(&merged_config.url, "") {
                merged_config.url = url;
            }
        } else {
            let path = site_entry.resolved_output_path();
            if let Some(url) = resolve_site_url(&merged_config.url, path) {
                merged_config.url = url;
            }
        }

        let path_prefix = if is_default && !ws_config.redirect {
            ""
        } else {
            site_entry.resolved_output_path()
        };

        info!(
            "Building site '{}' -> {}",
            site_entry.name,
            site_output.display()
        );

        site::build_site_with_config(
            &merged_config,
            &site_input,
            &site_output,
            cli_args,
            Some(&cross_site_data),
            path_prefix,
        )?;
    }

    if ws_config.redirect {
        if let Some(default_name) = &default_site_name {
            let default_path = ws_config
                .sites
                .iter()
                .find(|s| s.name == *default_name)
                .map_or(
                    default_name.as_str(),
                    WorkspaceSiteEntry::resolved_output_path,
                );
            let redirect_html = site::generate_redirect_html(&format!("/{default_path}/"));
            if let Err(e) = fs::create_dir_all(&output_root) {
                error!(
                    "Failed to create output directory: {}",
                    output_root.display()
                );
                return Err(e.into());
            }
            fs::write(output_root.join("index.html"), redirect_html)?;
            info!("Generated root redirect to /{default_path}/");
        }
    }

    write_sites_json(&ws_config, &output_root)?;
    write_workspace_build_info(&ws_config, &cross_site_data, &output_root)?;
    write_workspace_urls_json(&cross_site_data, &output_root)?;

    info!("Workspace generated at: {}/", output_root.display());

    if watch || serve {
        handle_workspace_watch_serve(
            &ws_config,
            ws_config_path,
            workspace_root,
            &output_root,
            watch,
            serve,
            bind_address,
            cli_args,
        );
    }

    Ok(())
}

fn resolve_site_url(base_url: &str, subpath: &str) -> Option<String> {
    if base_url.is_empty() {
        return None;
    }
    let base = base_url.trim_end_matches('/');
    if subpath.is_empty() {
        Some(base.to_string())
    } else {
        Some(format!("{base}/{subpath}"))
    }
}

fn write_sites_json(
    ws_config: &WorkspaceConfig,
    output_root: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let sites_info: Vec<serde_json::Value> = ws_config
        .sites
        .iter()
        .map(|s| {
            serde_json::json!({
                "name": s.name,
                "path": format!("/{}/", s.resolved_output_path()),
            })
        })
        .collect();

    let json = serde_json::to_string_pretty(&sites_info)?;
    fs::write(output_root.join("sites.json"), json)?;
    info!("Generated sites.json");
    Ok(())
}

fn write_workspace_build_info(
    ws_config: &WorkspaceConfig,
    cross_site_data: &CrossSiteData,
    output_root: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut total_posts = 0;
    let mut total_pages = 0;
    let mut sites = Vec::new();

    for site_entry in &ws_config.sites {
        if let Some(sd) = cross_site_data.sites.get(&site_entry.name) {
            let post_count = sd.data.posts.len();
            let page_count = sd.data.pages.len();
            total_posts += post_count;
            total_pages += page_count;
            sites.push(serde_json::json!({
                "name": site_entry.name,
                "output_path": sd.output_path,
                "posts": post_count,
                "pages": page_count,
            }));
        }
    }

    let build_info = serde_json::json!({
        "workspace": true,
        "marmite_version": env!("CARGO_PKG_VERSION"),
        "generated_at": chrono::Local::now().to_string(),
        "sites": sites,
        "total_posts": total_posts,
        "total_pages": total_pages,
    });

    let json = serde_json::to_string_pretty(&build_info)?;
    fs::write(output_root.join("marmite-workspace.json"), json)?;
    info!("Generated marmite-workspace.json");
    Ok(())
}

fn write_workspace_urls_json(
    cross_site_data: &CrossSiteData,
    output_root: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged = serde_json::Map::new();

    for sd in cross_site_data.sites.values() {
        let site_json = site::create_urls_json(&sd.data, &sd.output_path);
        if let serde_json::Value::Object(map) = site_json {
            for (category, urls) in map {
                let entry = merged
                    .entry(category)
                    .or_insert_with(|| serde_json::Value::Array(Vec::new()));
                if let (serde_json::Value::Array(existing), serde_json::Value::Array(new)) =
                    (entry, urls)
                {
                    existing.extend(new);
                }
            }
        }
    }

    let json = serde_json::to_string_pretty(&serde_json::Value::Object(merged))?;
    fs::write(output_root.join("urls-workspace.json"), json)?;
    info!("Generated urls-workspace.json");
    Ok(())
}

pub fn show_urls_workspace(
    ws_config: &WorkspaceConfig,
    workspace_root: &Path,
    cli_args: &Arc<Cli>,
) {
    let cross_site = match preprocess_all_sites(ws_config, workspace_root, cli_args) {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to preprocess workspace sites: {e}");
            process::exit(1);
        }
    };

    let mut all_urls = serde_json::Map::new();
    for site_entry in &ws_config.sites {
        if let Some(site_data) = cross_site.sites.get(&site_entry.name) {
            let urls = site_data.data.generated_urls.get_all_urls();
            let prefix = &site_data.output_path;
            let prefixed: Vec<String> = urls
                .iter()
                .map(|u| {
                    if prefix.is_empty() {
                        if u.starts_with('/') {
                            u.clone()
                        } else {
                            format!("/{u}")
                        }
                    } else if u.starts_with('/') {
                        format!("/{prefix}{u}")
                    } else {
                        format!("/{prefix}/{u}")
                    }
                })
                .collect();
            all_urls.insert(
                site_entry.name.clone(),
                serde_json::Value::Array(
                    prefixed
                        .into_iter()
                        .map(serde_json::Value::String)
                        .collect(),
                ),
            );
        }
    }

    match serde_json::to_string_pretty(&serde_json::Value::Object(all_urls)) {
        Ok(json_string) => println!("{json_string}"),
        Err(e) => error!("Failed to serialize workspace URLs to JSON: {e}"),
    }
}

pub fn show_shortcodes_workspace(
    ws_config: &WorkspaceConfig,
    workspace_root: &Path,
    cli_args: &Arc<Cli>,
) {
    for site_entry in &ws_config.sites {
        let site_input = workspace_root.join(&site_entry.name);
        let site_config_path = site_input.join(&cli_args.config);
        let merged_config =
            merge_site_config(ws_config.defaults.as_ref(), &site_config_path, cli_args);

        println!("\n=== Site: {} ===", site_entry.name);

        let mut processor =
            crate::shortcodes::ShortcodeProcessor::new(merged_config.shortcode_pattern.as_deref());
        if let Err(e) = processor.collect_shortcodes(&site_input) {
            eprintln!("Error collecting shortcodes for '{}': {e}", site_entry.name);
            continue;
        }

        println!("Enabled: {}", merged_config.enable_shortcodes);
        if processor.shortcodes.is_empty() {
            println!("No shortcodes available.");
        } else {
            for (name, description) in processor.list_shortcodes_with_descriptions() {
                match description {
                    Some(desc) => println!("  - {name}: {desc}"),
                    None => println!("  - {name}"),
                }
            }
        }
    }
}

pub fn resolve_cross_site_refs(html: &str, cross_site_data: &CrossSiteData) -> String {
    let sep = &cross_site_data.separator;
    let escaped_sep = regex::escape(sep);

    let pattern = format!(r#"((?:href|src)=["'])(\w+){escaped_sep}([^"']+)(["'])"#);
    let Ok(re) = regex::Regex::new(&pattern) else {
        return html.to_string();
    };

    re.replace_all(html, |caps: &regex::Captures| {
        let prefix = &caps[1];
        let site_name = &caps[2];
        let path = &caps[3];
        let suffix = &caps[4];

        if let Some(site_data) = cross_site_data.sites.get(site_name) {
            if site_data.output_path.is_empty() {
                format!("{prefix}/{path}{suffix}")
            } else {
                format!("{prefix}/{}/{path}{suffix}", site_data.output_path)
            }
        } else {
            caps[0].to_string()
        }
    })
    .to_string()
}

fn make_rebuild_fn(
    ws_config: &WorkspaceConfig,
    workspace_root: &Path,
    output_root: &Path,
    cli_args: &Arc<Cli>,
    live_reload: Option<crate::server::LiveReload>,
) -> Arc<std::sync::Mutex<impl Fn()>> {
    let ws_config_clone = ws_config.clone();
    let workspace_root_owned = workspace_root.to_path_buf();
    let output_root_owned = output_root.to_path_buf();
    let cli_clone = Arc::clone(cli_args);

    Arc::new(std::sync::Mutex::new(move || {
        info!("Change detected. Rebuilding workspace...");
        let cross_site_data =
            match preprocess_all_sites(&ws_config_clone, &workspace_root_owned, &cli_clone) {
                Ok(data) => data,
                Err(e) => {
                    error!("Failed to preprocess workspace: {e}");
                    return;
                }
            };

        let default_site_name = ws_config_clone.resolved_default_site().map(String::from);

        for site_entry in &ws_config_clone.sites {
            let site_input = workspace_root_owned.join(&site_entry.name);
            let site_config_path = site_input.join(&cli_clone.config);
            let mut merged_config = merge_site_config(
                ws_config_clone.defaults.as_ref(),
                &site_config_path,
                &cli_clone,
            );

            let is_default = default_site_name.as_deref() == Some(&site_entry.name);
            let site_output = if is_default && !ws_config_clone.redirect {
                output_root_owned.clone()
            } else {
                output_root_owned.join(site_entry.resolved_output_path())
            };

            if is_default && !ws_config_clone.redirect {
                if let Some(url) = resolve_site_url(&merged_config.url, "") {
                    merged_config.url = url;
                }
            } else {
                let path = site_entry.resolved_output_path();
                if let Some(url) = resolve_site_url(&merged_config.url, path) {
                    merged_config.url = url;
                }
            }

            let path_prefix = if is_default && !ws_config_clone.redirect {
                ""
            } else {
                site_entry.resolved_output_path()
            };

            if let Err(e) = site::build_site_with_config(
                &merged_config,
                &site_input,
                &site_output,
                &cli_clone,
                Some(&cross_site_data),
                path_prefix,
            ) {
                error!("Failed to rebuild site '{}': {e}", site_entry.name);
            }
        }

        if ws_config_clone.redirect {
            if let Some(default_name) = &default_site_name {
                let default_path = ws_config_clone
                    .sites
                    .iter()
                    .find(|s| s.name == *default_name)
                    .map_or(
                        default_name.as_str(),
                        WorkspaceSiteEntry::resolved_output_path,
                    );
                let redirect_html = site::generate_redirect_html(&format!("/{default_path}/"));
                let _ = fs::write(output_root_owned.join("index.html"), redirect_html);
            }
        }

        if let Some(lr) = &live_reload {
            lr.notify_reload();
        }
    }))
}

#[allow(clippy::too_many_arguments)]
fn handle_workspace_watch_serve(
    ws_config: &WorkspaceConfig,
    ws_config_path: &Path,
    workspace_root: &Path,
    output_root: &Path,
    watch: bool,
    serve: bool,
    bind_address: &str,
    cli_args: &Arc<Cli>,
) {
    let live_reload = if watch && serve {
        Some(crate::server::LiveReload::new())
    } else {
        None
    };

    if watch {
        let mut hotwatch = match hotwatch::Hotwatch::new() {
            Ok(hw) => hw,
            Err(e) => {
                error!("Failed to initialize hotwatch: {e}");
                return;
            }
        };

        let rebuild = make_rebuild_fn(
            ws_config,
            workspace_root,
            output_root,
            cli_args,
            live_reload.clone(),
        );

        for site_entry in &ws_config.sites {
            let site_input = workspace_root.join(&site_entry.name);
            let out_folder = output_root.to_path_buf();
            let rebuild_clone = Arc::clone(&rebuild);
            let watch_result = hotwatch.watch(site_input.clone(), move |event: hotwatch::Event| {
                use hotwatch::EventKind;
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                        let in_output = event.paths.iter().any(|p| {
                            p.starts_with(
                                fs::canonicalize(&out_folder)
                                    .unwrap_or_else(|_| out_folder.clone()),
                            )
                        });
                        if !in_output {
                            if let Ok(rebuild_fn) = rebuild_clone.lock() {
                                rebuild_fn();
                            }
                        }
                    }
                    _ => {}
                }
            });
            if let Err(e) = watch_result {
                error!(
                    "Failed to watch site directory '{}': {e}",
                    site_input.display()
                );
            }
        }

        let ws_config_file = ws_config_path.to_path_buf();
        let rebuild_clone = Arc::clone(&rebuild);
        let watch_result = hotwatch.watch(ws_config_file.clone(), move |event: hotwatch::Event| {
            use hotwatch::EventKind;
            if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
                if let Ok(rebuild_fn) = rebuild_clone.lock() {
                    rebuild_fn();
                }
            }
        });
        if let Err(e) = watch_result {
            error!(
                "Failed to watch workspace config '{}': {e}",
                ws_config_file.display()
            );
        }

        info!(
            "Watching for changes in workspace: {}",
            workspace_root.display()
        );

        if serve {
            info!("Starting built-in HTTP server...");
            let ctx = crate::server::ServerContext {
                output_folder: Arc::new(output_root.to_path_buf()),
                input_folder: Arc::new(workspace_root.to_path_buf()),
                config_path: Arc::new(ws_config_path.to_path_buf()),
                enable_toolbar: true,
            };
            crate::server::start(bind_address, &ctx, live_reload.as_ref());
        } else {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    } else if serve {
        info!("Starting built-in HTTP server...");
        let ctx = crate::server::ServerContext {
            output_folder: Arc::new(output_root.to_path_buf()),
            input_folder: Arc::new(workspace_root.to_path_buf()),
            config_path: Arc::new(ws_config_path.to_path_buf()),
            enable_toolbar: true,
        };
        crate::server::start(bind_address, &ctx, None);
    }
}

#[cfg(test)]
#[path = "tests/workspace.rs"]
mod tests;
