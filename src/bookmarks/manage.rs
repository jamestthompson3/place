use anyhow::Result;
use reqwest::Client;
use scraper::{Html, Selector};
use scraper::ElementRef;

use std::collections::BTreeSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::bookmarks::data::{Bookmark, Folder};
use crate::filesystem::open_data_file;

struct PreppedHTML {
    pub document: Html,
    pub selector: Selector,
    pub client: Client,
}

pub fn add_bookmark(bookmark: Bookmark) -> Result<()> {
    let bookmark_file = open_data_file("bookmarks.html");
    Ok(())
}

pub fn import_from_html() -> Result<()> {
    let bookmark_file = open_data_file("bookmarks.html");
    let mut doc_string = String::new();
    bookmark_file.unwrap().read_to_string(&mut doc_string)?;
    let doc = Html::parse_document(&doc_string);
    let mut folders: Vec<Folder> = Vec::new();
    // Does find_all recursively search?
    // Folder data structure needs to implement double linked list (ish), pointing to parent and children
    let a_selector = Selector::parse("a").unwrap();
    let dt_selector = Selector::parse("dl").unwrap();
    let dd_selector = Selector::parse("dd").unwrap();
    for element in doc.select(&dt_selector) {
        for child in element.children() {
            match ElementRef::wrap(child) {
                Some(el) => {
                    let mut desc = el.select(&dd_selector);
                    for anchor in el.select(&a_selector) {
                        println!("{}",anchor.inner_html());
                        match desc.next() {
                            Some(d) => println!("  \x1b[38;5;81m{:?}\x1b[0m",d.inner_html()),
                            None => {println!("\n---")}
                        }
                    }
                }
                None => {
                }
            }
        }

    }
    Ok(())
}

fn prepare_html(path: &Path, selector: &str) -> Result<PreppedHTML> {
    let mut html_doc = File::open(path)?;
    let mut doc_string = String::new();
    html_doc.read_to_string(&mut doc_string).unwrap();
    let document = Html::parse_document(&doc_string);
    let selector = Selector::parse(selector).unwrap();
    let client = reqwest::Client::new();
    Ok(PreppedHTML { document, selector, client })
}

pub fn find_broken_links(path: &Path) -> Result<Vec<String>> {
    let mut borked_links = Vec::new();
    let prep_request = prepare_html(path, "a")?;
    let anchors = prep_request.document.select(&prep_request.selector).collect::<Vec<_>>();
    let mut index = 0;
    for element in prep_request.document.select(&prep_request.selector) {
        println!("Checking {} of {}",index, anchors.len());
        index+= 1;
        let href = element.value().attr("href").unwrap();
        smol::run(async {
            let resp = prep_request.client.get(href).header("Accept", "*/*").header("User-Agent", "place-app").send().await;
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
    let prep_request = prepare_html(path, "a")?;
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
