use reqwest::{Client, IntoUrl};
use scraper::Html;

pub async fn get_html(client: &Client, url: impl IntoUrl) -> Result<Html, reqwest::Error> {
  let raw = client.get(url).send().await?.text().await?;

  Ok(Html::parse_document(&raw))
}
