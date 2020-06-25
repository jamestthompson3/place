mod bookmarks;
mod filesystem;
mod sharing;

use std::thread;

fn main() {
    sharing::become_discoverable();
}
