use scraper::error::SelectorErrorKind;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScrapeError {
    #[error("IP banned by ddos-guard")]
    DDoSGuarded,
    #[error("FileKeeper hoster links not found")]
    FileKeeperSourceMissing,
    #[error("URL must refers to a single game")]
    UnexpectedURL,
    #[error("http: {0}")]
    RequestError(String),
    #[error("Thread join failed")]
    JoinError,
    #[error("Invalid css selector")]
    InvalidCSSSelector,
    #[error("Invalid URI")]
    InvalidURI(#[from] http::uri::InvalidUri),
}

impl From<SelectorErrorKind<'_>> for ScrapeError {
    fn from(_: SelectorErrorKind<'_>) -> Self {
        Self::InvalidCSSSelector
    }
}
