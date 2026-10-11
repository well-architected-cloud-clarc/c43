//! drawio rendering of a C4 document through the c43-layout engine.
pub mod export;
pub mod graph;
pub mod header;
pub mod pretty;
pub mod text;

use crate::model::C4Document;
use c43_layout::model::{Lanes, Layout, Rect};
use export::{to_drawio, Drawing, Styler};
use indexmap::IndexMap;
use pretty::{Meta, PrettyHints, FRAME_STROKE};
use header::header;
use text::{label_lines, TITLE_PX, UNIT_PX};

pub struct Rendered {
    pub xml: String,
    /// relations that could not be drawn as given and layout rule violations, one line each
    pub warnings: Vec<String>,
}

/// a laid-out diagram as drawio: label lines, header band and frame are added here, at 20 px per grid unit
pub fn render_layout(l: &Layout, title: Option<&str>, description: Option<&str>) -> String {
    render_pretty(l, title, description, &IndexMap::new())
}

/// as `render_layout`, with roles and icons of nodes and groups given
pub fn render_pretty(l: &Layout, title: Option<&str>, description: Option<&str>, meta: &IndexMap<String, Meta>) -> String {
    let frame_style = format!("rounded=0;fillColor=none;strokeWidth={};", c43_layout::js::num(FRAME_STROKE));
    render_with(l, title, description, Styler::pretty(meta), &frame_style)
}

/// as `render_layout`, with the node, group and edge drawing given
pub fn render_styled(l: &Layout, title: Option<&str>, description: Option<&str>, style: Styler) -> String {
    render_with(l, title, description, style, "rounded=0;fillColor=none;")
}

fn render_with(l: &Layout, title: Option<&str>, description: Option<&str>, style: Styler, frame_style: &str) -> String {
    let head = header(title, description, l.frame.w, UNIT_PX);
    let frame = head.as_ref().map_or(l.frame, |h| Rect { x: 0.0, y: h.rect.y, w: h.rect.w, h: h.rect.h + l.frame.h });
    let node_lines = l.nodes.iter().map(|n| label_lines(&n.label, n.size * UNIT_PX).unwrap_or_else(|| vec![n.label.clone()])).collect();
    to_drawio(&Drawing { layout: l, unit_px: UNIT_PX, node_lines, header: head, frame, style, title_font: TITLE_PX, frame_style })
}

/// an empty grid: only the frame (and the title above it) is drawn
fn empty_layout() -> Layout {
    Layout {
        s: 0.0,
        cols: 0,
        rows: 0,
        lanes: Lanes { v: vec![], h: vec![] },
        groups: vec![],
        nodes: vec![],
        edges: vec![],
        frame: Rect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 },
        title: None,
        description: None,
    }
}

pub fn render(doc: &C4Document) -> Result<Rendered, String> {
    render_with_hints(doc, &PrettyHints::default())
}

/// as `render`, with the icons of nodes given
pub fn render_with_hints(doc: &C4Document, hints: &PrettyHints) -> Result<Rendered, String> {
    let mut built = graph::build(doc);
    hints.icons.iter().for_each(|(id, icon)| built.meta.entry(id.clone()).or_default().icon = Some(*icon));
    let mut warnings = built.warnings;
    let g = &built.graph;
    if g.nodes.is_empty() {
        warnings.push("nothing to lay out".into());
        let xml = render_layout(&empty_layout(), g.title.as_deref(), g.description.as_deref());
        return Ok(Rendered { xml, warnings });
    }
    let r = c43_layout::layout(g, &built.hints, &Default::default())?;
    warnings.extend(r.metrics.violations.iter().map(|v| format!("{}: {}", v.rule, v.detail)));
    Ok(Rendered { xml: render_pretty(&r.layout, g.title.as_deref(), g.description.as_deref(), &built.meta), warnings })
}
