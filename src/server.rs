use log::{error, info};
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;
use std::{fs::File, path::Path};
use tiny_http::{Response, Server};

pub fn start(bind_address: &str, output_folder: &Arc<PathBuf>) {
    let server = Server::http(bind_address).unwrap();

    info!(
        "Server started at http://{}/ - Type ^C to stop.",
        bind_address
    );

    for request in server.incoming_requests() {
        let response = match handle_request(&request, output_folder) {
            Ok(response) => response,
            Err(err) => {
                error!("Error handling request: {}", err);
                Response::from_string("Internal Server Error").with_status_code(500)
            }
        };

        if let Err(err) = request.respond(response) {
            error!("Failed to send response: {}", err);
        }
    }
}

fn handle_request(
    request: &tiny_http::Request,
    output_folder: &Path,
) -> Result<Response<Cursor<Vec<u8>>>, String> {
    let request_path = match request.url() {
        "/" => "index.html",
        url => &url[1..], // Remove the leading '/'
    };

    let file_path = output_folder.join(request_path);

    if file_path.is_file() {
        match File::open(&file_path) {
            Ok(mut file) => {
                let mut buffer = Vec::new();
                std::io::copy(&mut file, &mut buffer).map_err(|e| e.to_string())?;
                Ok(Response::from_data(buffer))
            }
            Err(err) => {
                error!("Failed to read file {}: {}", file_path.display(), err);
                Err(format!("Error reading file: {err}"))
            }
        }
    } else {
        error!("File not found: {}", file_path.display());
        Ok(Response::from_string("404 Not Found").with_status_code(404))
    }
}
