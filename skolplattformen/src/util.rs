use std::fmt::Debug;

use reqwest::{Client, IntoUrl};
use scraper::Html;
use tracing::{instrument, trace};

#[instrument(skip(client))]
pub async fn get_html<U: IntoUrl + Debug>(client: &Client, url: U) -> Result<Html, reqwest::Error> {
    trace!("GET {:?}", url);

    let res = client.get(url).send().await?;

    trace!(status = ?res.status());

    Ok(Html::parse_document(&res.text().await?))
}
