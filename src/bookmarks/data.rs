use anyhow::{Error, Result};
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
    link: String,
    add_date: u32,
    last_modified: u32,
    tags: Option<Vec<String>>,
    description: String,
    title: String,
}

impl Bookmark {
    pub fn to_html(&self) -> Result<String, Error> {
        let tag_string;
        match &self.tags {
            Some(tags) => tag_string = tags.join(","),
            _ => tag_string = String::from(""),
        }

        let bookmark_string = format!(
            "<DT><A HREF=\"{}\" ADD_DATE=\"{}\" LAST_MODIFIED=\"{}\" TAGS=\"{}\">{}</A>\n<DD>{}",
            self.link, self.add_date, self.last_modified, tag_string, self.title, self.description
        );
        Ok(bookmark_string)
    }
    pub fn from_html(html: &str) -> Result<Bookmark, Error> {
        let document = Html::parse_fragment(html);
        let link_selector = Selector::parse("a").unwrap();
        let desc_selector = Selector::parse("dd").unwrap();
        let link = document.select(&link_selector).next().unwrap();
        let description = document.select(&desc_selector).next().unwrap();
        let tag_attrs = link.value().attr("tags");
        let tags;
        match tag_attrs {
            Some(attrs) => tags = Some(attrs.split(",").map(|x| x.to_string()).collect()),
            _ => tags = None,
        }

        let bookmark = Bookmark {
            link: link.value().attr("href").unwrap().to_string(),
            add_date: link.value().attr("add_date").unwrap().parse::<u32>()?,
            last_modified: link.value().attr("last_modified").unwrap().parse::<u32>()?,
            tags,
            description: description.text().collect::<Vec<_>>().join(" "),
            title: link.text().collect::<Vec<_>>().join(" "),
        };
        Ok(bookmark)
    }
}

#[cfg(test)]
mod tests {
    use crate::bookmarks::data::{Bookmark, Folder};
    #[test]
    fn parses_to_html() {
        let bookmark = Bookmark {
        title: "luvit/luv".to_string(),
        tags: Some(vec!["programming".to_string(), "vim".to_string()]),
        add_date: 1578165853,
        last_modified: 1578165853,
        link: "https://github.com/luvit/luv/blob/master/docs.md#uvspawnfile-options-onexit".to_string(),
        description: "Bare libuv bindings for lua. Contribute to luvit/luv development by creating an account on GitHub.".to_string(),
        };
        let test_string = String::from("<DT><A HREF=\"https://github.com/luvit/luv/blob/master/docs.md#uvspawnfile-options-onexit\" ADD_DATE=\"1578165853\" LAST_MODIFIED=\"1578165853\" TAGS=\"programming,vim\">luvit/luv</A>\n<DD>Bare libuv bindings for lua. Contribute to luvit/luv development by creating an account on GitHub.");
        assert_eq!(bookmark.to_html().unwrap(), test_string);
    }
    #[test]
    fn parses_from_html() {
        let test_string = String::from("<DT><A HREF=\"https://github.com/luvit/luv/blob/master/docs.md#uvspawnfile-options-onexit\" ADD_DATE=\"1578165853\" LAST_MODIFIED=\"1578165853\" TAGS=\"programming,vim\">luvit/luv</A>\n<DD>Bare libuv bindings for lua. Contribute to luvit/luv development by creating an account on GitHub.");
        assert_eq!(
            Bookmark::from_html(&test_string)
                .unwrap()
                .to_html()
                .unwrap(),
            test_string
        );
    }
}
