use super::*;
use std::fs;
use std::io::Read;
use tempfile::TempDir;

#[test]
fn test_default_bind_is_loopback_and_custom_bind_is_preserved() {
    use clap::Parser;

    let args = crate::cli::Cli::try_parse_from(["marmite", ".", "--serve"]).unwrap();
    assert_eq!(args.bind, "127.0.0.1:8000");
    let args =
        crate::cli::Cli::try_parse_from(["marmite", ".", "--serve", "--bind", "0.0.0.0:9000"])
            .unwrap();
    assert_eq!(args.bind, "0.0.0.0:9000");
}

#[test]
fn test_bind_fallback_preserves_interface() {
    for ip in ["127.0.0.1", "0.0.0.0", "::1"] {
        let ip: std::net::IpAddr = ip.parse().unwrap();
        let occupied = match std::net::TcpListener::bind((ip, 0)) {
            Ok(listener) => listener,
            Err(err) if ip.is_ipv6() && err.kind() == ErrorKind::AddrNotAvailable => continue,
            Err(err) => panic!("Failed to bind test listener: {err}"),
        };
        let requested = occupied.local_addr().unwrap();
        let server = bind_server(&requested.to_string()).unwrap();
        let actual = server.server_addr().to_ip().unwrap();
        assert_eq!(actual.ip(), requested.ip());
        assert_ne!(actual.port(), 0);
        assert_ne!(actual.port(), requested.port());
    }
}

#[test]
fn test_invalid_bind_does_not_fall_back_to_all_interfaces() {
    assert!(bind_server("invalid-address").is_err());
}

#[test]
fn test_render_not_found_with_file() {
    let temp_dir = TempDir::new().unwrap();
    let error_path = temp_dir.path().join("404.html");
    fs::write(&error_path, "Custom 404 page").unwrap();

    let response = render_not_found(&error_path, "missing.html", false, true);
    assert!(response.is_ok());
}

#[test]
fn test_render_not_found_without_file() {
    let temp_dir = TempDir::new().unwrap();
    let error_path = temp_dir.path().join("nonexistent_404.html");

    let response = render_not_found(&error_path, "missing.html", false, true);
    assert!(response.is_ok());
}

#[test]
fn test_render_not_found_with_file_content() {
    let temp_dir = TempDir::new().unwrap();
    let error_path = temp_dir.path().join("404.html");
    let content = "<html><body><h1>404 - Page Not Found</h1></body></html>";
    fs::write(&error_path, content).unwrap();

    let _response = render_not_found(&error_path, "missing.html", false, true).unwrap();
}

#[test]
fn test_render_not_found_fallback() {
    let temp_dir = TempDir::new().unwrap();
    let error_path = temp_dir.path().join("non_existent_404.html");

    let _response = render_not_found(&error_path, "missing.html", false, true).unwrap();
}

#[test]
fn test_render_not_found_injects_toolbar_with_live_reload() {
    let temp_dir = TempDir::new().unwrap();
    let error_path = temp_dir.path().join("404.html");
    let content = "<html><body><h1>Not Found</h1></body></html>";
    fs::write(&error_path, content).unwrap();

    let response = render_not_found(&error_path, "my-page.html", true, true).unwrap();
    let mut body = Vec::new();
    response.into_reader().read_to_end(&mut body).unwrap();
    let html = String::from_utf8(body).unwrap();
    assert!(html.contains("__marmite_404_slug__"));
    assert!(html.contains("\"my-page\""));
    assert!(html.contains(TOOLBAR_JS_PATH));
    assert!(html.contains(TOOLBAR_CSS_PATH));
}

#[test]
fn test_render_not_found_toolbar_always_injected() {
    let temp_dir = TempDir::new().unwrap();
    let error_path = temp_dir.path().join("404.html");
    let content = "<html><body><h1>Not Found</h1></body></html>";
    fs::write(&error_path, content).unwrap();

    let response = render_not_found(&error_path, "my-page.html", false, true).unwrap();
    let mut body = Vec::new();
    response.into_reader().read_to_end(&mut body).unwrap();
    let html = String::from_utf8(body).unwrap();
    assert!(html.contains("__marmite_404_slug__"));
    assert!(html.contains(TOOLBAR_JS_PATH));
    assert!(!html.contains(LIVE_RELOAD_SCRIPT_PATH));
}

#[test]
fn test_content_type_for_svg() {
    assert_eq!(content_type_for("image.svg"), Some("image/svg+xml"));
}

#[test]
fn test_content_type_for_common_types() {
    assert_eq!(
        content_type_for("index.html"),
        Some("text/html; charset=utf-8")
    );
    assert_eq!(
        content_type_for("style.css"),
        Some("text/css; charset=utf-8")
    );
    assert_eq!(
        content_type_for("app.js"),
        Some("text/javascript; charset=utf-8")
    );
    assert_eq!(
        content_type_for("data.json"),
        Some("application/json; charset=utf-8")
    );
    assert_eq!(
        content_type_for("sitemap.xml"),
        Some("application/xml; charset=utf-8")
    );
    assert_eq!(content_type_for("photo.png"), Some("image/png"));
    assert_eq!(content_type_for("photo.jpg"), Some("image/jpeg"));
    assert_eq!(content_type_for("photo.webp"), Some("image/webp"));
    assert_eq!(content_type_for("font.woff2"), Some("font/woff2"));
}

#[test]
fn test_content_type_for_unknown_extension() {
    assert_eq!(content_type_for("file.xyz"), None);
}

#[test]
fn test_content_type_for_nested_path() {
    assert_eq!(
        content_type_for("assets/icons/logo.svg"),
        Some("image/svg+xml")
    );
}
