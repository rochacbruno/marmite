use crate::atproto::client;
use crate::atproto::credentials::{self, Credential};
use crate::cli::Cli;
use crate::site::Data;
use std::env;

pub fn auth(input_folder: &std::path::Path, args: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Read handle from configuration file in input_folder
    let config_path = if args.config.starts_with('.') || args.config.starts_with('/') {
        std::path::PathBuf::from(&args.config)
    } else {
        input_folder.join(&args.config)
    };
    if !config_path.exists() {
        return Err(format!("No configuration file found at '{}'.\n\
                            Please run this command specifying the input folder that contains the configuration file.",
                            config_path.display()).into());
    }

    let config_filename = config_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("marmite.yaml");

    let site_data = Data::from_file(&config_path);
    let handle = site_data
        .site
        .atproto
        .as_ref()
        .and_then(|a| a.handle.as_ref())
        .ok_or(format!(
            "atproto.handle must be configured in {config_filename} to authenticate."
        ))?;

    // 2. Read password from environment
    let password = env::var("ATPROTO_APP_PASSWORD").map_err(|_| {
        concat!(
            "ATPROTO_APP_PASSWORD env var is required.\n",
            "Create an app password at: https://bsky.app/settings/app-passwords\n\n",
            "Then set:\n",
            "  export ATPROTO_APP_PASSWORD=xxxx-xxxx-xxxx-xxxx"
        )
    })?;

    // 3. Resolve PDS endpoint (return error instead of falling back to hardcoded default)
    let pds_url = if let Ok(val) = env::var("ATPROTO_PDS_URL") {
        val
    } else {
        client::resolve_pds_endpoint(handle).map_err(|e| {
            format!(
                "Could not resolve PDS endpoint for handle '{handle}': {e}.\n\
                 If you are self-hosting or the resolution failed, you can set the ATPROTO_PDS_URL environment variable to override."
            )
        })?
    };

    // 4. Authenticate
    let session = client::create_session(&pds_url, handle, &password)
        .map_err(|e| format!("Authentication failed: {e}\nCheck your handle and app password."))?;

    // 5. Save credentials
    credentials::save(&Credential {
        pds_url,
        identifier: handle.clone(),
        password,
    })?;

    log::info!(
        "Authenticated as @{}\nCredentials saved to {}",
        session.handle,
        credentials::credentials_path().display()
    );

    // 6. Check for publication_uri
    if site_data
        .site
        .atproto
        .as_ref()
        .and_then(|a| a.publication_uri.as_ref())
        .is_some()
    {
        log::info!("\nPublication already configured in {config_filename}.");
        return Ok(());
    }

    log::info!(
        "\nNo publication_uri found in {config_filename}.\n\
         Please register your publication externally (e.g. at https://standard.site or via Sequoia CLI)\n\
         to obtain your publication AT-URI, then configure it in {config_filename}:\n\n\
         atproto:\n\
           handle: {handle}\n\
           publication_uri: at://did:plc:.../site.standard.publication/...\n"
    );

    Ok(())
}
