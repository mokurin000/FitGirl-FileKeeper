use std::borrow::Cow;

use color_eyre::Result;
use color_eyre::eyre::eyre;
use spdlog::info;
use wreq::header::{CONTENT_TYPE, COOKIE, HeaderValue, LOCATION};
use wreq::{Client, redirect};

const DOWNLOAD_API: &str = "https://filekeeper.net/download";

pub async fn initialize_cookies(client: &Client) -> Result<()> {
    info!("Populating cookies for filekeeper...");

    let resp = client
        .get(DOWNLOAD_API)
        .header(COOKIE, "file_code=zpgpm6ak0drc; lang=english")
        .redirect(redirect::Policy::none())
        .send()
        .await?;

    resp.error_for_status()?;

    Ok(())
}

pub async fn extract_direct_link(client: &Client, file_code: &str) -> Result<String> {
    let resp = client
        .post(DOWNLOAD_API)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(format!("op=download2&id={file_code}&rand=&referer=https%3A%2F%2Ffitgirl-repacks.site%2F&method_free=Free+download&down_direct=1"))
        .redirect(redirect::Policy::none())
        .send()
        .await?;

    let direct_link = resp
        .headers()
        .get(LOCATION)
        .map(HeaderValue::as_bytes)
        .map(String::from_utf8_lossy)
        .map(Cow::into_owned);

    resp.error_for_status()?;

    Ok(direct_link.ok_or_else(|| eyre!("Direct link was missing!"))?)
}
