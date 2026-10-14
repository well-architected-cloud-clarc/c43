//! The Pretty C4 look: outlined square nodes with centered name/role text, icons sitting on the node's top-left corner, barrels for data stores.
use c43_layout::drawio::export::{esc, vertex_in, EdgeDraw, Styler};
use super::fonts::{self, NAME_FACE, REGULAR_FACE};
use crate::roles;
use c43_layout::drawio::text::{text_width_in, wrap_in, Face, LABEL_PX, LINE_H, PAD};
use c43_layout::js::num;
use c43_layout::model::{Kind, PlacementHint};
use indexmap::IndexMap;
use serde::Deserialize;

/// connection line width in px; a node outline is 4x, the outer diagram border 2x
pub const LINE: f64 = 0.5;
pub const NODE_STROKE: f64 = 4.0 * LINE;
/// the outer diagram border and group frames: one "frame" weight, twice a connection so that connections stay distinct
pub const FRAME_STROKE: f64 = 2.0 * LINE;
/// an edge label wider than this is split into lines: about the width of the lane an edge runs in between two nodes
pub const EDGE_LABEL_PX: f64 = 48.0;
/// stroke of the icon parts in the icon files (px): the icon's bottom line lies fully inside the node, on the inner edge of its outline
pub const ICON_STROKE: f64 = 1.55;
/// smallest node side in grid units: room for the hanging icon
const MIN_SIDE: f64 = 5.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Icon {
    User,
    Db,
    Store,
    Bus,
}

impl Icon {
    /// the icon a role is drawn with, if it has one
    pub fn of_role(role: &str) -> Option<Icon> {
        match role {
            roles::PERSON => Some(Icon::User),
            roles::DATABASE => Some(Icon::Db),
            roles::OBJECT_STORE => Some(Icon::Store),
            roles::EVENT_BUS => Some(Icon::Bus),
            _ => None,
        }
    }
}

/// per-case hints, from a hints file: icons by node id, overriding the ones roles give, and
/// placement suggestions for the layout (`a` above `b`, ...; ids are node uids)
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PrettyHints {
    #[serde(default)]
    pub icons: IndexMap<String, Icon>,
    #[serde(default)]
    pub placement: Vec<PlacementHint>,
}

/// what the look needs to know about a node or a group besides its name
#[derive(Debug, Clone, Default)]
pub struct Meta {
    pub role: Option<String>,
    pub icon: Option<Icon>,
}

/// a line of node or group text; the name is bold
struct Line {
    text: String,
    bold: bool,
}

/// how wide the text of a name (bold) or a regular line is set
fn face(bold: bool) -> &'static Face {
    if bold { &NAME_FACE } else { &REGULAR_FACE }
}

fn role_text(role: &str) -> String {
    format!("[ {role} ]")
}

/// name and role lines wrapped to `inner_px`; None when a word does not fit
fn text_lines(name: &str, role: Option<&str>, inner_px: f64) -> Option<Vec<Line>> {
    let part = |t: &str, bold: bool| wrap_in(t, inner_px, LABEL_PX, face(bold)).map(|ls| ls.into_iter().map(|text| Line { text, bold }).collect::<Vec<_>>());
    let mut lines = part(name, true)?;
    lines.extend(role.map_or(Some(vec![]), |r| part(&role_text(r), false))?);
    Some(lines)
}

/// smallest node side in grid units that holds the name and the role
pub fn node_side(name: &str, role: Option<&str>, unit_px: f64) -> f64 {
    let fits = |s: f64| text_lines(name, role, s * unit_px - 2.0 * PAD).is_some_and(|ls| ls.len() as f64 * LABEL_PX * LINE_H <= s * unit_px - 2.0 * PAD);
    std::iter::successors(Some(MIN_SIDE), |s| Some(s + 1.0)).find(|&s| fits(s)).unwrap_or(MIN_SIDE)
}

/// widest line of a group's text in px, for the engine to keep the group at least that wide
pub fn group_text_width(name: &str, role: Option<&str>) -> f64 {
    let w = |t: &str, bold: bool| text_width_in(t, LABEL_PX, face(bold));
    w(name, true).max(role.map_or(0.0, |r| w(&role_text(r), false))) + LABEL_PX / 2.0
}

/// plain text value of lines, escaped for an attribute, lines separated by the xml newline
fn text_value(lines: &[&Line]) -> String {
    lines.iter().map(|l| esc(&l.text)).collect::<Vec<_>>().join("&#xa;")
}

fn text_style(align: &str, bold: bool) -> String {
    format!("text;html=0;whiteSpace=nowrap;align={align};verticalAlign=middle;spacing=0;{}fontSize={};", fonts::style(bold), num(LABEL_PX))
}

