use http::header::SERVER;
use scraper::Selector;
use wreq::{Client, Uri};

use compio::runtime::spawn_blocking;

use crate::errors::ScrapeError;

#[derive(Debug, Clone)]
pub struct GameInfo {
    pub path_part: String,
    pub fuckingfast_links: Vec<String>,
}

pub async fn scrape_game(client: &Client, url: impl AsRef<str>) -> Result<GameInfo, ScrapeError> {
    let url: Uri = url.as_ref().parse()?;

    let path_slug = url
        .path()
        .split("/")
        .nth(1)
        .ok_or(ScrapeError::UnexpectedURL)?
        .to_string();

    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| ScrapeError::RequestError(e.to_string()))?;

    // Should not block ISP ip now
    if resp
        .headers()
        .get(SERVER)
        .is_some_and(|server| server.as_bytes() == b"ddos-guard")
    {
        return Err(ScrapeError::DDoSGuarded);
    }

    let document = resp
        .text()
        .await
        .map_err(|e| ScrapeError::RequestError(e.to_string()))?;

    let fuckingfast_links = spawn_blocking(move || parse_html(document))
        .await
        .map_err(|_| ScrapeError::JoinError)??;

    Ok(GameInfo {
        path_part: path_slug,
        fuckingfast_links,
    })
}

fn parse_html(document: impl AsRef<str>) -> Result<Vec<String>, ScrapeError> {
    let document = document.as_ref();
    let document = scraper::Html::parse_document(document);

    let file_hoster = Selector::parse("div.entry-content ul > li:nth-child(2) > a")?;
    let tags = document
        .select(&file_hoster)
        .filter(|tag| {
            tag.text()
                .collect::<String>()
                .contains("FileHoster: FuckingFast")
        })
        .collect::<Vec<_>>();

    let single_tag = match tags.len() {
        0 => return Err(ScrapeError::FuckingFastSourceMissing)?,
        _ => tags[0],
    };

    let file_hoster_spolier = Selector::parse(
        "div.entry-content ul > li:nth-child(2) > div.su-spoiler > div.su-spoiler-content",
    )?;

    let spoiler_content = document.select(&file_hoster_spolier).collect::<Vec<_>>();
    match &*spoiler_content {
        &[] => Ok(vec![
            single_tag
                .attr("href")
                .ok_or(ScrapeError::FuckingFastSourceMissing)?
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
            results.sort_unstable();
            results.dedup();
            Ok(results)
        }
    }
}
