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
