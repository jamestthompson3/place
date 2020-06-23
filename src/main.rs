mod bookmarks;
mod filesystem;
mod sharing;

use std::thread;

fn main() {
    sharing::cast().unwrap();
}
