use anyhow::Result;
use scraper::{Html, Selector};
use reqwest::Client;

use std::io::Read;
use std::collections::BTreeSet;
use std::fs::File;
use std::path::Path;

struct PreppedHTML {
    document: Html,
    selector: Selector,
    client: Client
}

fn prepare_html(path: &Path) -> Result<PreppedHTML> {
    let mut html_doc = File::open(path)?;
    let mut doc_string = String::new();
    html_doc.read_to_string(&mut doc_string).unwrap();
    let document = Html::parse_document(&doc_string);
    let selector = Selector::parse("a").unwrap();
    let client = reqwest::Client::new();
    Ok(PreppedHTML { document, selector, client })
}

pub fn find_broken_links(path: &Path) -> Result<Vec<&str>> {
    let mut borked_links = Vec::new();
    let prep_request = prepare_html(path)?;
    for element in prep_request.document.select(&prep_request.selector) {
        let href = element.value().attr("href").unwrap();
        smol::run(async {
            let resp = prep_request.client.get(href).header("Accept", "*/*").header("User-Agent", "curl/7.70.0").send().await;
            match resp {
                Ok(res) => {
                    if res.status().as_u16() >= 400 {
                        borked_links.push(href);
                    }
                },
                Err(e) => {
                    borked_links.push(href);
                }
            }

        })
    }
    Ok(borked_links)
}

pub fn find_duplicate_links(path: &Path) -> Result<Vec<&str>> {
    let mut dup_links: Vec<&str> = Vec::new();
    let mut seen_links = BTreeSet::new();
    let prep_request = prepare_html(path)?;
    for element in prep_request.document.select(&prep_request.selector) {
        let href = element.value().attr("href").unwrap();
        if !seen_links.contains(href) {
            seen_links.insert(href);
        } else {
            dup_links.push(href);
        }
    }
    Ok(dup_links)
}
