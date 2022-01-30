//! Scraping utilities, etc.
use std::{collections::HashMap, fmt::Debug};

use reqwest::{Client, IntoUrl};
use select::{document::Document, node::Node, predicate::Name};
use tracing::{instrument, trace};

#[instrument(skip(client))]
pub(crate) async fn get_doc<U: IntoUrl + Debug>(
    client: &Client,
    url: U,
) -> Result<Document, reqwest::Error> {
    trace!("GET {:?}", url);

    let res = client.get(url).send().await?;

    trace!(status = ?res.status());

    Ok(Document::from(res.text().await?.as_str()))
}

/// Extract form fields with their values from a `form` element.
#[must_use]
pub fn form_fields(form: &Node) -> HashMap<String, String> {
    form.find(Name("input"))
        .filter_map(|n| {
            let name = n.attr("name")?.to_owned();

            Some((name, n.attr("value").unwrap_or("").to_owned()))
        })
        .collect()
}

/// Get the form data from the first form element.
///
/// ```
/// use select::document::Document;
/// use skolplattformen::util::scrape_form;
///
/// let doc = Document::from(r#"<html>
/// <form>
///   <input type="email" name="email" />
///   <input type="text" name="username" value="Prefilled" />
///   <input type="submit" />
/// </form>
/// </html>"#);
///
/// let fields = scrape_form(&doc).unwrap();
///
/// assert_eq!(fields.get("email").unwrap(), "");
/// assert_eq!(fields.get("username").unwrap(), "Prefilled");
/// assert!(fields.get("submit").is_none());
/// ```
#[must_use]
pub fn scrape_form(doc: &Document) -> Option<HashMap<String, String>> {
    let form = doc.find(Name("form")).next()?;
    Some(form_fields(&form))
}
