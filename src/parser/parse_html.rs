use http::StatusCode;
use scraper::{Html, Selector};
use std::io;
use url::Url;

use crate::http_client::connection::ResponseData;

pub fn parse_html(response: ResponseData) -> Result<(String, String, Vec<String>), io::Error> {
    if !StatusCode::from_u16(response.status_code)
        .unwrap()
        .is_success()
    {
        eprintln!(
            "request to url: {} is returning status_code {}",
            response.url, response.status_code
        );
        return Err(io::Error::new(io::ErrorKind::ConnectionRefused, "Invalid status code"));
    }

    let html = match &response.html {
        Some(h) => h.as_str(),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Html could not be parsed",
            ));
        }
    };

    let document = Html::parse_document(html);

    let site_title = match extract_website_title(&document) {
        Some(title) => title,
        None => format!("No title for url {}", response.url),
    };

    let url: Url =
        Url::parse(response.url.as_str()).expect("Error occured while parsing response url ");

    let child_refs = extract_child_sites(&document, &url);

    Ok((url.to_string(), site_title, child_refs))
}

fn extract_website_title(doc: &Html) -> Option<String> {
    let selector = Selector::parse("title").unwrap();

    let title = doc
        .select(&selector)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string());
    // title = Option::Some(element.text().collect::<String>());

    title
}

fn extract_child_sites(doc: &Html, url: &Url) -> Vec<String> {
    let selector = Selector::parse("a[href]").unwrap();

    doc.select(&selector)
        .filter_map(|h| h.value().attr("href"))
        .filter_map(|raw| url.join(raw).ok())
        .filter(|u| matches!(u.scheme(), "http" | "https"))
        .map(|url_string| url_string.to_string())
        .collect()
    // for element in doc.select(selector){
    //
    // }
}
