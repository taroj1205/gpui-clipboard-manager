use std::{env, time::Duration};

use anyhow::{Result, anyhow};
use async_std::future::timeout;
use scraper::{Html, Selector};
use serde_json::json;
use url::Url;

const GEMINI_MODEL: &str = "gemini-1.5-flash";
const GEMINI_TIMEOUT: Duration = Duration::from_secs(10);
const GEMINI_MAX_INPUT_CHARS: usize = 8000;

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

    let summary = match summarize_with_gemini(url, title.as_deref(), &document).await {
        Ok(summary) => summary,
        Err(err) => {
            eprintln!("Failed to summarize link with Gemini: {err}");
            None
        }
    };

    Ok(Some(LinkMetadata {
        url: url.to_string(),
        title,
        description: summary.or(description),
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

async fn summarize_with_gemini(
    url: &Url,
    title: Option<&str>,
    document: &Html,
) -> Result<Option<String>> {
    let api_key = match gemini_api_key() {
        Some(key) => key,
        None => return Ok(None),
    };
    let source_text = document_text(document);
    if source_text.is_empty() {
        return Ok(None);
    }
    let truncated = truncate_for_summary(&source_text);
    let title_line = title
        .and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        })
        .unwrap_or("(unknown title)");
    let prompt = format!(
        "Summarize the page below in 1-2 concise sentences.\nTitle: {title_line}\nURL: {url}\nContent: {truncated}"
    );

    let request_body = json!({
        "contents": [
            {
                "role": "user",
                "parts": [{ "text": prompt }]
            }
        ],
        "generationConfig": {
            "temperature": 0.2,
            "maxOutputTokens": 120
        }
    });
    let endpoint = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{GEMINI_MODEL}:generateContent?key={api_key}"
    );
    let request = surf::post(endpoint)
        .body_json(&request_body)
        .map_err(|err| anyhow!(err.to_string()))?;
    let mut response = match timeout(GEMINI_TIMEOUT, request).await {
        Ok(Ok(response)) => response,
        Ok(Err(err)) => return Err(anyhow!(err.to_string())),
        Err(_) => return Ok(None),
    };
    if !response.status().is_success() {
        return Ok(None);
    }
    let payload: serde_json::Value = response
        .body_json()
        .await
        .map_err(|err| anyhow!(err.to_string()))?;
    let summary = payload
        .get("candidates")
        .and_then(|value| value.get(0))
        .and_then(|value| value.get("content"))
        .and_then(|value| value.get("parts"))
        .and_then(|value| value.get(0))
        .and_then(|value| value.get("text"))
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());
    Ok(summary)
}

fn gemini_api_key() -> Option<String> {
    env::var("GEMINI_API_KEY")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn document_text(document: &Html) -> String {
    let text = document.root_element().text().collect::<Vec<_>>().join(" ");
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_for_summary(text: &str) -> String {
    if text.len() <= GEMINI_MAX_INPUT_CHARS {
        return text.to_string();
    }
    let mut output = text[..GEMINI_MAX_INPUT_CHARS].to_string();
    output.push_str("...");
    output
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
