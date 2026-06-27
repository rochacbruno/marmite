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
fn test_embedded_templates_exist() {
    let template_names: Vec<_> = Templates::iter().collect();
    assert!(!template_names.is_empty());
    for name in &template_names {
        let template = Templates::get(name.as_ref());
        assert!(template.is_some(), "Template {name} should exist");
    }
}

#[test]
fn test_embedded_static_initialization() {
    let static_files = &*EMBEDDED_STATIC;
    assert!(!static_files.is_empty());
}

#[test]
fn test_embedded_agent_skills_initialization() {
    let skill_files = &*EMBEDDED_AGENT_SKILLS;
    assert!(!skill_files.is_empty());
}

#[test]
fn test_get_skill_content() {
    let content = get_skill_content();
    assert!(content.is_some());
    let content = content.unwrap();
    assert!(content.contains("marmite"));
}

#[test]
fn test_install_skills_to_agents() {
    let temp_dir = TempDir::new().unwrap();
    let target = temp_dir.path();

    install_skills_to_agents(target);

    assert!(target.join(".agents/skills/marmite/SKILL.md").exists());
}

#[test]
fn test_install_skills_to_claude() {
    let temp_dir = TempDir::new().unwrap();
    let target = temp_dir.path();

    install_skills_to_claude(target);

    assert!(target.join(".claude/skills/marmite/SKILL.md").exists());
}
