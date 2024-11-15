use rss::{ChannelBuilder, ItemBuilder};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use crate::config::Marmite;
use crate::content::Content;

pub fn generate_rss(
    contents: &[Content],
    output_path: &Path,
    filename: &str,
    config: &Marmite,
) -> Result<(), String> {
    let mut channel = ChannelBuilder::default()
        .title(&config.name)
        .link(&config.url)
        .description(&config.tagline)
        .generator("marmite".to_string())
        .build();

    for content in contents.iter().take(15) {
        let mut item = ItemBuilder::default()
            .title(content.title.clone())
            .link(format!("{}/{}", &config.url, &content.slug))
            .guid(rss::GuidBuilder::default().value(&content.slug).build())
            .pub_date(content.date.unwrap().to_string())
            .content(content.html.clone())
            .source(
                rss::SourceBuilder::default()
                    .url(&config.url)
                    .title(filename.to_string())
                    .build(),
            )
            .description(content.description.clone())
            .build();

        if let Some(author) = content.authors.first() {
            item.author = Some(author.clone());
        }
        item.categories = content
            .tags
            .iter()
            .map(|tag| rss::CategoryBuilder::default().name(tag.clone()).build())
            .collect();
        channel.items.push(item);
    }

    if let Some(latest_item) = channel.items.first() {
        channel.pub_date = latest_item.pub_date.clone();
    }

    channel.last_build_date = Some(chrono::Utc::now().format("%+").to_string());

    if !config.card_image.is_empty() {
        channel.image = Some(
            rss::ImageBuilder::default()
                .url(format!("{}/{}", &config.url, &config.card_image))
                .build(),
        );
    }

    let rss = channel.to_string();
    let feed_path = output_path.join(format!("{filename}.rss"));
    let mut file = File::create(feed_path).map_err(|e| e.to_string())?;
    file.write_all(rss.as_bytes()).map_err(|e| e.to_string())?;

    Ok(())
}
