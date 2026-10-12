use c43::drawio::graph::doc_from_json;
use c43::drawio::pretty::PrettyHints;
use c43::drawio::{render, render_layout, render_with_hints};
use c43_layout::{layout, Hints, InputGraph};
use serde_json::json;

fn case(name: &str, f: &str) -> String {
    std::fs::read_to_string(format!("{}/fixtures/layout-cases/{name}/{f}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

fn draw(g: &InputGraph, h: &Hints) -> String {
    let r = layout(g, h, &Default::default()).unwrap();
    render_layout(&r.layout, g.title.as_deref(), g.description.as_deref())
}

fn path(name: &str, f: &str) -> String {
    format!("{}/fixtures/layout-cases/{name}/{f}", env!("CARGO_MANIFEST_DIR"))
}

/// a case is either a router graph + layout hints, or a real c43 dump + look hints (icons)
fn drawn(name: &str) -> String {
    match std::fs::read_to_string(path(name, "c43.json")) {
        Ok(dump) => {
            let hints: PrettyHints = serde_json::from_str(&case(name, "hints.json")).unwrap();
            render_with_hints(&doc_from_json(&serde_json::from_str(&dump).unwrap()), &hints).unwrap().xml
        }
        Err(_) => draw(&serde_json::from_str(&case(name, "graph.json")).unwrap(), &serde_json::from_str(&case(name, "hints.json")).unwrap()),
    }
}

fn golden(name: &str) {
    let got = drawn(name);
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        std::fs::write(path(name, "expected.drawio"), &got).unwrap();
        return;
    }
    let want = case(name, "expected.drawio");
    if got != want {
        let line = got.lines().zip(want.lines()).position(|(a, b)| a != b);
        panic!("{name}: drawio differs from the golden at line {line:?}:\n{:?}\n{:?}",
            line.map(|i| got.lines().nth(i)), line.map(|i| want.lines().nth(i)));
    }
}

#[test]
fn aws() {
    golden("aws")
}

#[test]
fn star16() {
    golden("star16")
}

#[test]
fn groups() {
    golden("groups")
}

#[test]
fn rebob_system() {
    golden("rebob-system")
}

#[test]
fn rebob_container() {
    golden("rebob-container")
}

/// synthetic, from the ticket's mermaid: richer than what c43 generates (explicit roles, a boundary group, labelled relations)
#[test]
fn hello_world_mermaid() {
    golden("hello-world-mermaid");
    let x = drawn("hello-world-mermaid");
    assert!(x.contains("[ enterprise boundary ]") && x.contains("[ software") && x.contains("[ database ]"));
    assert!(x.contains("value=\"says&#xa;hello\"") && x.contains("value=\"persists&#xa;hellos\""));
}

#[test]
fn hello_c43_system() {
    golden("hello-c43-system")
}

/// real dumps carry the look: icons from the hints, the role is the type
#[test]
fn real_dumps_show_icons_and_roles() {
    let x = drawn("rebob-container");
    ["i-@bob/frontend", "i-queue/task-queue", "i-memory/graph"].iter().for_each(|id| assert!(x.contains(&format!("id=\"{id}\"")), "{id}"));
    assert!(x.contains("[ apicontainer ]"));
    assert!(drawn("hello-c43-system").contains("[ frontend ]"));
}

fn graph(v: serde_json::Value) -> InputGraph {
    serde_json::from_value(v).unwrap()
}

fn cell_line<'a>(xml: &'a str, id: &str) -> &'a str {
    xml.lines().find(|l| l.contains(&format!("<mxCell id=\"{id}\""))).unwrap()
}

#[test]
fn labels_are_xml_escaped_and_unicode_is_kept() {
    let g = graph(json!({"title": "T & <x>", "nodes": [{"id":"A","label":"R&D <core> \"x\" — Ünï"},{"id":"B"}], "edges": [{"from":"A","to":"B"}]}));
    let xml = draw(&g, &Hints::default());
    // the name is a plain-text cell of its own (lines separated by the xml newline)
    let value = cell_line(&xml, "nt-A").split("value=\"").nth(1).unwrap().split('"').next().unwrap();
    assert_eq!(value.replace("&#xa;", " "), "R&amp;D &lt;core&gt; &quot;x&quot; — Ünï");
    assert!(!xml.contains("<core>"));
    assert!(cell_line(&xml, "title").contains("value=\"T &amp; &lt;x&gt;\""));
}

#[test]
fn long_title_keeps_the_frame_width_and_gets_room_for_several_lines() {
    let words = (0..30).map(|i| format!("word{i}")).collect::<Vec<_>>().join(" ");
    let base = json!({"nodes":[{"id":"A"},{"id":"B"}],"edges":[{"from":"A","to":"B"}]});
    let with = |t: &str| { let mut v = base.clone(); v["title"] = json!(t); draw(&graph(v), &Hints::default()) };
    let attr = |line: &str, k: &str| -> f64 { line.split(&format!(" {k}=\"")).nth(1).unwrap().split('"').next().unwrap().parse().unwrap() };
    let (short, long) = (with("x"), with(&words));
    assert_eq!(attr(cell_line(&short, "frame"), "width"), attr(cell_line(&long, "frame"), "width"));
    assert!(attr(cell_line(&long, "title"), "height") > attr(cell_line(&short, "title"), "height"));
}

#[test]
fn no_header_cells_without_title_or_description() {
    let xml = draw(&graph(json!({"nodes":[{"id":"A"},{"id":"B"}],"edges":[{"from":"A","to":"B"}]})), &Hints::default());
    assert!(!xml.contains("id=\"title\"") && !xml.contains("id=\"description\""));
    assert!(xml.contains("<mxCell id=\"frame\""));
}

#[test]
fn edge_labels_become_edge_values() {
    let g = graph(json!({"nodes":[{"id":"A"},{"id":"B"}],"edges":[{"from":"A","to":"B","label":"uses, reads"}]}));
    assert!(cell_line(&draw(&g, &Hints::default()), "e-0").contains("value=\"uses, reads\""));
}

#[test]
fn empty_document_renders_frame_and_title() {
    let doc = doc_from_json(&json!({"nodes":[{"uid":"system:s","name":"local","type":"system","attributes":{}}],"relations":[]}));
    let r = render(&doc).unwrap();
    assert!(r.xml.starts_with("<mxfile"));
    assert!(cell_line(&r.xml, "title").contains("value=\"local\""));
    assert!(r.xml.contains("<mxCell id=\"frame\""));
    assert_eq!(r.warnings, ["nothing to lay out"]);
}

#[test]
fn render_reports_violations_as_warnings() {
    let doc = doc_from_json(&serde_json::from_str(&std::fs::read_to_string(
        format!("{}/fixtures/layout-cases/rebob-container/c43.json", env!("CARGO_MANIFEST_DIR"))).unwrap()).unwrap());
    let r = render(&doc).unwrap();
    assert!(r.xml.contains("id=\"n-backend:sub-bob\""));
    assert!(r.warnings.iter().any(|w| w.starts_with("crossing: ")), "{:?}", r.warnings);
}
