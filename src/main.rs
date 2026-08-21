use std::sync::Arc;

use argh::FromArgs;
use color_eyre::Result;
use fitgirl_filekeeper::scrape::scrape_game;
use fitgirl_filekeeper::{DirectFile, extract_direct_link, initialize_cookies};
use inquire::MultiSelect;
use spdlog::sink::StdStreamSink;
use spdlog::terminal_style::StyleMode;
use spdlog::{error, info};
use wreq::Client;
use wreq_util::Emulation;

#[compio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let stderr_sink = StdStreamSink::builder()
        .stderr()
        .style_mode(StyleMode::Always)
        .build()
        .unwrap();
    let logger = spdlog::Logger::builder()
        .sink(Arc::new(stderr_sink))
        .build()
        .unwrap();
    spdlog::set_default_logger(Arc::new(logger));

    let Args {
        version,
        fitgirl_url,
    } = argh::from_env();

    if version {
        eprintln!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if !fitgirl_url.starts_with("https://fitgirl-repacks.site") {
        error!("Invalid FitGirl-Repacks URL found!");
    }

    let client = Client::builder().emulation(Emulation::Chrome149).build()?;

    let game_info = scrape_game(&client, fitgirl_url).await?;

    let groups = game_info.grouped();
    let selection = MultiSelect::new("Please select groups to download", groups.keys().collect())
        .with_default(
            &groups
                .keys()
                .enumerate()
                .filter_map(|(index, group)| {
                    if ["setup", "fitgirl-repacks"]
                        .iter()
                        .any(|keyword| group.contains(keyword))
                    {
                        Some(index)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
        )
        .prompt()?;

    let slug = &game_info.path_part;
    let filekeeper_urls = selection
        .into_iter()
        .map(|group| groups[group].iter())
        .flatten()
        .collect::<Vec<_>>();

    initialize_cookies(&client).await?;

    for url in filekeeper_urls {
        let DirectFile {
            file_name,
            direct_link,
        } = match extract_direct_link(&client, url).await {
            Ok(direct_link) => direct_link,
            Err(e) => {
                error!("Failed extracting {url}: {e}");
                continue;
            }
        };

        info!("Extracted: {file_name}");

        println!(
            "{direct_link}
    out={slug}/{file_name}
    continue=true"
        );
    }

    Ok(())
}

#[derive(FromArgs)]
/// Reach new heights.
struct Args {
    /// show version and exit
    #[argh(switch, short = 'V')]
    version: bool,

    /// fitgirl game to scrape
    #[argh(positional)]
    fitgirl_url: String,
}
