use crate::atproto::client;
use crate::atproto::credentials::{self, Credential};
use crate::cli::Cli;
use crate::site::Data;
use std::env;

pub fn auth(input_folder: &std::path::Path, _args: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Read handle from marmite.yaml in input_folder
    let config_path = input_folder.join("marmite.yaml");
    if !config_path.exists() {
        return Err(format!("No marmite.yaml found in folder '{}'.\n\
                            Please run this command specifying the input folder that contains marmite.yaml.",
                            input_folder.display()).into());
    }

    let site_data = Data::from_file(&config_path);
    let handle = site_data
        .site
        .atproto
        .as_ref()
        .and_then(|a| a.handle.as_ref())
        .ok_or("atproto.handle must be configured in marmite.yaml to authenticate.")?;

    // 2. Read password from environment
    let password = env::var("ATPROTO_APP_PASSWORD").map_err(|_| {
        concat!(
            "ATPROTO_APP_PASSWORD env var is required.\n",
            "Create an app password at: https://bsky.app/settings/app-passwords\n\n",
            "Then set:\n",
            "  export ATPROTO_APP_PASSWORD=xxxx-xxxx-xxxx-xxxx"
        )
    })?;

    let pds_url = env::var("ATPROTO_PDS_URL").unwrap_or_else(|_| {
        match client::resolve_pds_endpoint(handle) {
            Ok(resolved) => resolved,
            Err(e) => {
                log::warn!("Could not resolve PDS endpoint for handle '{handle}': {e}. Falling back to default.");
                "https://bsky.social".to_string()
            }
        }
    });

    // 3. Authenticate
    let session = client::create_session(&pds_url, handle, &password)
        .map_err(|e| format!("Authentication failed: {e}\nCheck your handle and app password."))?;

    // 4. Save credentials
    credentials::save(&Credential {
        pds_url,
        identifier: handle.clone(),
        password,
    })?;

    eprintln!(
        "Authenticated as @{}\nCredentials saved to {}",
        session.handle,
        credentials::credentials_path().display()
    );

    // 5. Check for publication_uri
    if site_data
        .site
        .atproto
        .as_ref()
        .and_then(|a| a.publication_uri.as_ref())
        .is_some()
    {
        eprintln!("\nPublication already configured in marmite.yaml.");
        return Ok(());
    }

    eprintln!(
        "\nNo publication_uri found in marmite.yaml.\n\
         Please register your publication externally (e.g. at https://standard.site or via Sequoia CLI)\n\
         to obtain your publication AT-URI, then configure it in marmite.yaml:\n\n\
         atproto:\n\
           handle: {handle}\n\
           publication_uri: at://did:plc:.../site.standard.publication/...\n"
    );

    Ok(())
}
