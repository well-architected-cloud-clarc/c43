use c43::drawio::graph::doc_from_json;
use c43::drawio::icon_files::{load, parse};
use c43::drawio::pretty::{Icon, PrettyHints};
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

/// a cell's text: attribute escaping undone, lines separated by `\n`
fn text(line: &str) -> String {
    attr(line, "value").replace("&#xa;", "\n").replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&amp;", "&")
}

/// absolute (x, y, width, height) of a cell: positions inside a group cell are relative to it
fn pos(xml: &str, id: &str) -> (f64, f64, f64, f64) {
    let l = cell(xml, id);
    let (x, y) = (num(l, "x"), num(l, "y"));
    match attr(l, "parent").as_str() {
        "1" => (x, y, num(l, "width"), num(l, "height")),
        p => {
            let (px, py, _, _) = pos(xml, p);
            (px + x, py + y, num(l, "width"), num(l, "height"))
        }
    }
}

#[test]
fn strokes_are_4_2_1() {
    let x = xml();
    assert_eq!(style_num(cell(&x, "n-api"), "strokeWidth"), 2.0);
    assert_eq!(style_num(cell(&x, "frame"), "strokeWidth"), 1.0);
    // group frames weigh like the outer border, twice a connection
    assert_eq!(style_num(cell(&x, "g-g"), "strokeWidth"), 1.0);
    let edge = x.lines().find(|l| l.contains("edge=\"1\"")).unwrap();
    assert_eq!(style_num(edge, "strokeWidth"), 0.5);
}

#[test]
fn nodes_stay_squares() {
    let x = xml();
    ["n-user", "n-api", "n-bus"].iter().for_each(|id| assert_eq!(num(cell(&x, id), "width"), num(cell(&x, id), "height"), "{id}"));
}

#[test]
fn node_text_is_a_name_cell_then_a_role_cell_in_brackets() {
    let x = xml();
    let (name, role) = (cell(&x, "nt-api"), cell(&x, "nr-api"));
    assert_eq!(text(name), "Greeting\nsystem");
    assert_eq!(text(role), "[ software\nsystem ]");
    assert!(attr(name, "style").contains("fontFamily=Elms Sans;") && attr(role, "style").contains("fontFamily=Elms Sans Thin;"));
    // plain text, no html: it exports as text, not as an image
    assert!(!x.contains("html=1"));
    // the role is under the name, both centred in the square
    let (sq, n, r) = (pos(&x, "n-api"), pos(&x, "nt-api"), pos(&x, "nr-api"));
    assert!(r.1 >= n.1 + n.3 - 1e-9);
    assert!(((n.1 + (r.1 + r.3 - n.1) / 2.0) - (sq.1 + sq.3 / 2.0)).abs() < 1e-6);
}

#[test]
fn node_without_a_given_role_shows_its_type() {
    let x = xml();
    assert_eq!(text(cell(&x, "nt-bus")), "Events");
    assert_eq!(text(cell(&x, "nr-bus")), "[ eventbus ]");
}

#[test]
fn a_node_is_one_group_holding_its_square_text_and_icon() {
    let x = xml();
    let g = cell(&x, "ng-user");
    assert!(attr(g, "style").contains("group"));
    ["n-user", "nt-user", "nr-user", "i-user", "i-user-1"].iter().for_each(|id| assert_eq!(attr(cell(&x, id), "parent"), "ng-user", "{id}"));
    // the group spans the square and the icon hanging off its corner
    let (gx, gy, gw, gh) = pos(&x, "ng-user");
    let (sx, sy, sw, sh) = pos(&x, "n-user");
    let (ix, iy, _, _) = pos(&x, "i-user");
    assert_eq!((gx, gy), (ix.min(sx), iy.min(sy)));
    assert_eq!((gx + gw, gy + gh), (sx + sw, sy + sh));
    // a node without an icon: the group is the square
    assert_eq!(pos(&x, "ng-api"), pos(&x, "n-api"));
    // edges still end on the square
    assert!(x.contains("target=\"n-api\"") && !x.contains("target=\"ng-"));
}

