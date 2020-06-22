mod bookmarks;
mod filesystem;
mod sharing;

use std::thread;

fn main() {
    thread::spawn(|| {
        sharing::listen().unwrap();
    });
    sharing::cast().unwrap();
}
