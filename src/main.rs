mod bookmarks;
mod filesystem;
mod sharing;

use std::thread;

fn main() {
    sharing::listen().unwrap();
    sharing::cast().unwrap();
}