#[test]
fn group_text_is_bottom_left_inside_the_group() {
    let x = xml();
    let (g, n, r) = (pos(&x, "g-g"), pos(&x, "gt-g"), pos(&x, "gr-g"));
    assert_eq!((text(cell(&x, "gt-g")).as_str(), text(cell(&x, "gr-g")).as_str()), ("Hello Inc.", "[ enterprise boundary ]"));
    assert!(attr(cell(&x, "gt-g"), "style").contains("align=left") && attr(cell(&x, "gr-g"), "style").contains("align=left"));
    assert!(n.0 > g.0 && n.0 + n.2 <= g.0 + g.2);
    // the role is under the name, the block ends above the frame's bottom line, in the bottom half
    assert!(r.1 >= n.1 + n.3 - 1e-9);
    assert!(r.1 + r.3 < g.1 + g.3);
    assert!(n.1 > g.1 + g.3 / 2.0);
    // frame and text are one drawio group
    assert_eq!((attr(cell(&x, "g-g"), "parent"), attr(cell(&x, "gt-g"), "parent")), ("gg-g".to_string(), "gg-g".to_string()));
}

#[test]
fn icons_sit_on_the_top_edge_at_the_left_corner_half_outside() {
    let x = xml();
    ["user", "store", "bus"].iter().for_each(|id| {
        let (n, i) = (pos(&x, &format!("n-{id}")), pos(&x, &format!("i-{id}")));
        let icon_cell = cell(&x, &format!("i-{id}"));
        // native drawio shapes (vector in every export), no embedded images
        assert!(attr(icon_cell, "style").contains("strokeColor=none"), "{id}: the first cell is the icon's box");
        assert!(x.contains(&format!("id=\"i-{id}-1\"")), "{id}: parts follow the box");
        assert!((i.0 + i.2 / 2.0 - n.0).abs() < 1e-9, "{id}");
        // the cell is the artwork's size: no empty margin below the sit line
        assert!(i.3 < i.2, "{id}");
        // the sit line (the icon's bottom, half the node outline plus half the icon stroke below the node's top edge, so the icon's bottom line lies on the inner edge of the outline), except for a barrel: its bottom oval hangs below the line through the oval's extremes
        let hang = if *id == "store" { 0.1 * i.3 } else { 0.0 };
        assert!((i.1 + i.3 - hang - n.1 - (2.0 + 1.55) / 2.0).abs() < 1e-6, "{id}");
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

#[test]
fn icons_are_native_shapes_not_images() {
    let x = xml();
    assert!(!x.contains("data:image") && !x.contains("shape=image"));
    // one cell per part: the box and the shapes of the icon file
    assert!(x.matches("id=\"i-user").count() >= 3);
}

#[test]
fn every_icon_file_loads_and_has_a_box_and_parts() {
    [Icon::User, Icon::Db, Icon::Store, Icon::Bus].iter().for_each(|i| {
        let d = load(*i).unwrap_or_else(|e| panic!("{i:?}: {e}"));
        assert!(d.width > 0.0 && d.height > 0.0 && (0.0..0.5).contains(&d.below_sit_line), "{i:?}");
        assert!(d.cells("x", 0.0, 0.0).len() >= 2, "{i:?}");
    });
    // only the barrels hang below their sit line
    assert!(load(Icon::Db).unwrap().below_sit_line > 0.0 && load(Icon::Store).unwrap().below_sit_line > 0.0);
    assert_eq!((load(Icon::User).unwrap().below_sit_line, load(Icon::Bus).unwrap().below_sit_line), (0.0, 0.0));
}

#[test]
fn an_icon_file_saved_by_the_drawio_editor_parses() {
    // pretty-printed, attributes in the editor's order, geometry on its own line, extra cells and attributes
    let saved = r##"<mxfile host="app.diagrams.net" modified="2026-10-11T12:00:00.000Z" agent="x" version="24.0.0">
  <diagram id="d" name="Page-1">
    <mxGraphModel dx="800" dy="600" grid="1" gridSize="10" guides="1" tooltips="1" connect="1" arrows="1" fold="1" page="1" pageScale="1" pageWidth="827" pageHeight="1169" math="0" shadow="0">
      <root>
        <mxCell id="0" />
        <mxCell id="1" parent="0" />
        <mxCell id="icon" parent="1" style="fillColor=none;strokeColor=none;c43SitLine=0.25;" value="" vertex="1">
          <mxGeometry height="20" width="40" x="10" y="5" as="geometry" />
        </mxCell>
        <mxCell id="a" parent="1" style="ellipse;" value="" vertex="1">
          <mxGeometry height="8" width="8" x="12" y="6" as="geometry" />
        </mxCell>
      </root>
    </mxGraphModel>
  </diagram>
</mxfile>"##;
    let d = parse(saved).unwrap();
    assert_eq!((d.width, d.height, d.below_sit_line), (40.0, 20.0, 0.25));
    let cells = d.cells("i-n", 100.0, 200.0);
    assert_eq!(cells.len(), 2);
    assert!(cells[1].contains("id=\"i-n-1\"") && cells[1].contains("x=\"112\"") && cells[1].contains("y=\"206\""));
}

#[test]
fn a_compressed_icon_file_is_reported_not_misread() {
    let compressed = r#"<mxfile><diagram id="d" name="p">dZHBDoIwDIafZvcxFbgbBA+evHgZ</diagram></mxfile>"#;
    assert!(parse(compressed).is_err());
}

/// every text is set in Elms Sans: names in weight 400, the rest in weight 100; no bold style fakes a heavier face
#[test]
fn all_text_is_elms_sans_and_none_is_bold() {
    let x = xml();
    let texts: Vec<&str> = x.lines().filter(|l| l.contains("style=\"text;") && !l.contains("fontSource")).collect();
    assert!(texts.len() > 6);
    texts.iter().for_each(|l| assert!(l.contains("fontFamily=Elms Sans;") || l.contains("fontFamily=Elms Sans Thin;"), "{l}"));
    assert!(!x.contains("fontStyle"));
    ["nt-api", "gt-g"].iter().for_each(|id| assert!(cell(&x, id).contains("fontFamily=Elms Sans;"), "{id}"));
    ["nr-api", "gr-g"].iter().for_each(|id| assert!(cell(&x, id).contains("fontFamily=Elms Sans Thin;"), "{id}"));
    let edge = x.lines().find(|l| l.contains("edge=\"1\"")).unwrap();
    assert!(edge.contains("fontFamily=Elms Sans Thin;"));
}

/// the font data is in the document once per family, as a woff2 data URI (not once per text cell)
#[test]
fn font_data_is_embedded_once_per_family() {
    let x = xml();
    assert_eq!(x.matches("fontSource=").count(), 2);
    let carrier = |id: &str, family: &str| {
        let l = cell(&x, id);
        assert!(l.contains(&format!("fontFamily={family};")) && l.contains("fontSource=data%3Afont/woff2%3Bbase64%2C"), "{id}");
    };
    carrier("font-regular", "Elms Sans Thin");
    carrier("font-name", "Elms Sans");
    // the two files differ
    let src = |id: &str| attr(cell(&x, id), "style").split("fontSource=").nth(1).unwrap().to_string();
    assert_ne!(src("font-regular"), src("font-name"));
    // attribute-safe: the data carries nothing that separates style parts or ends the attribute
    assert!(!src("font-name").trim_end_matches(';').contains([';', '"', '+', '=']));
}

/// the width estimate is Elms Sans's: a text is never wider than its node's inner box
#[test]
fn node_text_fits_the_square_with_the_elms_sans_widths() {
    use c43::drawio::fonts::{NAME_FACE, REGULAR_FACE};
    use c43_layout::drawio::text::{text_width_in, LABEL_PX, PAD};
    let x = xml();
    ["api", "store", "user", "bus"].iter().for_each(|id| {
        let side = num(cell(&x, &format!("n-{id}")), "width");
        [("nt-", &NAME_FACE), ("nr-", &REGULAR_FACE)].iter().filter(|(p, _)| x.contains(&format!("id=\"{p}{id}\""))).for_each(|(p, f)| {
            text(cell(&x, &format!("{p}{id}"))).lines().for_each(|line| assert!(text_width_in(line, LABEL_PX, f) <= side - 2.0 * PAD + 1e-9, "{id}: {line}"));
        });
    });
}
