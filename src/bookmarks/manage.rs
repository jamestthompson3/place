use anyhow::Result;
use reqwest::Client;
use scraper::{Html, Selector};

use std::io::Read;
use std::collections::BTreeSet;
use std::fs::File;
use std::path::Path;

use crate::bookmarks::data::{Bookmark};


struct PreppedHTML {
    pub document: Html,
    pub selector: Selector,
    pub client: Client,
}


pub fn add_bookmark(bookmark: Bookmark) -> Result<()> {
    Ok(())
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

pub fn find_broken_links(path: &Path) -> Result<Vec<String>> {
    let mut borked_links = Vec::new();
    let prep_request = prepare_html(path)?;
    for element in prep_request.document.select(&prep_request.selector) {
        let href = element.value().attr("href").unwrap();
        smol::run(async {
            let resp = prep_request.client.get(href).header("Accept", "*/*").header("User-Agent", "curl/7.70.0").send().await;
            match resp {
                Ok(res) => {
                    if res.status().as_u16() >= 400 {
                        borked_links.push(href.to_owned());
                    }
                },
                Err(_e) => {
                    borked_links.push(href.to_owned());
                }
            }

        })
    }
    Ok(borked_links)
}

pub fn find_duplicate_links(path: &Path) -> Result<Vec<String>> {
    let mut dup_links: Vec<String> = Vec::new();
    let mut seen_links = BTreeSet::new();
    let prep_request = prepare_html(path)?;
    for element in prep_request.document.select(&prep_request.selector) {
        let href = element.value().attr("href").unwrap();
        if !seen_links.contains(href) {
            seen_links.insert(href.to_owned());
        } else {
            dup_links.push(href.to_owned());
        }
    }
    Ok(dup_links)
}
