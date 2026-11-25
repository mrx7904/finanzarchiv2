//! The text of every page with the place of each line, and pages as images (FA-10).

use std::io::Cursor;

use image::{ImageFormat, imageops::FilterType};
use pdfium_render::prelude::{PdfRenderConfig, Pdfium};

/// One line of text and its box on the page, in PDF points from the lower left corner.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub text: String,
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
    pub top: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageText {
    /// From 1.
    pub number: u32,
    pub width: f32,
    pub height: f32,
    /// Top to bottom.
    pub lines: Vec<Line>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PageError {
    #[error("Die Datei ließ sich nicht öffnen")]
    Unreadable,
    #[error("Die Seite gibt es nicht")]
    NoSuchPage,
    #[error("Das Bild ließ sich nicht erzeugen")]
    Image,
}

/// Every page of a PDF with its lines.
pub fn read(pdfium: &Pdfium, bytes: &[u8]) -> Result<Vec<PageText>, PageError> {
    let document = pdfium
        .load_pdf_from_byte_slice(bytes, None)
        .map_err(|_| PageError::Unreadable)?;
    let mut pages = Vec::new();
    for (index, page) in document.pages().iter().enumerate() {
        let text = page.text().map_err(|_| PageError::Unreadable)?;
        let segments = text
            .segments()
            .iter()
            .map(|segment| {
                let bounds = segment.bounds();
                Line {
                    text: segment.text(),
                    left: bounds.left().value,
                    bottom: bounds.bottom().value,
                    right: bounds.right().value,
                    top: bounds.top().value,
                }
            })
            .collect();
        pages.push(PageText {
            number: u32::try_from(index + 1).unwrap_or(u32::MAX),
            width: page.width().value,
            height: page.height().value,
            lines: group_lines(segments),
        });
    }
    Ok(pages)
}

/// A PDF whose pages carry no text at all is a scan, to be read as an image in stage 2.
pub fn is_scan(pages: &[PageText]) -> bool {
    pages.iter().all(|page| page.lines.is_empty())
}

/// PDFium gives runs of text. Runs whose middle lies within the height of a line belong to it,
/// left to right, with a space where they stand apart.
fn group_lines(mut segments: Vec<Line>) -> Vec<Line> {
    segments.retain(|segment| !segment.text.trim().is_empty());
    segments.sort_by(|a, b| b.top.total_cmp(&a.top).then(a.left.total_cmp(&b.left)));
    let mut lines: Vec<Vec<Line>> = Vec::new();
    for segment in segments {
        let middle = (segment.top + segment.bottom) / 2.0;
        match lines.iter_mut().find(|line| {
            line.first()
                .is_some_and(|first| middle <= first.top && middle >= first.bottom)
        }) {
            Some(line) => line.push(segment),
            None => lines.push(vec![segment]),
        }
    }
    lines.into_iter().filter_map(join).collect()
}

fn join(mut runs: Vec<Line>) -> Option<Line> {
    runs.sort_by(|a, b| a.left.total_cmp(&b.left));
    let first = runs.first()?.clone();
    let mut line = Line {
        text: String::new(),
        ..first
    };
    for run in &runs {
        let gap = run.left - line.right;
        if !line.text.is_empty() && gap > (run.top - run.bottom) * 0.2 {
            line.text.push(' ');
        }
        line.text.push_str(run.text.trim_end_matches(['\r', '\n']));
        line.left = line.left.min(run.left);
        line.right = line.right.max(run.right);
        line.bottom = line.bottom.min(run.bottom);
        line.top = line.top.max(run.top);
    }
    line.text = line.text.trim().to_owned();
    Some(line)
}

/// A page of a PDF as a PNG of the given width.
pub fn render_pdf_page(
    pdfium: &Pdfium,
    bytes: &[u8],
    number: u32,
    width: u32,
) -> Result<Vec<u8>, PageError> {
    let document = pdfium
        .load_pdf_from_byte_slice(bytes, None)
        .map_err(|_| PageError::Unreadable)?;
    let index = i32::try_from(number.checked_sub(1).ok_or(PageError::NoSuchPage)?)
        .map_err(|_| PageError::NoSuchPage)?;
    let page = document
        .pages()
        .get(index)
        .map_err(|_| PageError::NoSuchPage)?;
    let config = PdfRenderConfig::new()
        .set_target_width(i32::try_from(width).map_err(|_| PageError::Image)?);
    let image = page
        .render_with_config(&config)
        .and_then(|bitmap| bitmap.as_image())
        .map_err(|_| PageError::Image)?;
    png(&image)
}

/// An image file (PNG, JPEG, WebP) scaled to the given width, as a PNG.
pub fn render_image(bytes: &[u8], width: u32) -> Result<Vec<u8>, PageError> {
    let image = image::load_from_memory(bytes).map_err(|_| PageError::Unreadable)?;
    let height =
        (u64::from(image.height()) * u64::from(width) / u64::from(image.width().max(1))).max(1);
    png(&image.resize(
        width,
        u32::try_from(height).unwrap_or(u32::MAX),
        FilterType::Triangle,
    ))
}

fn png(image: &image::DynamicImage) -> Result<Vec<u8>, PageError> {
    let mut bytes = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .map_err(|_| PageError::Image)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing;

    fn png_width(bytes: &[u8]) -> u32 {
        u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]])
    }

    #[test]
    fn finds_every_printed_amount_with_its_line_from_top_to_bottom() {
        let invoice = testing::pdf(&[
            "Musterfirma Beispiel GmbH",
            "Rechnung Nr. 2026-0042",
            "Netto 1.234,56 EUR",
            "Umsatzsteuer 19 % 234,57 EUR",
            "Brutto 1.469,13 EUR",
            "Fällig am 15.12.2026",
        ]);
        let pages = read(testing::pdfium(), &invoice).unwrap();
        assert_eq!(pages.len(), 1);
        let page = &pages[0];
        assert_eq!((page.width.round(), page.height.round()), (595.0, 842.0));
        let texts: Vec<&str> = page.lines.iter().map(|line| line.text.as_str()).collect();
        assert_eq!(
            texts,
            [
                "Musterfirma Beispiel GmbH",
                "Rechnung Nr. 2026-0042",
                "Netto 1.234,56 EUR",
                "Umsatzsteuer 19 % 234,57 EUR",
                "Brutto 1.469,13 EUR",
                "Fällig am 15.12.2026"
            ]
        );
        for amount in ["1.234,56", "234,57", "1.469,13"] {
            let line = page
                .lines
                .iter()
                .find(|line| line.text.contains(amount))
                .unwrap();
            assert!(
                line.left >= 59.0 && line.left <= 61.0,
                "{amount} starts at {}",
                line.left
            );
            assert!(line.top > line.bottom && line.right > line.left);
        }
        assert!(
            page.lines.windows(2).all(|pair| pair[0].top > pair[1].top),
            "lines run top to bottom"
        );
        assert!(!is_scan(&pages));
    }

    #[test]
    fn a_page_without_text_is_a_scan() {
        let pages = read(testing::pdfium(), &testing::pdf_without_text()).unwrap();
        assert_eq!(pages.len(), 1);
        assert!(pages[0].lines.is_empty());
        assert!(is_scan(&pages));
    }

    #[test]
    fn runs_on_one_line_are_joined_left_to_right() {
        let run = |text: &str, left: f32, right: f32| Line {
            text: text.into(),
            left,
            bottom: 700.0,
            right,
            top: 711.0,
        };
        let lines = group_lines(vec![
            run("49,99 EUR", 300.0, 350.0),
            run("Summe", 60.0, 95.0),
            run("  ", 10.0, 12.0),
        ]);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text, "Summe 49,99 EUR");
        assert_eq!((lines[0].left, lines[0].right), (60.0, 350.0));
    }

    #[test]
    fn renders_a_page_and_an_image_at_the_asked_width() {
        let page =
            render_pdf_page(testing::pdfium(), &testing::pdf(&["Rechnung"]), 1, 240).unwrap();
        assert!(page.starts_with(b"\x89PNG"));
        assert_eq!(png_width(&page), 240);
        assert_eq!(
            render_pdf_page(testing::pdfium(), &testing::pdf(&["Rechnung"]), 2, 240),
            Err(PageError::NoSuchPage)
        );
        let photo = render_image(&page, 120).unwrap();
        assert_eq!(png_width(&photo), 120);
    }
}
