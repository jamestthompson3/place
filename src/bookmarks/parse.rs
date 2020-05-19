use anyhow::Result;
use scraper::{Html, Selector};
use std::io::Read;

use std::fs::File;
use std::path::Path;

pub fn find_broken_links(path: &Path) -> Result<()> {
    let mut html_doc = File::open(path)?;
    let mut doc_string = String::new();
    html_doc.read_to_string(&mut doc_string).unwrap();
    let document = Html::parse_document(&doc_string);
    let selector = Selector::parse("a").unwrap();
    let client = reqwest::Client::new();
    let mut borked_links = Vec::new();
    for element in document.select(&selector) {
        let href = element.value().attr("href").unwrap();
        println!("{}", href);
        smol::run(async {
            let resp = client.get(href).header("Accept", "*/*").header("User-Agent", "curl/7.70.0").send().await;
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
    println!("{:?}",borked_links);
    Ok(())
}
