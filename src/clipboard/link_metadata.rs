use std::time::Duration;

use anyhow::{Result, anyhow};
use async_std::future::timeout;
use scraper::{Html, Selector};
use url::Url;

#[derive(Clone, Debug)]
pub struct LinkMetadata {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub site_name: Option<String>,
}

pub fn parse_link_url(text: &str) -> Option<Url> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    let url = Url::parse(trimmed).ok()?;
    match url.scheme() {
        "http" | "https" => Some(url),
        _ => None,
    }
}

pub async fn fetch_link_metadata(url: &Url) -> Result<Option<LinkMetadata>> {
    let body = match timeout(Duration::from_secs(8), fetch_body(url)).await {
        Ok(Ok(body)) => body,
        Ok(Err(err)) => return Err(err),
        Err(_) => return Ok(None),
    };

    let document = Html::parse_document(&body);
    let title =
        meta_content(&document, "meta[property=\"og:title\"]").or_else(|| title_text(&document));
    let description = meta_content(&document, "meta[property=\"og:description\"]")
        .or_else(|| meta_content(&document, "meta[name=\"description\"]"));
    let site_name = meta_content(&document, "meta[property=\"og:site_name\"]")
        .or_else(|| meta_content(&document, "meta[name=\"application-name\"]"));

    Ok(Some(LinkMetadata {
        url: url.to_string(),
        title,
        description,
        site_name,
    }))
}

async fn fetch_body(url: &Url) -> Result<String> {
    let mut response = surf::get(url.as_str())
        .header("User-Agent", "gpui-clipboard-manager/0.1")
        .await
        .map_err(|err| anyhow!(err.to_string()))?;
    if !response.status().is_success() {
        return Ok(String::new());
    }
    response
        .body_string()
        .await
        .map_err(|err| anyhow!(err.to_string()))
}

fn title_text(document: &Html) -> Option<String> {
    let selector = Selector::parse("title").ok()?;
    let title = document
        .select(&selector)
        .next()
        .map(|node| node.text().collect::<String>())?;
    let trimmed = title.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn meta_content(document: &Html, selector: &str) -> Option<String> {
    let selector = Selector::parse(selector).ok()?;
    let value = document
        .select(&selector)
        .filter_map(|node| node.value().attr("content"))
        .map(str::trim)
        .find(|content| !content.is_empty())?;
    Some(value.to_string())
}
