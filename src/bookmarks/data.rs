use anyhow::{Error, Result};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Folder {
    title: String,
    bookmarks: Vec<Bookmark>,
    add_date: u32,
    last_modified: u32,
}

#[derive(Deserialize, Serialize)]
pub struct Bookmark {
    folder: Option<Folder>,
    link: String,
    add_date: u32,
    last_modified: u32,
    tags: Vec<String>,
    description: String,
    title: String,
}

pub struct PreppedHTML {
    pub document: Html,
    pub selector: Selector,
    pub client: Client,
}

impl Bookmark {
    fn to_html(&self) -> Result<String, Error> {
        let mut tag_string = String::new();
        tag_string = format!(
            "<DT><A HREF=\"{}\" ADD_DATE=\"{}\" LAST_MODIFIED=\"{}\" TAGS=\"{}\">{}</A>\n<DD>{}",
            self.link,
            self.add_date,
            self.last_modified,
            self.tags.join(","),
            self.title,
            self.description
        );
        Ok(tag_string)
    }
    fn from_html(&self, html: &str) -> Result<Bookmark, Error> {
        let document = Html::parse_document(html.to_string());
    }
}

#[cfg(test)]
mod tests {
    use crate::bookmarks::data::{Bookmark, Folder};
    #[test]
    fn parses_to_html() {
        let folder = Folder {
            title: "Test".to_string(),
            bookmarks: vec![],
            add_date: 1578165853,
            last_modified: 1578165853,
        };
        let bookmark = Bookmark {
        title: "luvit/luv".to_string(),
        tags: vec!["programming".to_string(), "vim".to_string()],
        add_date: 1578165853,
        last_modified: 1578165853,
        link: "https://github.com/luvit/luv/blob/master/docs.md#uvspawnfile-options-onexit".to_string(),
        description: "Bare libuv bindings for lua. Contribute to luvit/luv development by creating an account on GitHub.".to_string(),
            folder: Some(folder)
        };
        let test_string = String::from("<DT><A HREF=\"https://github.com/luvit/luv/blob/master/docs.md#uvspawnfile-options-onexit\" ADD_DATE=\"1578165853\" LAST_MODIFIED=\"1578165853\" TAGS=\"programming,vim\">luvit/luv</A>\n<DD>Bare libuv bindings for lua. Contribute to luvit/luv development by creating an account on GitHub.");
        assert_eq!(bookmark.to_html().unwrap(), test_string);
    }
}
