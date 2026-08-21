use color_eyre::Result;
use fitgirl_filekeeper::{extract_direct_link, initialize_cookies};
use wreq::{Client, Uri};
use wreq_util::Emulation;

#[compio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let client = Client::builder().emulation(Emulation::Chrome149).build()?;
    initialize_cookies(&client).await?;

    let url = "https://filekeeper.net/mo1aao8ranw3/DRIVE_Rally_--_fitgirl-repacks.site_--_.rar";
    let uri = url.parse::<Uri>()?;

    let mut path_segments = uri.path().split("/");
    let file_code = path_segments.nth(1).unwrap();
    let file_name = path_segments.next().unwrap();

    let dl = extract_direct_link(&client, file_code).await?;
    println!("{file_name}: {dl}");

    Ok(())
}
