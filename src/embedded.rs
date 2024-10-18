use rust_embed::Embed;

#[derive(Embed, Debug)]
#[folder = "$CARGO_MANIFEST_DIR/example/static/"]
pub struct Static;

#[derive(Embed, Debug)]
#[folder = "$CARGO_MANIFEST_DIR/example/templates/"]
pub struct Templates;
