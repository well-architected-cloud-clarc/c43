//! The Pretty C4 look: outlined square nodes with centered name/role text, icons sitting on the node's top-left corner, barrels for data stores.
use super::export::{esc, vertex, EdgeDraw, Styler};
use super::text::{wrap, LABEL_PX, LINE_H, PAD};
use c43_layout::js::num;
use c43_layout::model::Kind;
use indexmap::IndexMap;
use serde::Deserialize;

/// connection line width in px; a node outline is 4x, the outer diagram border 2x
pub const LINE: f64 = 1.0;
pub const NODE_STROKE: f64 = 4.0 * LINE;
pub const FRAME_STROKE: f64 = 2.0 * LINE;
/// smallest node side in grid units: room for the hanging icon
const MIN_SIDE: f64 = 5.0;
const ICON_PX: f64 = 32.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Icon {
    User,
    Db,
    Store,
    Bus,
}

impl Icon {
    /// fraction of the icon's height below its sit line: the bottom oval of a barrel hangs below the line joining the oval's leftmost and rightmost points (1.0 / 9.95 in `media/icons.svg`)
    fn below_sit_line(self) -> f64 {
        match self {
            Icon::Db | Icon::Store => 0.1005,
            Icon::User | Icon::Bus => 0.0,
        }
    }

    /// height over width of the icon's artwork (its svg viewBox), so the cell has no empty margin and the sit line is where it is drawn
    fn aspect(self) -> f64 {
        match self {
            Icon::User => 9.718853 / 10.149083,
            Icon::Db | Icon::Store => 9.9476194 / 10.2925,
            Icon::Bus => 9.7866592 / 10.270384,
        }
    }

    fn svg(self) -> &'static str {
        match self {
            Icon::User => include_str!("icons/user.svg"),
            Icon::Db => include_str!("icons/db.svg"),
            Icon::Store => include_str!("icons/bucket.svg"),
            Icon::Bus => include_str!("icons/pipe.svg"),
        }
    }
}

/// per-case hints for the look, from a hints file: the icon of nodes by id (until the c43 output carries it)
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PrettyHints {
    #[serde(default)]
    pub icons: IndexMap<String, Icon>,
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

fn role_text(role: &str) -> String {
    format!("[ {role} ]")
}

/// name and role lines wrapped to `inner_px`; None when a word does not fit
fn text_lines(name: &str, role: Option<&str>, inner_px: f64) -> Option<Vec<Line>> {
    let part = |t: &str, bold: bool| wrap(t, inner_px, LABEL_PX, bold).map(|ls| ls.into_iter().map(|text| Line { text, bold }).collect::<Vec<_>>());
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
    let w = |t: &str, bold: bool| super::text::text_width(t, LABEL_PX, bold);
    w(name, true).max(role.map_or(0.0, |r| w(&role_text(r), false))) + LABEL_PX / 2.0
}

fn html(lines: &[Line]) -> String {
    let body = lines.iter().map(|l| if l.bold { format!("<b>{}</b>", esc(&l.text)) } else { esc(&l.text) }).collect::<Vec<_>>().join("<br>");
    esc(&body)
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64(data: &[u8]) -> String {
    data.chunks(3)
        .flat_map(|c| {
            let n = c.iter().enumerate().fold(0u32, |n, (i, b)| n | (*b as u32) << (16 - 8 * i));
            (0..=c.len()).map(move |i| B64[(n >> (18 - 6 * i) & 63) as usize] as char).chain(std::iter::repeat('=').take(3 - c.len()))
        })
        .collect()
}

/// drawio image style of an icon: a data URI without `;base64` (the style syntax uses `;`)
fn icon_style(icon: Icon) -> String {
    format!("shape=image;aspect=fixed;imageAspect=0;image=data:image/svg+xml,{};", base64(icon.svg().as_bytes()))
}

impl Styler<'_> {
    /// the Pretty C4 look; `meta` is keyed by node and group id, unknown ids get plain text and no icon
    pub fn pretty(meta: &IndexMap<String, Meta>) -> Styler<'_> {
        let meta_of = move |id: &str| meta.get(id).cloned().unwrap_or_default();
        Styler {
            node: Box::new(move |n, _, u| {
                let m = meta_of(&n.id);
                let side = n.size * u;
                let inner = side - 2.0 * PAD;
                let lines = text_lines(&n.label, m.role.as_deref(), inner).unwrap_or_else(|| vec![Line { text: n.label.clone(), bold: true }]);
                let style = format!("html=1;whiteSpace=nowrap;fontSize={};strokeWidth={};fillColor=#ffffff;rounded=0;", num(LABEL_PX), num(NODE_STROKE));
                let node = vertex(&format!("n-{}", esc(&n.id)), &html(&lines), &style, n.x * u, n.y * u, side, side);
                // the icon sits on top of the node, its sit line overlapping the node's outline (its half width, the outline being centred on the edge), with half of its width outside on the left
                let icon = m.icon.map_or(vec![], |i| {
                    let h = ICON_PX * i.aspect();
                    let (x, y) = (n.x * u - ICON_PX / 2.0, n.y * u + NODE_STROKE / 2.0 - h * (1.0 - i.below_sit_line()));
                    // experiment: C43_NATIVE_ICONS=1 draws the icons with native drawio shapes
                    if std::env::var("C43_NATIVE_ICONS").is_ok() {
                        super::native_icons::cells(i, &format!("i-{}", esc(&n.id)), x, y, ICON_PX, h)
                    } else {
                        vec![vertex(&format!("i-{}", esc(&n.id)), "", &icon_style(i), x, y, ICON_PX, h)]
                    }
                });
                std::iter::once(node).chain(icon).collect()
            }),
            group: Box::new(move |g, u| {
                let m = meta_of(&g.id);
                let lines = std::iter::once(Line { text: g.label.clone(), bold: true }).chain(m.role.map(|r| Line { text: role_text(&r), bold: false })).collect::<Vec<_>>();
                // the engine keeps a band for the text at the bottom of the group; the text is left aligned at the bottom of its block
                let text_style = format!("text;html=1;whiteSpace=nowrap;align=left;verticalAlign=bottom;spacing=0;fontSize={};", num(LABEL_PX));
                let t = &g.title;
                vec![
                    vertex(&format!("g-{}", esc(&g.id)), "", &format!("rounded=0;fillColor=none;strokeWidth={};", num(LINE)), g.x * u, g.y * u, g.w * u, g.h * u),
                    vertex(&format!("gt-{}", esc(&g.id)), &html(&lines), &text_style, t.x * u, t.y * u, t.w * u, t.h * u),
                ]
            }),
            edge: Box::new(|e| EdgeDraw {
                style: format!("strokeWidth={};{}", num(LINE), if e.kind == Kind::Nf { "dashed=1;" } else { "" }),
                moved: None,
                straight: false,
            }),
            label_font: LABEL_PX,
        }
    }
}
