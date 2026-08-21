use argh::FromArgs;
use color_eyre::Result;
use fitgirl_filekeeper::initialize_cookies;
use fitgirl_filekeeper::scrape::scrape_game;
use spdlog::error;
use wreq::Client;
use wreq_util::Emulation;

#[compio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

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
    initialize_cookies(&client).await?;

    let game_info = scrape_game(&client, fitgirl_url).await?;
    eprintln!("{game_info:#?}");

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