/// the name cell (bold) and, when there are role lines, the role cell below it, stacked from `top` (absolute) in a block `w` wide at `x`;
/// positions are relative to the group cell at `(gx, gy)`
#[allow(clippy::too_many_arguments)]
fn text_cells(ids: (&str, &str), parent: &str, lines: &[Line], align: &str, x: f64, top: f64, w: f64, (gx, gy): (f64, f64)) -> Vec<String> {
    let lh = LABEL_PX * LINE_H;
    let (name, role): (Vec<&Line>, Vec<&Line>) = lines.iter().partition(|l| l.bold);
    let name_h = name.len() as f64 * lh;
    std::iter::once(vertex_in(ids.0, parent, &text_value(&name), &text_style(align, true), x - gx, top - gy, w, name_h))
        .chain((!role.is_empty()).then(|| vertex_in(ids.1, parent, &text_value(&role), &text_style(align, false), x - gx, top + name_h - gy, w, role.len() as f64 * lh)))
        .collect()
}

/// the Pretty C4 look; `meta` is keyed by node and group id, unknown ids get plain text and no icon
pub fn pretty_styler(meta: &IndexMap<String, Meta>) -> Styler<'_> {
    let meta_of = move |id: &str| meta.get(id).cloned().unwrap_or_default();
    Styler {
        node: Box::new(move |n, _, u| {
            let m = meta_of(&n.id);
            let (side, nx, ny) = (n.size * u, n.x * u, n.y * u);
            let lines = text_lines(&n.label, m.role.as_deref(), side - 2.0 * PAD).unwrap_or_else(|| vec![Line { text: n.label.clone(), bold: true }]);
            // the icon sits on top of the node, its sit line overlapping the node's outline (the outline being centred on the edge), with half of its width outside on the left
            let icon = m.icon.map(|i| {
                let d = super::icon_files::load(i).expect("icon file");
                let at = (nx - d.width / 2.0, ny + (NODE_STROKE + ICON_STROKE) / 2.0 - d.height * (1.0 - d.below_sit_line));
                (d, at)
            });
            // one drawio group per node: the square, its text and its icon move together
            let (gx, gy) = icon.as_ref().map_or((nx, ny), |(_, (ix, iy))| (ix.min(nx), iy.min(ny)));
            let gid = format!("ng-{}", esc(&n.id));
            let id = esc(&n.id);
            let style = format!("fillColor=#ffffff;rounded=0;strokeWidth={};", num(NODE_STROKE));
            let top = ny + (side - lines.len() as f64 * LABEL_PX * LINE_H) / 2.0;
            std::iter::once(vertex_in(&gid, "1", "", "group;", gx, gy, nx + side - gx, ny + side - gy))
                .chain(std::iter::once(vertex_in(&format!("n-{id}"), &gid, "", &style, nx - gx, ny - gy, side, side)))
                .chain(text_cells((&format!("nt-{id}"), &format!("nr-{id}")), &gid, &lines, "center", nx, top, side, (gx, gy)))
                .chain(icon.iter().flat_map(|(d, (ix, iy))| d.cells_in(&gid, &format!("i-{id}"), ix - gx, iy - gy)))
                .collect()
        }),
        group: Box::new(move |g, u| {
            let m = meta_of(&g.id);
            let lines = std::iter::once(Line { text: g.label.clone(), bold: true }).chain(m.role.map(|r| Line { text: role_text(&r), bold: false })).collect::<Vec<_>>();
            // the engine keeps a band for the text at the bottom of the group; the text is left aligned and sits at the bottom of that band
            let (gx, gy, t, id) = (g.x * u, g.y * u, &g.title, esc(&g.id));
            let gid = format!("gg-{id}");
            let top = (t.y + t.h) * u - lines.len() as f64 * LABEL_PX * LINE_H;
            std::iter::once(vertex_in(&gid, "1", "", "group;", gx, gy, g.w * u, g.h * u))
                .chain(std::iter::once(vertex_in(&format!("g-{id}"), &gid, "", &format!("rounded=0;fillColor=none;strokeWidth={};", num(FRAME_STROKE)), 0.0, 0.0, g.w * u, g.h * u)))
                .chain(text_cells((&format!("gt-{id}"), &format!("gr-{id}")), &gid, &lines, "left", t.x * u, top, t.w * u, (gx, gy)))
                .collect()
        }),
        edge: Box::new(|e| EdgeDraw {
            // the label sits above its line, not on it
            style: format!("strokeWidth={};verticalAlign=bottom;{}{}", num(LINE), fonts::style(false), if e.kind == Kind::Nf { "dashed=1;" } else { "" }),
            moved: None,
            straight: false,
        }),
        label_font: LABEL_PX,
    }
}
