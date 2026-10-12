//! Elms Sans, the font of all diagram text: weight 100 for regular text, weight 400 for names. The latin woff2 files under
//! `media/fonts` are compiled in and travel inside the drawio document (a `fontSource` data URI), so exports need no installed font.
use super::elms_metrics::{NAME, THIN};
use base64::{engine::general_purpose::STANDARD, Engine};
use c43_layout::drawio::export::{Font, Fonts};
use c43_layout::drawio::text::Face;
use std::sync::OnceLock;

pub const REGULAR_FAMILY: &str = "Elms Sans Thin";
pub const NAME_FAMILY: &str = "Elms Sans";

/// width of anything the table does not cover, in em
const FALLBACK_EM: f64 = 0.6;

fn em(table: &[u16; 95], ch: char) -> f64 {
    u32::from(ch).checked_sub(32).and_then(|i| table.get(i as usize)).map_or(FALLBACK_EM, |w| f64::from(*w) / 1000.0)
}

pub const REGULAR_FACE: Face = Face { em: |ch| em(&THIN, ch), factor: 1.0 };
pub const NAME_FACE: Face = Face { em: |ch| em(&NAME, ch), factor: 1.0 };

/// the url-encoded `fontSource` value for a woff2 file
fn source(woff2: &[u8]) -> String {
    format!("data%3Afont/woff2%3Bbase64%2C{}", STANDARD.encode(woff2).replace('+', "%2B").replace('/', "%2F").replace('=', "%3D"))
}

/// the two fonts as the renderer wants them (the font data is encoded once)
pub fn fonts() -> Fonts<'static> {
    static SOURCES: OnceLock<(String, String)> = OnceLock::new();
    let (thin, name) = SOURCES.get_or_init(|| (source(include_bytes!("../../media/fonts/elms-sans-latin-100-normal.woff2")), source(include_bytes!("../../media/fonts/elms-sans-latin-400-normal.woff2"))));
    Fonts {
        regular: Font { family: REGULAR_FAMILY, source: thin, face: REGULAR_FACE },
        name: Font { family: NAME_FAMILY, source: name, face: NAME_FACE },
    }
}

/// the style part that sets a text cell in the regular or in the name font
pub fn style(bold: bool) -> String {
    format!("fontFamily={};", if bold { NAME_FAMILY } else { REGULAR_FAMILY })
}
