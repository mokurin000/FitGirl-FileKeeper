use color_eyre::Result;
use fitgirl_filekeeper::{DirectFile, extract_direct_link_, initialize_cookies};
use wreq::Client;
use wreq_util::Emulation;

#[compio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let client = Client::builder().emulation(Emulation::Chrome149).build()?;
    initialize_cookies(&client).await?;

    let url = "https://filekeeper.net/mo1aao8ranw3/DRIVE_Rally_--_fitgirl-repacks.site_--_.rar";
    let DirectFile {
        file_name,
        direct_link,
    } = extract_direct_link_(&client, url).await?;
    println!("{file_name}: {direct_link}");

    Ok(())
}
