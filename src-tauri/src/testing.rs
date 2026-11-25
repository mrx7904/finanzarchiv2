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

/// A PDF with the lines and an attached XML, as a ZUGFeRD or Factur-X invoice carries its data.
pub fn pdf_with_xml(lines: &[&str], name: &str, xml: &str) -> Vec<u8> {
    let mut doc = document(lines);
    let file = doc.add_object(Stream::new(
        dictionary! { "Type" => "EmbeddedFile", "Subtype" => "text#2Fxml" },
        xml.as_bytes().to_vec(),
    ));
    let spec = doc.add_object(dictionary! {
        "Type" => "Filespec",
        "F" => Object::string_literal(name),
        "UF" => Object::string_literal(name),
        "AFRelationship" => "Data",
        "EF" => dictionary! { "F" => file },
    });
    let names = dictionary! {
        "EmbeddedFiles" => dictionary! { "Names" => vec![Object::string_literal(name), spec.into()] },
    };
    let catalog = doc.catalog_mut().unwrap();
    catalog.set("Names", names);
    catalog.set("AF", vec![spec.into()]);
    save(doc)
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

/// An invented invoice in the form of the reference messages, with one tax and a second total
/// of the tax in another currency.
pub const E_INVOICE_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rsm:CrossIndustryInvoice xmlns:rsm="urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100" xmlns:ram="urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100" xmlns:udt="urn:un:unece:uncefact:data:standard:UnqualifiedDataType:100">
  <rsm:ExchangedDocument>
<ram:ID>R-2026-0042</ram:ID>
<ram:IssueDateTime><udt:DateTimeString format="102">20261120</udt:DateTimeString></ram:IssueDateTime>
  </rsm:ExchangedDocument>
  <rsm:SupplyChainTradeTransaction>
<ram:IncludedSupplyChainTradeLineItem>
  <ram:SpecifiedTradeProduct><ram:Name>Leistung</ram:Name></ram:SpecifiedTradeProduct>
</ram:IncludedSupplyChainTradeLineItem>
<ram:ApplicableHeaderTradeAgreement>
  <ram:SellerTradeParty><ram:Name>Beispiel Stadtwerke &amp; Co. GmbH</ram:Name></ram:SellerTradeParty>
  <ram:BuyerTradeParty><ram:Name>Erika Musterfrau</ram:Name></ram:BuyerTradeParty>
</ram:ApplicableHeaderTradeAgreement>
<ram:ApplicableHeaderTradeSettlement>
  <ram:InvoiceCurrencyCode>EUR</ram:InvoiceCurrencyCode>
  <ram:SpecifiedTradeSettlementPaymentMeans>
    <ram:PayeePartyCreditorFinancialAccount><ram:IBANID>DE89370400440532013000</ram:IBANID></ram:PayeePartyCreditorFinancialAccount>
  </ram:SpecifiedTradeSettlementPaymentMeans>
  <ram:ApplicableTradeTax><ram:CalculatedAmount>234.57</ram:CalculatedAmount><ram:RateApplicablePercent>19.00</ram:RateApplicablePercent></ram:ApplicableTradeTax>
  <ram:SpecifiedTradePaymentTerms><ram:DueDateDateTime><udt:DateTimeString format="102">20261204</udt:DateTimeString></ram:DueDateDateTime></ram:SpecifiedTradePaymentTerms>
  <ram:SpecifiedTradeSettlementHeaderMonetarySummation>
    <ram:TaxBasisTotalAmount>1234.56</ram:TaxBasisTotalAmount>
    <ram:TaxTotalAmount currencyID="CHF">220.00</ram:TaxTotalAmount>
    <ram:TaxTotalAmount currencyID="EUR">234.57</ram:TaxTotalAmount>
    <ram:GrandTotalAmount>1469.13</ram:GrandTotalAmount>
  </ram:SpecifiedTradeSettlementHeaderMonetarySummation>
</ram:ApplicableHeaderTradeSettlement>
  </rsm:SupplyChainTradeTransaction>
</rsm:CrossIndustryInvoice>
"#;

// The fake Ollama of the tests of #14.
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;

/// A fake Ollama: answers each connection with the given response and sends the request it read.
pub fn fake_server(response: String) -> (String, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap().to_string();
    let (sender, requests) = mpsc::channel();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { break };
            let request = read_request(&mut stream);
            let _ = stream.write_all(response.as_bytes());
            if sender.send(request).is_err() {
                break;
            }
        }
    });
    (address, requests)
}

fn read_request(stream: &mut TcpStream) -> String {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut head = String::new();
    let mut length = 0;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
            break;
        }
        if let Some(value) = line.to_lowercase().strip_prefix("content-length:") {
            length = value.trim().parse().unwrap_or(0);
        }
        head.push_str(&line);
    }
    let mut body = vec![0; length];
    let _ = reader.read_exact(&mut body);
    format!("{head}\r\n{}", String::from_utf8_lossy(&body))
}

pub fn http_response(status: &str, extra: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{extra}\r\n{body}",
        body.len()
    )
}
