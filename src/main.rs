use std::fmt::Write as _;
use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use std::path::PathBuf;

use argh::FromArgs;
use color_eyre::Result;
use fitgirl_filekeeper::scrape::scrape_game;
use fitgirl_filekeeper::{DirectFile, extract_direct_link, initialize_cookies};
use inquire::MultiSelect;
use spdlog::{error, info};
use wreq::Client;
use wreq_util::Emulation;

#[compio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let Args {
        version,
        fitgirl_url,
        output_dir,
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

    let mut output = String::new();
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

        writeln!(
            &mut output,
            "{direct_link}
    out={slug}/{file_name}
    continue=true"
        )?;
    }

    create_dir_all(&output_dir)?;
    OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(output_dir.join(slug).with_extension("txt"))?
        .write_all(output.as_bytes())?;

    Ok(())
}

#[derive(FromArgs)]
/// Reach new heights.
struct Args {
    /// show version and exit.
    #[argh(switch, short = 'V')]
    version: bool,

    /// directory to generate aria2 input files.
    #[argh(option, short = 'o', default = "PathBuf::from(\"./aria2\")")]
    output_dir: PathBuf,

    /// fitgirl game url to scrape.
    #[argh(positional)]
    fitgirl_url: String,
}
