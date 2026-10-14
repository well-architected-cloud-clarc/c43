//! drawio rendering of a C4 document through the c43-layout engine.
pub use c43_layout::drawio::{export, header, render_styled, text};
mod elms_metrics;
pub mod fonts;
pub mod graph;
pub mod icon_files;
pub mod pretty;

use crate::model::C4Document;
use c43_layout::model::{Lanes, Layout, Rect};
use c43_layout::drawio::render_fonts;
use indexmap::IndexMap;
use pretty::{Meta, PrettyHints, FRAME_STROKE};

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
    render_fonts(l, title, description, pretty::pretty_styler(meta), &frame_style, Some(&fonts::fonts()))
}

/// Sources (nodes nothing points at) tend to the left of the diagram, where the story starts. With
/// user nodes (a person, or a node drawn with the user icon) only those lean left; with more than two
/// sources and no user nodes none do: the rest sit just before what they use, so a service that only
/// uses others is not pushed to the edge. Otherwise the layout is the default one.
fn lean_sources(built: &mut graph::Built) {
    let targets: std::collections::HashSet<&str> = built.graph.edges.iter().map(|e| e.to.as_str()).collect();
    let sources: Vec<&str> = built.graph.nodes.iter().map(|n| n.id.as_str()).filter(|id| !targets.contains(id)).collect();
    let is_user = |id: &str| built.meta.get(id).is_some_and(|m| m.icon == Some(pretty::Icon::User) || m.role.as_deref() == Some(crate::roles::PERSON));
    let users: Vec<String> = sources.iter().filter(|id| is_user(id)).map(|id| id.to_string()).collect();
    if users.is_empty() && sources.len() <= 2 {
        return;
    }
    built.hints.pull_sources = true;
    built.hints.placement.extend(users.into_iter().map(|a| c43_layout::model::PlacementHint { rel: c43_layout::model::Rel::Leftmost, a, b: None, priority: false }));
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

/// as `render`, with the icons of nodes and placement suggestions given
pub fn render_with_hints(doc: &C4Document, hints: &PrettyHints) -> Result<Rendered, String> {
    let mut built = graph::build(doc);
    hints.icons.iter().for_each(|(id, icon)| built.meta.entry(id.clone()).or_default().icon = Some(*icon));
    built.hints.placement.extend(hints.placement.iter().cloned());
    lean_sources(&mut built);
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
