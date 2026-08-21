use std::collections::BTreeMap;

use scraper::Selector;
use spdlog::info;
use wreq::{Client, Uri};

use compio::runtime::spawn_blocking;

use crate::errors::ScrapeError;

#[derive(Debug, Clone)]
pub struct GameInfo {
    pub path_part: String,
    pub filekeeper_links: Vec<String>,
}

pub async fn scrape_game(client: &Client, url: impl AsRef<str>) -> Result<GameInfo, ScrapeError> {
    let url: Uri = url.as_ref().parse()?;
    let path_slug = url
        .path()
        .split("/")
        .filter(|s| !s.is_empty())
        .next()
        .ok_or(ScrapeError::UnexpectedURL)?
        .to_string();

    info!("Scraping: {path_slug}");

    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| ScrapeError::RequestError(e.to_string()))?;

    // Should not block ISP ip now
    if resp.status() == 403 {
        return Err(ScrapeError::DDoSGuarded);
    }

    let document = resp
        .text()
        .await
        .map_err(|e| ScrapeError::RequestError(e.to_string()))?;

    let filekeeper_links = spawn_blocking(move || parse_html(document))
        .await
        .map_err(|_| ScrapeError::JoinError)??;

    Ok(GameInfo {
        path_part: path_slug,
        filekeeper_links,
    })
}

fn parse_html(document: impl AsRef<str>) -> Result<Vec<String>, ScrapeError> {
    let document = document.as_ref();
    let document = scraper::Html::parse_document(document);

    let file_hoster = Selector::parse("div.entry-content ul > li:nth-child(3) > a")?;
    let tags = document
        .select(&file_hoster)
        .filter(|tag| {
            tag.text()
                .collect::<String>()
                .contains("Filehoster: FileKeeper")
        })
        .collect::<Vec<_>>();

    let single_tag = match tags.len() {
        0 => return Err(ScrapeError::FileKeeperSourceMissing)?,
        _ => tags[0],
    };

    let file_hoster_spolier =
        Selector::parse("div.entry-content ul > div.su-spoiler > div.su-spoiler-content")?;

    let spoiler_content = document.select(&file_hoster_spolier).collect::<Vec<_>>();
    match &*spoiler_content {
        &[] => Ok(vec![
            single_tag
                .attr("href")
                .ok_or(ScrapeError::FileKeeperSourceMissing)?
                .to_string(),
        ]),
        spoilers => {
            let mut results = Vec::new();
            for spoiler in spoilers {
                results.extend(
                    spoiler
                        .select(&Selector::parse("a")?)
                        .filter_map(|tag| tag.attr("href"))
                        .map(str::to_string),
                );
            }
            results.dedup();
            Ok(results)
        }
    }
}

impl GameInfo {
    pub fn grouped(&self) -> BTreeMap<&str, Vec<&str>> {
        let mut groups = BTreeMap::<&str, Vec<&str>>::new();

        for link in &self.filekeeper_links {
            let link = &**link;

            // Validated before construction
            let group = link.split("/").last().unwrap();
            let group = group.split_once(".part").map(|(a, _)| a).unwrap_or(group);
            let group = group.strip_suffix(".bin").unwrap_or(group);
            let group = group.strip_suffix(".rar").unwrap_or(group);
            groups.entry(group).or_insert(Default::default()).push(link);
        }

        groups
    }
}
