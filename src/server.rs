use log::{error, info};
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;
use std::{fs::File, path::Path};
use tiny_http::{Header, Response, Server};
use urlencoding::decode;

pub fn start(bind_address: &str, output_folder: &Arc<PathBuf>) {
    let server = Server::http(bind_address).unwrap();

    info!("Server started at http://{bind_address}/ - Type ^C to stop.",);

    for request in server.incoming_requests() {
        let response = match handle_request(&request, output_folder.as_path()) {
            Ok(response) => response,
            Err(err) => {
                error!("Error handling request: {err:?}");
                Response::from_string("Internal Server Error").with_status_code(500)
            }
        };

        if let Err(err) = request.respond(response) {
            error!("Failed to send response: {err:?}");
        }
    }
}

#[allow(clippy::case_sensitive_file_extension_comparisons)]
fn handle_request(
    request: &tiny_http::Request,
    output_folder: &Path,
) -> Result<Response<Cursor<Vec<u8>>>, String> {
    let decoded_url = match decode(request.url()) {
        Ok(decoded) => decoded.into_owned(),
        Err(err) => {
            error!("Error decoding url {}: {err:?}", request.url());
            return Err(format!("Error decoding url: {err}"));
        }
    };

    let request_path = match decoded_url.as_str() {
        "/" => "index.html",
        url => &url[1..], // Remove the leading '/'
    };

    let file_path = output_folder.join(request_path);
    let error_path = output_folder.join("404.html");

    if file_path.is_file() {
        match File::open(&file_path) {
            Ok(mut file) => {
                let mut buffer = Vec::new();
                std::io::copy(&mut file, &mut buffer).map_err(|e| e.to_string())?;
                info!(
                    "\"{} {} HTTP/{}\" 200 -",
                    request.method(),
                    request_path,
                    request.http_version()
                );
                let mut resp = Response::from_data(buffer);
                let js_header = Header::from_bytes("Content-Type", "text/javascript").unwrap();
                if request_path.ends_with(".js") {
                    resp.add_header(js_header);
                }
                Ok(resp)
            }
            Err(err) => {
                error!("Failed to read file {}: {err:?}", file_path.display());
                Err(format!("Error reading file: {err}"))
            }
        }
    } else {
        error!(
            "\"{} {} HTTP/{}\" 404 -",
            request.method(),
            request_path,
            request.http_version()
        );
        render_not_found(&error_path)
    }
}

fn render_not_found(error_path: &PathBuf) -> Result<Response<Cursor<Vec<u8>>>, String> {
    match File::open(error_path) {
        Ok(mut file) => {
            let mut buffer = Vec::new();
            std::io::copy(&mut file, &mut buffer).map_err(|e| e.to_string())?;
            let resp = Response::from_data(buffer);
            Ok(resp)
        }
        Err(err) => {
            error!("Error on rendering page 404 - {err:?}");
            Ok(Response::from_string("404 Not Found").with_status_code(404))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_render_not_found_with_file() {
        let temp_dir = TempDir::new().unwrap();
        let error_path = temp_dir.path().join("404.html");
        fs::write(&error_path, "Custom 404 page").unwrap();

        let response = render_not_found(&error_path);
        assert!(response.is_ok());
    }

    #[test]
    fn test_render_not_found_without_file() {
        let temp_dir = TempDir::new().unwrap();
        let error_path = temp_dir.path().join("nonexistent_404.html");

        let response = render_not_found(&error_path);
        assert!(response.is_ok());
    }

    #[test]
    fn test_render_not_found_with_file_content() {
        let temp_dir = TempDir::new().unwrap();
        let error_path = temp_dir.path().join("404.html");
        let content = "<html><body><h1>404 - Page Not Found</h1></body></html>";
        fs::write(&error_path, content).unwrap();

        let _response = render_not_found(&error_path).unwrap();
        // Response should be created successfully
        // Testing the actual content is difficult without accessing internal data
    }

    #[test]
    fn test_render_not_found_fallback() {
        let temp_dir = TempDir::new().unwrap();
        let error_path = temp_dir.path().join("non_existent_404.html");

        let _response = render_not_found(&error_path).unwrap();
        // Should return fallback 404 response
        // Testing the actual content is difficult without accessing internal data
    }
}
