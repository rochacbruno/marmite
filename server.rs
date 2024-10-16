use tiny_http::{Server, Response};
use std::fs;
use std::path::Path;

pub fn serve_website(output_dir: &Path) -> Result<(), String> {
    let server = Server::http("0.0.0.0:8000").map_err(|e| e.to_string())?;
    println!("Serving website at http://0.0.0.0:8000");

    for request in server.incoming_requests() {
        let path = request.url().trim_start_matches('/');
        let file_path = if path.is_empty() {
            output_dir.join("index.html")
        } else {
            output_dir.join(path)
        };

        let response = if file_path.is_file() {
            let content = fs::read(&file_path).map_err(|e| e.to_string())?;
            Response::from_data(content)
        } else {
            Response::from_string("404 Not Found").with_status_code(404)
        };

        if let Err(e) = request.respond(response) {
            eprintln!("Failed to respond to request: {}", e);
        }
    }
    Ok(())
}
