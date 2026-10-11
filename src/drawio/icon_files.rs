//! The icons as drawio files under `media/`: native drawio shapes (vector in every export), one file per icon,
//! drawn in a box 32 px wide at the origin. The first cell of a file is the icon, further cells are its parts.
use super::export::vertex;
use super::pretty::Icon;

/// width of the box the icon files are drawn in
pub const DESIGN_PX: f64 = 32.0;

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

fn attr<'a>(line: &'a str, k: &str) -> Option<&'a str> {
    line.split(&format!(" {k}=\"")).nth(1)?.split('"').next()
}

/// the cells of an icon moved to `(x, y)`; the first has id `id` (already escaped), the others `id-<n>`
pub fn cells(icon: Icon, id: &str, x: f64, y: f64) -> Vec<String> {
    let f = |line: &str, k: &str| attr(line, k).and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
    icon.file()
        .lines()
        .filter(|l| l.contains("vertex=\"1\""))
        .enumerate()
        .map(|(i, l)| {
            let cell_id = if i == 0 { id.to_string() } else { format!("{id}-{i}") };
            let style = attr(l, "style").unwrap_or_default();
            vertex(&cell_id, "", style, x + f(l, "x"), y + f(l, "y"), f(l, "width"), f(l, "height"))
        })
        .collect()
}

/// height of the icon's box in the file
pub fn height(icon: Icon) -> f64 {
    icon.file().lines().find(|l| l.contains("vertex=\"1\"")).and_then(|l| attr(l, "height")?.parse().ok()).unwrap_or(DESIGN_PX)
}
