mod api;

use api::fetch;
fn main() {
    let endpoint = String::from("getPublicPageData");
    let body = r#"{"type":"block-space","name":"page","blockId":"b59819a3-270d-477f-b9d6-073f09456b8e","spaceDomain":"teukka","showMoveTo":false,"saveParent":false}"#;
    let body = fetch(endpoint, body.to_string());
    println!("{:?}", body);
}
