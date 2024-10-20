use lazy_static::lazy_static;
use log::{error, info};
use rust_embed::Embed;
use std::fs;
use std::fs::File;
use std::io::{Error, Write};
use std::path::Path;
use tera::Tera;

#[derive(Embed, Debug)]
#[folder = "$CARGO_MANIFEST_DIR/example/static/"]
pub struct Static;

#[derive(Embed, Debug)]
#[folder = "$CARGO_MANIFEST_DIR/example/templates/"]
pub struct Templates;

lazy_static! {
    pub static ref EMBEDDED_TERA: Tera = {
        let mut tera = Tera::default();
        tera.autoescape_on(vec![]);
        for name in Templates::iter() {
            let template = Templates::get(name.as_ref()).unwrap();
            let template_str = std::str::from_utf8(template.data.as_ref()).unwrap();
            tera.add_raw_template(&name, template_str).unwrap();
        }
        tera
    };
}

pub fn generate_static(static_folder: &Path) {
    if let Err(e) = fs::create_dir_all(&static_folder) {
        error!("Unable to create static directory: {}", e);
        return;
    }
    for name in Static::iter() {
        let static_file = Static::get(name.as_ref()).unwrap();
        let file_data = static_file.data;
        let file_path = static_folder.join(name.as_ref());

        match write_bytes_to_file(&static_folder.join(file_path.as_path()), &file_data) {
            Ok(_) => info!("Generated {}", &file_path.display()),
            Err(e) => eprintln!("Error writing file: {}", e),
        }
    }
}

fn write_bytes_to_file(filename: &Path, data: &[u8]) -> Result<(), Error> {
    // Create or open the file for writing
    let mut file = File::create(filename)?;

    // Write the byte slice to the file
    file.write_all(data)?;

    Ok(())
}
