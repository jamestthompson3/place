mod bookmarks;
mod filesystem;
mod sharing;

fn main() {
    // let mut peering = sharing::PeerSharing::new().unwrap();
    // peering.make_discoverable();
    let fname = sharing::protocol::encode();
    sharing::protocol::decode(fname);
}
