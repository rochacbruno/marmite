use super::*;

fn create_test_cli(overrides: impl FnOnce(&mut cli::Cli)) -> cli::Cli {
    let mut args = cli::Cli {
        input_folder: PathBuf::from("test"),
        output_folder: None,
        verbose: 0,
        watch: false,
        serve: false,
        bind: "0.0.0.0:8000".to_string(),
        config: "marmite.yaml".to_string(),
        debug: false,
        init_templates: false,
        start_theme: None,
        set_theme: None,
        generate_config: false,
        init_site: false,
        force: false,
        shortcodes: false,
        show_urls: false,
        create: cli::Create {
            new: None,
            edit: false,
            page: false,
            tags: None,
        },
        configuration: cli::Configuration {
            name: None,
            tagline: None,
            url: None,
            https: None,
            footer: None,
            language: None,
            pagination: None,
            enable_search: None,
            enable_related_content: None,
            content_path: None,
            templates_path: None,
            static_path: None,
            media_path: None,
            default_date_format: None,
            colorscheme: None,
            toc: None,
            json_feed: None,
            show_next_prev_links: None,
            publish_md: None,
            source_repository: None,
            image_provider: None,
            theme: None,
            build_sitemap: None,
            publish_urls_json: None,
            enable_shortcodes: None,
            shortcode_pattern: None,
            skip_image_resize: None,
        },
    };
    overrides(&mut args);
    args
}

#[test]
fn test_determine_verbosity_default() {
    let args = create_test_cli(|_| {});
    assert_eq!(determine_verbosity(&args), 0);
}

#[test]
fn test_determine_verbosity_watch_sets_verbose() {
    let args = create_test_cli(|args| {
        args.watch = true;
    });
    assert_eq!(determine_verbosity(&args), 1);

    let args = create_test_cli(|args| {
        args.serve = true;
    });
    assert_eq!(determine_verbosity(&args), 1);

    let args = create_test_cli(|args| {
        args.init_templates = true;
    });
    assert_eq!(determine_verbosity(&args), 1);
}

#[test]
fn test_determine_verbosity_debug_flag() {
    let args = create_test_cli(|args| {
        args.debug = true;
    });
    assert_eq!(determine_verbosity(&args), 2);
}

#[test]
fn test_determine_verbosity_explicit_verbose() {
    let args = create_test_cli(|args| {
        args.verbose = 3;
    });
    assert_eq!(determine_verbosity(&args), 3);
}

#[test]
fn test_get_config_path_relative() {
    let input_folder = PathBuf::from("/home/user/project");
    let config = "marmite.yaml";
    let result = get_config_path(&input_folder, config);
    assert_eq!(result, PathBuf::from("/home/user/project/marmite.yaml"));
}

#[test]
fn test_get_config_path_absolute() {
    let input_folder = PathBuf::from("/home/user/project");
    let config = "/etc/marmite.yaml";
    let result = get_config_path(&input_folder, config);
    assert_eq!(result, PathBuf::from("/etc/marmite.yaml"));
}

#[test]
fn test_get_config_path_dot_relative() {
    let input_folder = PathBuf::from("/home/user/project");
    let config = "./config/marmite.yaml";
    let result = get_config_path(&input_folder, config);
    assert_eq!(result, PathBuf::from("./config/marmite.yaml"));
}

#[test]
fn test_run_cli_nonexistent_folder() {
    let args = create_test_cli(|args| {
        args.input_folder = PathBuf::from("/nonexistent/folder");
    });

    let result = run_cli(args);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Input folder does not exist"));
}
