use lazy_static::lazy_static;
use reqwest::Client;
use scraper::Selector;
use thiserror::Error;

use crate::util::get_html;

#[derive(Debug, Error)]
pub enum GetScopeError {
  #[error("{0}")]
  ReqwestError(#[from] reqwest::Error),

  #[error("scraping failed")]
  ScrapingFailed,
}

pub async fn get_scope(client: &Client) -> Result<String, GetScopeError> {
  lazy_static! {
        static ref NOVA_WIDGET: Selector = Selector::parse("nova-widget").unwrap();
  }

  let doc = get_html(client, "https://fns.stockholm.se/ng/timetable/timetable-viewer/fns.stockholm.se/").await?;

  let scope = doc
        .select(&NOVA_WIDGET)
        .next()
        .map(|e| e.value().attr("scope"))
        .flatten()
        .ok_or(GetScopeError::ScrapingFailed)?
        .to_owned();

  Ok(scope)
}
