//! Experiment: the icons drawn with native drawio shapes (vector in every export) instead of inline svg images.
use super::export::vertex;
use super::pretty::Icon;
use c43_layout::js::num;

const INK: &str = "#000203";

/// stroke of the artwork: 1 unit of the 10.2-unit-wide original
fn stroke(w: f64) -> f64 {
    w * 0.097
}

/// cells of an icon inside the box `(bx, by, w, h)`; the first cell has id `id`, the rest `id-<part>`
pub fn cells(icon: Icon, id: &str, bx: f64, by: f64, w: f64, h: f64) -> Vec<String> {
    let s = num(stroke(w));
    let shape = |id: &str, style: &str, x: f64, y: f64, cw: f64, ch: f64| {
        vertex(id, "", &format!("{style}fillColor=#ffffff;strokeColor={INK};strokeWidth={s};"), bx + x * w, by + y * h, cw * w, ch * h)
    };
    match icon {
        // head over a shoulders block
        Icon::User => vec![
            shape(&format!("{id}-body"), "rounded=1;arcSize=35;", 0.05, 0.535, 0.9, 0.415),
            shape(id, "ellipse;", 0.29, 0.05, 0.42, 0.43),
        ],
        // barrel
        Icon::Db => vec![shape(id, &format!("shape=cylinder3;boundedLbl=1;size={};", num(0.18 * h)), 0.05, 0.05, 0.9, 0.9)],
        // tapered barrel: tapered body under an oval rim
        Icon::Store => vec![
            shape(&format!("{id}-body"), "shape=trapezoid;perimeter=trapezoidPerimeter;flipV=1;size=0.18;", 0.05, 0.2, 0.9, 0.75),
            shape(id, "ellipse;", 0.05, 0.05, 0.9, 0.3),
        ],
        // a tube lying down: open end left
        // (a rotated cell: its geometry is the visual box turned a quarter, centred on the same point)
        Icon::Bus => {
            let (vw, vh) = (0.9 * w, 0.9 * h);
            let style = format!("shape=cylinder3;boundedLbl=1;rotation=90;strokeWidth={s};fillColor=#ffffff;strokeColor={INK};size={};", num(0.2 * vw));
            vec![vertex(id, "", &style, bx + 0.5 * w - vh / 2.0, by + 0.5 * h - vw / 2.0, vh, vw)]
        }
    }
}
