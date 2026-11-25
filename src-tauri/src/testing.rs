//! Helpers of the tests: a fixed time, the PDF engine and generated documents with invented text.

use std::path::PathBuf;

use jiff::Timestamp;
use lopdf::content::{Content, Operation};
use lopdf::{
    Document, EncryptionState, EncryptionVersion, Object, Permissions, Stream, StringFormat,
    dictionary,
};

use crate::pdf::{self, Pdfium};

/// The first eight bytes of every PNG, enough for the type check.
pub const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR";

pub fn t0() -> Timestamp {
    "2026-11-25T08:00:00Z".parse().unwrap()
}

/// PDFium as `node scripts/fetch-pdfium.mjs` unpacks it. The check fetches it before the tests.
pub fn pdfium() -> &'static Pdfium {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("pdfium");
    pdf::init(&[pdf::platform_folder(&root)])
        .expect("PDFium is missing, run `node scripts/fetch-pdfium.mjs`")
}

/// A one-page A4 PDF with one line of text per entry, in Helvetica with WinAnsi encoding,
/// so German umlauts survive.
pub fn pdf(lines: &[&str]) -> Vec<u8> {
    save(document(lines))
}

/// A page with a drawn line and no text, as a scan looks to a text reader.
pub fn pdf_without_text() -> Vec<u8> {
    let mut doc = document(&[]);
    let content = Content {
        operations: vec![
            Operation::new("m", vec![60.into(), 700.into()]),
            Operation::new("l", vec![500.into(), 700.into()]),
            Operation::new("S", vec![]),
        ],
    };
    let page_id = doc.page_iter().next().unwrap();
    let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));
    doc.get_object_mut(page_id)
        .unwrap()
        .as_dict_mut()
        .unwrap()
        .set("Contents", content_id);
    save(doc)
}

/// The same page, encrypted so that it opens only with `password`.
pub fn protected_pdf(lines: &[&str], password: &str) -> Vec<u8> {
    let mut doc = document(lines);
    let state = EncryptionState::try_from(EncryptionVersion::V2 {
        document: &doc,
        owner_password: "eigentuemer",
        user_password: password,
        key_length: 128,
        permissions: Permissions::all(),
    })
    .unwrap();
    doc.encrypt(&state).unwrap();
    save(doc)
}

fn document(lines: &[&str]) -> Document {
    let mut doc = Document::with_version("1.7");
    let pages_id = doc.new_object_id();
    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
        "Encoding" => "WinAnsiEncoding",
    });
    let resources_id = doc.add_object(dictionary! { "Font" => dictionary! { "F1" => font_id } });
    let mut operations = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let y = 780 - 20 * i64::try_from(index).unwrap();
        operations.extend([
            Operation::new("BT", vec![]),
            Operation::new("Tf", vec!["F1".into(), 11.into()]),
            Operation::new("Td", vec![60.into(), y.into()]),
            Operation::new(
                "Tj",
                vec![Object::String(latin1(line), StringFormat::Literal)],
            ),
            Operation::new("ET", vec![]),
        ]);
    }
    let content_id = doc.add_object(Stream::new(
        dictionary! {},
        Content { operations }.encode().unwrap(),
    ));
    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "Contents" => content_id,
    });
    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1,
            "Resources" => resources_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        }),
    );
    let catalog_id = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    doc.trailer.set("Root", catalog_id);
    let id = Object::String(b"finanzarchiv2-test".to_vec(), StringFormat::Hexadecimal);
    doc.trailer.set("ID", vec![id.clone(), id]);
    doc
}

fn latin1(text: &str) -> Vec<u8> {
    text.chars()
        .map(|c| u8::try_from(u32::from(c)).unwrap_or(b'?'))
        .collect()
}

fn save(mut doc: Document) -> Vec<u8> {
    let mut bytes = Vec::new();
    doc.save_to(&mut bytes).unwrap();
    bytes
}
