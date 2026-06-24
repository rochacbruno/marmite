pub mod auth;
pub mod client;
pub mod credentials;
pub mod publish;

use crate::cli::{AtprotoCommand, Cli};

pub fn dispatch(cmd: &AtprotoCommand, args: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let input_folder = args
        .input_folder
        .as_deref()
        .ok_or("Input folder is required for atproto commands")?;
    match cmd {
        AtprotoCommand::Auth => auth::auth(input_folder, args),
        AtprotoCommand::Publish { force, dry_run } => {
            publish::publish(input_folder, *force, *dry_run, &args.config)
        }
    }
}
