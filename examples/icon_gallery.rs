//! Experiment driver: a drawio with one node per icon. `C43_NATIVE_ICONS=1` switches to native shapes.
//! usage: cargo run --example icon_gallery > out/gallery.drawio
use c43::drawio::graph::doc_from_json;
use c43::drawio::pretty::PrettyHints;
use serde_json::json;

fn main() {
    let doc = doc_from_json(&json!({"nodes":[
        {"uid":"system:g","name":"Icons","type":"system","attributes":{}},
        {"uid":"u","name":"User","type":"person","attributes":{}},
        {"uid":"d","name":"Db","type":"database","attributes":{}},
        {"uid":"s","name":"Store","type":"store","attributes":{}},
        {"uid":"b","name":"Bus","type":"bus","attributes":{}}
    ],"relations":[
        {"start":"u","is":"uses","end":"d"},{"start":"u","is":"uses","end":"s"},{"start":"u","is":"uses","end":"b"}
    ]}));
    let hints: PrettyHints = serde_json::from_value(json!({"icons": {"u":"user","d":"db","s":"store","b":"bus"}})).unwrap();
    print!("{}", c43::drawio::render_with_hints(&doc, &hints).unwrap().xml);
}
