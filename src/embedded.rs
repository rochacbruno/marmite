use log::{error, info};
use rust_embed::Embed;
use std::fs;
use std::fs::File;
use std::io::{Error, Write};
use std::path::Path;
use std::sync::LazyLock;
use tera::Tera;

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

pub static EMBEDDED_SHORTCODES: LazyLock<Vec<(String, Vec<u8>)>> = LazyLock::new(|| {
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();

    for name in Shortcodes::iter() {
        let shortcode = Shortcodes::get(name.as_ref()).unwrap();
        let file_data = shortcode.data;
        files.push((name.clone().to_string(), file_data.clone().to_vec()));
    }

    files
});

pub static EMBEDDED_TERA: LazyLock<Tera> = LazyLock::new(|| {
    let mut tera = Tera::default();
    tera.autoescape_on(vec![]);
    for name in Templates::iter() {
        let template = Templates::get(name.as_ref()).unwrap();
        let template_str = std::str::from_utf8(template.data.as_ref()).unwrap();
        tera.add_raw_template(&name, template_str).unwrap();
    }
    tera
});

pub static EMBEDDED_STATIC: LazyLock<Vec<(String, Vec<u8>)>> = LazyLock::new(|| {
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();

    for name in Static::iter() {
        let static_file = Static::get(name.as_ref()).unwrap();
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

fn write_bytes_to_file(filename: &Path, data: &[u8]) -> Result<(), Error> {
    // Create or open the file for writing
    let mut file = File::create(filename)?;

    // Write the byte slice to the file
    file.write_all(data)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::TempDir;

    #[test]
    fn test_write_bytes_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");
        let test_data = b"Hello, World!";

        let result = write_bytes_to_file(&file_path, test_data);
        assert!(result.is_ok());

        let mut file = File::open(&file_path).unwrap();
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).unwrap();
        assert_eq!(contents, test_data);
    }

    #[test]
    fn test_generate_static() {
        let temp_dir = TempDir::new().unwrap();
        let static_folder = temp_dir.path().join("static");

        generate_static(&static_folder);

        assert!(static_folder.exists());
    }

    #[test]
    fn test_embedded_tera_initialization() {
        let tera = &*EMBEDDED_TERA;
        assert!(!tera.get_template_names().collect::<Vec<_>>().is_empty());
    }

    #[test]
    fn test_embedded_static_initialization() {
        let static_files = &*EMBEDDED_STATIC;
        assert!(!static_files.is_empty());
    }
}
