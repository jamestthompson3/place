mod bookmarks;
use bookmarks::find_broken_links;

use std::path::Path;

fn main() {
    let doc_path = Path::new("/home/taylor/Downloads/Raindrop.io.copy.html");
    find_broken_links(&doc_path).unwrap();
}
