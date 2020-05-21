use std::path::Path;
mod bookmarks;

fn main() {
    let doc_path = Path::new("/home/taylor/Downloads/Raindrop.io.copy.html");
    // find_duplicate_links(&doc_path).unwrap();
    println!("{:?}", doc_path);
}
