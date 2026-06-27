use log::{error, info};
use regex::Regex;
use rust_embed::Embed;
use std::fs;
use std::fs::File;
use std::io::{Error, Write};
use std::path::Path;
use std::sync::LazyLock;

#[derive(Embed, Debug)]
#[folder = "$CARGO_MANIFEST_DIR/example/static/"]
pub struct Static;

#[derive(Embed, Debug)]
#[folder = "$CARGO_MANIFEST_DIR/example/templates/"]
pub struct Templates;

#[derive(Embed, Debug)]
#[folder = "$CARGO_MANIFEST_DIR/example/theme_template/"]
pub struct ThemeTemplate;

#[derive(Embed, Debug)]
#[folder = "$CARGO_MANIFEST_DIR/example/shortcodes/"]
pub struct Shortcodes;

#[derive(Embed, Debug)]
#[folder = "$CARGO_MANIFEST_DIR/.agents/"]
pub struct AgentSkills;

pub static EMBEDDED_SHORTCODES: LazyLock<Vec<(String, Vec<u8>)>> = LazyLock::new(|| {
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();

    for name in Shortcodes::iter() {
        let shortcode = Shortcodes::get(name.as_ref())
            .expect("Failed to get embedded shortcode - this is a build-time error");
        let file_data = shortcode.data;
        files.push((name.clone().to_string(), file_data.clone().to_vec()));
    }

    files
});

/// Preprocess template content for Tera 2.0 compatibility.
/// - Strips `ignore missing` from `{% include %}` tags (not supported in Tera 2.0)
/// - Converts dot-notation numeric indexing (`item.0`) to bracket notation (`item[0]`)
/// - Converts positional test args to keyword args (e.g. `starting_with("x")` -> `starting_with(pat="x")`)
pub fn strip_ignore_missing(content: &str) -> String {
    let re = Regex::new(r#"(\{%-?\s*include\s+['"][^'"]+['"]\s+)ignore\s+missing\s*(-?%\})"#)
        .expect("Invalid include ignore missing pattern");
    let result = re.replace_all(content, "$1$2");
    let dot_index = Regex::new(r"(\b[a-zA-Z_]\w*)\.(\d+)\b").expect("Invalid dot-index pattern");
    let result = dot_index.replace_all(&result, "$1[$2]");
    let test_positional = Regex::new(
        r#"is (not\s+)?(starting_with|ending_with|containing|matching)\(("[^"]*"|'[^']*')\)"#,
    )
    .expect("Invalid test positional pattern");
    let result = test_positional.replace_all(&result, r#"is ${1}${2}(pat=${3})"#);
    // Convert chained property access in `is defined` / `is not defined` checks
    // to use optional chaining: `a.b.c is defined` -> `a?.b?.c is defined`
    let defined_check =
        Regex::new(r"(\b[a-zA-Z_]\w*(?:\.[a-zA-Z_]\w*)+)\s+is\s+(not\s+)?defined\b")
            .expect("Invalid defined check pattern");
    let result = defined_check.replace_all(&result, |caps: &regex::Captures| {
        let path = &caps[1];
        let not_part = caps.get(2).map_or("", |m| m.as_str());
        let parts: Vec<&str> = path.split('.').collect();
        if parts.len() > 2 {
            let optional_path = format!("{}?.{}", parts[0], parts[1..].join("?."));
            format!("{optional_path} is {not_part}defined")
        } else {
            caps[0].to_string()
        }
    });
    // Convert `{{ var.field | default(...)}}` to use optional chaining
    // for variables that might not be defined at render time
    let default_filter = Regex::new(r"\{\{-?\s*(\b[a-zA-Z_]\w*\.[a-zA-Z_]\w*)\s*\|\s*default\b")
        .expect("Invalid default filter pattern");
    default_filter
        .replace_all(&result, |caps: &regex::Captures| {
            let path = &caps[1];
            let full = &caps[0];
            let optional_path = path.replace('.', "?.");
            full.replace(path, &optional_path)
        })
        .to_string()
}

/// Collect template names referenced with `ignore missing` in a template string.
pub fn collect_ignore_missing_includes(content: &str) -> Vec<String> {
    let re = Regex::new(r#"\{%-?\s*include\s+['"]([^'"]+)['"]\s+ignore\s+missing\s*-?%\}"#)
        .expect("Invalid include ignore missing pattern");
    re.captures_iter(content)
        .map(|cap| cap[1].to_string())
        .collect()
}

pub static EMBEDDED_AGENT_SKILLS: LazyLock<Vec<(String, Vec<u8>)>> = LazyLock::new(|| {
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();

    for name in AgentSkills::iter() {
        let file = AgentSkills::get(name.as_ref())
            .expect("Failed to get embedded agent skill file - this is a build-time error");
        let file_data = file.data;
        files.push((name.clone().to_string(), file_data.clone().to_vec()));
    }

    files
});

pub static EMBEDDED_STATIC: LazyLock<Vec<(String, Vec<u8>)>> = LazyLock::new(|| {
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();

    for name in Static::iter() {
        let static_file = Static::get(name.as_ref())
            .expect("Failed to get embedded static file - this is a build-time error");
        let file_data = static_file.data;
        files.push((name.clone().to_string(), file_data.clone().to_vec()));
    }

    files
});

pub fn generate_static(static_folder: &Path) {
    if let Err(e) = fs::create_dir_all(static_folder) {
        error!("Unable to create static directory: {e:?}");
        return;
    }

    for (name, file_data) in EMBEDDED_STATIC.iter() {
        let file_path = static_folder.join(name); // static/foo.ext
                                                  // ensure the parent directory exists
        if let Some(parent) = file_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                error!("Unable to create directory: {e:?}");
                return;
            }
        }

        match write_bytes_to_file(file_path.as_path(), file_data) {
            Ok(()) => info!("Generated {}", &file_path.display()),
            Err(e) => error!("Error writing file: {e:?}"),
        }
    }
}

pub fn get_skill_content() -> Option<String> {
    AgentSkills::get("skills/marmite/SKILL.md").and_then(|file| {
        std::str::from_utf8(file.data.as_ref())
            .ok()
            .map(String::from)
    })
}

pub fn install_skills_to_agents(target_folder: &Path) {
    install_skills_to_dir(&target_folder.join(".agents"));
}

pub fn install_skills_to_claude(target_folder: &Path) {
    install_skills_to_dir(&target_folder.join(".claude"));
}

fn install_skills_to_dir(base_dir: &Path) {
    if let Err(e) = fs::create_dir_all(base_dir) {
        error!("Unable to create directory: {e:?}");
        return;
    }

    for (name, file_data) in EMBEDDED_AGENT_SKILLS.iter() {
        let file_path = base_dir.join(name);
        if let Some(parent) = file_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                error!("Unable to create directory: {e:?}");
                return;
            }
        }

        match write_bytes_to_file(file_path.as_path(), file_data) {
            Ok(()) => info!("Generated {}", &file_path.display()),
            Err(e) => error!("Error writing file: {e:?}"),
        }
    }
}

fn write_bytes_to_file(filename: &Path, data: &[u8]) -> Result<(), Error> {
    // Create or open the file for writing
    let mut file = File::create(filename)?;

    // Write the byte slice to the file
    file.write_all(data)?;

    Ok(())
}

#[cfg(test)]
#[path = "tests/embedded.rs"]
mod tests;
