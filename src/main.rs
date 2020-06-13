mod bookmarks;
mod filesystem;
use std::path::Path;

fn main() {
    let file_string = format!("{}/{}", filesystem::get_app_root_path(), "bookmarks.html");
    println!("{}", file_string);
    bookmarks::find_broken_links(Path::new(&file_string)).unwrap();
}
