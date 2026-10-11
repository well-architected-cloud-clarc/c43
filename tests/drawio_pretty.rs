use c43::drawio::graph::doc_from_json;
use c43::drawio::pretty::PrettyHints;
use c43::drawio::render_with_hints;
use serde_json::json;

/// system "s" holding group "g" (a boundary with a role) which holds a person, an api and a store
fn doc() -> serde_json::Value {
    json!({"nodes":[
        {"uid":"system:s","name":"hello","type":"system","attributes":{}},
        {"uid":"g","name":"Hello Inc.","type":"boundary","attributes":{"role":"enterprise boundary"}},
        {"uid":"user","name":"Polite person","type":"person","attributes":{"role":"person"}},
        {"uid":"api","name":"Greeting system","type":"backend","attributes":{"role":"software system"}},
        {"uid":"store","name":"Greeting storage","type":"jsonstore","attributes":{}},
        {"uid":"bus","name":"Events","type":"eventbus","attributes":{}}
    ],"relations":[
        {"start":"g","is":"contains","end":"api"},
        {"start":"g","is":"contains","end":"store"},
        {"start":"user","is":"uses","end":"api"},
        {"start":"api","is":"uses","end":"store"},
        {"start":"api","is":"uses","end":"bus"}
    ]})
}

fn hints() -> PrettyHints {
    serde_json::from_value(json!({"icons": {"user": "user", "store": "db", "bus": "bus"}})).unwrap()
}

fn xml() -> String {
    let x = render_with_hints(&doc_from_json(&doc()), &hints()).unwrap().xml;
    if let Ok(p) = std::env::var("DUMP_PRETTY") { std::fs::write(p, &x).unwrap(); }
    x
}

fn cell<'a>(xml: &'a str, id: &str) -> &'a str {
    xml.lines().find(|l| l.contains(&format!("<mxCell id=\"{id}\""))).unwrap_or_else(|| panic!("no cell {id}"))
}

fn attr(line: &str, k: &str) -> String {
    line.split(&format!(" {k}=\"")).nth(1).unwrap().split('"').next().unwrap().to_string()
}

fn num(line: &str, k: &str) -> f64 {
    attr(line, k).parse().unwrap()
}

fn style_num(line: &str, k: &str) -> f64 {
    line.split(&format!("{k}=")).nth(1).unwrap().split(';').next().unwrap().parse().unwrap()
}

/// the node text as drawio shows it: html with tags kept, attribute escaping undone
fn html(line: &str) -> String {
    attr(line, "value").replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&amp;", "&")
}

#[test]
fn strokes_are_4_2_1() {
    let x = xml();
    assert_eq!(style_num(cell(&x, "n-api"), "strokeWidth"), 4.0);
    assert_eq!(style_num(cell(&x, "frame"), "strokeWidth"), 2.0);
    assert_eq!(style_num(cell(&x, "g-g"), "strokeWidth"), 1.0);
    let edge = x.lines().find(|l| l.contains("edge=\"1\"")).unwrap();
    assert_eq!(style_num(edge, "strokeWidth"), 1.0);
}

#[test]
fn nodes_stay_squares() {
    let x = xml();
    ["n-user", "n-api", "n-bus"].iter().for_each(|id| assert_eq!(num(cell(&x, id), "width"), num(cell(&x, id), "height"), "{id}"));
}

#[test]
fn node_text_is_bold_name_then_role_in_brackets() {
    let x = xml();
    assert_eq!(html(cell(&x, "n-api")), "<b>Greeting</b><br><b>system</b><br>[ software<br>system ]");
}

#[test]
fn node_without_a_given_role_shows_its_type() {
    let x = xml();
    assert_eq!(html(cell(&x, "n-bus")), "<b>Events</b><br>[ eventbus ]");
}

#[test]
fn group_text_is_bottom_left_inside_the_group() {
    let x = xml();
    let (g, t) = (cell(&x, "g-g"), cell(&x, "gt-g"));
    assert_eq!(html(t), "<b>Hello Inc.</b><br>[ enterprise boundary ]");
    assert!(attr(t, "style").contains("align=left"));
    assert!(num(t, "x") > num(g, "x") && num(t, "x") + num(t, "width") <= num(g, "x") + num(g, "width"));
    // the text block ends above the frame's bottom line
    assert!(num(t, "y") + num(t, "height") < num(g, "y") + num(g, "height"));
    // and lies in the bottom half
    assert!(num(t, "y") > num(g, "y") + num(g, "height") / 2.0);
}

#[test]
fn icons_sit_on_the_top_edge_at_the_left_corner_half_outside() {
    let x = xml();
    ["user", "store", "bus"].iter().for_each(|id| {
        let (n, i) = (cell(&x, &format!("n-{id}")), cell(&x, &format!("i-{id}")));
        assert!(attr(i, "style").contains("data:image/svg+xml,"));
        assert_eq!(num(i, "x") + num(i, "width") / 2.0, num(n, "x"), "{id}");
        // the cell is the artwork's size: no empty margin below the sit line
        assert!(num(i, "height") < num(i, "width"), "{id}");
        // the sit line (the icon's bottom, 2 px = half the node outline below the node's top edge so the two overlap), except for a barrel: its bottom oval hangs below the line through the oval's extremes
        let hang = if id == "store" { 0.1005 * num(i, "height") } else { 0.0 };
        assert!((num(i, "y") + num(i, "height") - hang - num(n, "y") - 2.0).abs() < 1e-6, "{id}");
    });
    assert!(!x.contains("id=\"i-api\""));
    assert!(!x.contains("id=\"i-g\""));
}

#[test]
fn every_node_is_a_plain_square_an_icon_only_adds_flavour() {
    let x = xml();
    ["n-user", "n-api", "n-store", "n-bus"].iter().for_each(|id| {
        let c = cell(&x, id);
        assert_eq!(num(c, "width"), num(c, "height"), "{id}");
        assert!(attr(c, "style").contains("rounded=0;") && !attr(c, "style").contains("shape="), "{id}");
    });
    // edges end on the squares, nothing is pulled in
    assert!(!x.contains("exitPerimeter=0") && !x.contains("entryPerimeter=0"));
}

#[test]
fn role_is_read_from_the_node_attributes_and_widens_the_node() {
    let with = |role: &str| {
        let mut d = doc();
        d["nodes"][3]["attributes"]["role"] = json!(role);
        render_with_hints(&doc_from_json(&d), &hints()).unwrap().xml
    };
    let (short, long) = (with("x"), with("an exceptionally long role description of a system"));
    assert!(num(cell(&long, "n-api"), "width") > num(cell(&short, "n-api"), "width"));
}
