mod bookmarks;
mod filesystem;
mod sharing;

fn main() {
    sharing::become_discoverable();
}
