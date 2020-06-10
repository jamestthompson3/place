use anyhow::Result;
use reqwest::Client;
use scraper::{Html, Selector};

use std::io::Read;
use std::collections::BTreeSet;
use std::fs::File;
use std::path::Path;

use soup::prelude::*;

use crate::bookmarks::data::{Bookmark};
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
    bookmark_file.unwrap().read_to_string(&mut doc_string);
    let soup = Soup::new(&doc_string);
    // Does find_all recursively search?
    // Folder data structure needs to implement double linked list (ish), pointing to parent and children
    for folder in soup.tag("dt").find_all() {
        for child in folder.children().filter(|child| child.is_element()) {
            if child.name() == "h3" {
                println!(" \x1b[38;5;169m{:?}\x1b[0m",child.text());
            }
            if child.name() == "a" {
                println!("    {}",child.text());
            }
            // if child.name() == "dt" {
            //             println!("        \x1b[38;5;41m{:?}\x1b[0m",child.text().trim_matches('\n'));
            // }

            if child.name() == "dl" {
                for grandchild in child.children().filter(|child| child.is_element()) {
                    if grandchild.name() == "h3" {
                        println!(" \x1b[38;5;111m{:?}\x1b[0m",child.text());
                    }
                    // Somehow, bookmark descriptions are getting mashed together
                    // if grandchild.name() == "dt" {
                    //     println!("        \x1b[38;5;81m{:?}\x1b[0m",child.text().trim_matches('\n'));
                    // }
                }
            }
        }

    }
    // for link in soup.tag("a").find_all() {
    //     println!("{}",link.text());
    // }
    Ok(())
}

// pub fn import_from_html() -> Result<()> {
//     let bookmark_file = open_data_file("bookmarks.html");
//     let mut doc_string = String::new();
//     bookmark_file.unwrap().read_to_string(&mut doc_string);
//     let document = Html::parse_document(&doc_string);
//     let folder_selector = Selector::parse("dl").unwrap();
//     let description_selector = Selector::parse("dd").unwrap();
//     let link_selector = Selector::parse("a").unwrap();
//     let folder_title_selector = Selector::parse("h3").unwrap();
//     let selected = document.select(&folder_selector).next().unwrap();
//     // for folder in document.select(&link_selector) {
//     //     println!("{:?}",folder.html());
//     // }
//     let bookmarks = document.select(&folder_selector);
//     for bookmark in bookmarks {
//         for title in bookmark.select(&folder_title_selector) {
//             println!(" \x1b[38;5;169m{:?}\x1b[0m",title.inner_html());
//             for sibling in title.next_siblings() {
//                 match sibling.value().as_element() {
//                     Some(el) => {
//                         if el.name() == "dl" {
//                             for child in sibling.children() {
//                                 if child.value().as_element().unwrap().name() == "dt" {
//                                     for grandchild in child.children() {
//                                         if grandchild.value().is_element() {
//                                             println!("{:?}",grandchild.value().as_element().unwrap().attrs().collect::<Vec<_>>());
//                                         }
//                                     }
//                                 }
//                             }
//                         }
//                     }
//                     _ => {}
//                 }
//             }
//         }
//         // let titles = bookmark.select(&folder_title_selector).collect::<Vec<_>>();
//         // println!("{:?}",titles);
//         // match titles {
//         //     Some(title) => {
//         //         println!("\n");
//         //         println!(" \x1b[38;5;169m{:?}\x1b[0m",title.inner_html());
//         //     },
//         //     _ => {}
//         // }
//         // let mut links = bookmark.select(&link_selector).next();
//         // match links {
//         //     Some(link) => {
//         //         println!("{:?}",link.inner_html());
//         //     },
//         //     _ => {}
//         // }

//         let mut descriptions = bookmark.select(&description_selector).next();
//         match descriptions {
//             Some(description) => {
//                 println!("\x1b[38;5;28m{:?}\x1b[0m",description.inner_html().trim_matches('\n'));
//             },
//             _ => {}
//         }
//         // println!("{:?}",bookmark.html());
//         // let parent = bookmark.parent().unwrap();
//         // for child in parent.children() {
//         //     let child_value = child.value();
//         //     if child_value.is_element() {
//         //         if child_value.as_element().unwrap().name() == "dt" {
//         //             println!("{:?}",child_value.as_element());
//         //         }
//         //     }
//         // }
//     }

//     Ok(())
// }


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
