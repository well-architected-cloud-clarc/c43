//! The icons as drawio files under `media/`: native drawio shapes (vector in every export), one file per icon.
//! A file is plain drawio saved uncompressed (File > Properties > Compressed off), vertices only, everything on layer `1`.
//! Its first vertex is the icon: a transparent box with the icon's extent, whose style may carry `c43SitLine=<f>`, the
//! fraction of the box's height that hangs below the line the icon sits on (0 when absent; for a barrel the line through
//! the leftmost and rightmost points of its bottom oval). The further vertices are the parts of the icon.
use super::export::vertex;
use super::pretty::Icon;
use roxmltree::{Document, Node};

impl Icon {
    fn file(self) -> &'static str {
        match self {
            Icon::User => include_str!("../../media/user.drawio"),
            Icon::Db => include_str!("../../media/db.drawio"),
            Icon::Store => include_str!("../../media/store.drawio"),
            Icon::Bus => include_str!("../../media/bus.drawio"),
        }
    }
}

struct Cell {
    style: String,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

/// an icon read from its file
pub struct Def {
    /// size of the icon's box
    pub width: f64,
    pub height: f64,
    /// fraction of the height below the sit line
    pub below_sit_line: f64,
    cells: Vec<Cell>,
}

fn cell(n: Node) -> Option<Cell> {
    let g = n.children().find(|c| c.has_tag_name("mxGeometry"))?;
    let f = |k: &str| g.attribute(k).and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
    Some(Cell { style: n.attribute("style").unwrap_or_default().to_string(), x: f("x"), y: f("y"), w: f("width"), h: f("height") })
}

/// reads an icon file; a file that is not usable is a bug in the file, found by the tests over all icons
pub fn load(icon: Icon) -> Result<Def, String> {
    parse(icon.file())
}

/// an icon from drawio xml
pub fn parse(src: &str) -> Result<Def, String> {
    let doc = Document::parse(src).map_err(|e| format!("{e} (is the file saved uncompressed?)"))?;
    let cells: Vec<Cell> = doc.descendants().filter(|n| n.has_tag_name("mxCell") && n.attribute("vertex") == Some("1")).filter_map(cell).collect();
    let first = cells.first().ok_or("no vertex in the icon file")?;
    let sit = first.style.split(';').find_map(|kv| kv.strip_prefix("c43SitLine=")).map_or(Ok(0.0), str::parse::<f64>).map_err(|e| e.to_string())?;
    Ok(Def { width: first.w, height: first.h, below_sit_line: sit, cells })
}

impl Def {
    /// the cells of the icon moved to `(x, y)`; the first has id `id` (already escaped), the others `id-<n>`
    pub fn cells(&self, id: &str, x: f64, y: f64) -> Vec<String> {
        self.cells
            .iter()
            .enumerate()
            .map(|(i, c)| vertex(&if i == 0 { id.to_string() } else { format!("{id}-{i}") }, "", &c.style, x + c.x, y + c.y, c.w, c.h))
            .collect()
    }
}
