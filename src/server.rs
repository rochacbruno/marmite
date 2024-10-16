use std::fs;
use std::path::Path;
use tiny_http::{Response, Server};

pub fn serve_website(output_dir: &Path) -> Result<(), String> {
    let server = Server::http("localhost:8000").map_err(|e| e.to_string())?;
    println!("Serving website at http://localhost:8000");

    for request in server.incoming_requests() {
        let mut path = request.url().trim_start_matches('/');

        // Serve index.html for the root path
        if path.is_empty() {
            path = "index.html";
        }

        let file_path = output_dir.join(path);

        let response = if file_path.is_file() {
            let content = fs::read(&file_path).map_err(|e| e.to_string())?;
            let mime_type = match file_path.extension().and_then(|e| e.to_str()) {
                Some("html") => "text/html; charset=UTF-8",
                Some("css") => "text/css; charset=UTF-8",
                Some("js") => "application/javascript; charset=UTF-8",
                Some("png") => "image/png",
                Some("jpg") | Some("jpeg") => "image/jpeg",
                Some("gif") => "image/gif",
                Some("svg") => "image/svg+xml",
                _ => "application/octet-stream",
            };
            Response::from_data(content).with_header(tiny_http::Header {
                field: "Content-Type".parse().unwrap(),
                value: mime_type.parse().unwrap(),
            })
        } else {
            Response::from_string("404 Not Found").with_status_code(404)
        };

        if let Err(e) = request.respond(response) {
            eprintln!("Failed to respond to request: {}", e);
        }
    }
    Ok(())
}