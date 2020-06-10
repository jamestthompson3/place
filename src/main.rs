mod bookmarks;
mod filesystem;

fn main() {
    bookmarks::import_from_html();
}
