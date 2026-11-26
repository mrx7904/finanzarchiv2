//! A mail that is the invoice itself becomes a PDF of its text (#31): sender, date and subject on
//! top, then the text, in Helvetica on A4. Only text goes in, so nothing a mail would load from
//! outside (an image, a font, a link) is ever fetched.

use pdfium_render::prelude::{
    PdfFontToken, PdfPageObjectsCommon, PdfPagePaperSize, PdfPoints, PdfiumError,
};

use crate::pdf::Pdfium;

const HEIGHT: f32 = 842.0;
const MARGIN: f32 = 56.0;
const SIZE: f32 = 10.0;
const LEADING: f32 = 13.0;
/// Helvetica at 10 pt holds about this many characters on the width of the text.
const COLUMNS: usize = 92;

/// Keeps the characters Helvetica with WinAnsi can show, a space for any other.
fn printable(c: char) -> char {
    let shown = (c >= ' ' && (c as u32) < 0x7f)
        || ((c as u32) >= 0xa0 && (c as u32) <= 0xff)
        || "€„“”‚‘’–—…•™ŠšŽžŒœŸ".contains(c);
    if shown { c } else { ' ' }
}

/// Breaks a text into lines of at most [`COLUMNS`] characters, at spaces where it can.
pub fn wrap(text: &str) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.lines() {
        let paragraph: String = paragraph.chars().map(printable).collect();
        let paragraph = paragraph.trim_end();
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }
        let mut line = String::new();
        for word in paragraph.split(' ') {
            let mut word = word.to_owned();
            while word.chars().count() > COLUMNS {
                if !line.is_empty() {
                    lines.push(std::mem::take(&mut line));
                }
                let head: String = word.chars().take(COLUMNS).collect();
                word = word.chars().skip(COLUMNS).collect();
                lines.push(head);
            }
            let wanted =
                line.chars().count() + usize::from(!line.is_empty()) + word.chars().count();
            if wanted > COLUMNS && !line.is_empty() {
                lines.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(&word);
        }
        lines.push(line);
    }
    // More than two empty lines in a row say nothing.
    let mut kept: Vec<String> = Vec::new();
    for line in lines {
        let empties = kept.iter().rev().take_while(|line| line.is_empty()).count();
        if !(line.is_empty() && (empties >= 2 || kept.is_empty())) {
            kept.push(line);
        }
    }
    kept
}

/// The PDF of a mail: the head lines in bold, an empty line, then the text.
pub fn mail_pdf(pdfium: &Pdfium, head: &[String], text: &str) -> Result<Vec<u8>, PdfiumError> {
    let mut document = pdfium.create_new_pdf()?;
    let bold = document.fonts_mut().helvetica_bold();
    let regular = document.fonts_mut().helvetica();
    let mut lines: Vec<(String, PdfFontToken)> = Vec::new();
    for line in head {
        for wrapped in wrap(line) {
            lines.push((wrapped, bold));
        }
    }
    lines.push((String::new(), regular));
    lines.extend(wrap(text).into_iter().map(|line| (line, regular)));
    let per_page = ((HEIGHT - 2.0 * MARGIN) / LEADING) as usize;
    for chunk in lines.chunks(per_page) {
        let mut page = document
            .pages_mut()
            .create_page_at_end(PdfPagePaperSize::a4())?;
        for (row, (line, font)) in chunk.iter().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let y = HEIGHT - MARGIN - SIZE - LEADING * row as f32;
            page.objects_mut().create_text_object(
                PdfPoints::new(MARGIN),
                PdfPoints::new(y),
                line,
                *font,
                PdfPoints::new(SIZE),
            )?;
        }
    }
    document.save_to_bytes()
}
