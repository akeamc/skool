use std::collections::HashMap;

use lazy_static::lazy_static;
use scraper::{ElementRef, Html, Selector};

lazy_static! {
    static ref FORM_SELECTOR: Selector = Selector::parse("form").unwrap();
    static ref INPUT_SELECTOR: Selector = Selector::parse("input").unwrap();
}

pub fn map_form_fields(form: &ElementRef) -> HashMap<String, String> {
    form.select(&INPUT_SELECTOR)
        .filter_map(|e| {
            let v = e.value();
            Some((v.attr("name")?.to_owned(), v.attr("value")?.to_owned()))
        })
        .collect()
}

pub fn scrape_form(html: &Html) -> Option<HashMap<String, String>> {
    let form = html.select(&FORM_SELECTOR).next()?;
    Some(map_form_fields(&form))
}
