//! Writes the generated corpus (FA-13, #09): invoices, payslips and bank statements with invented
//! senders, numbers and IBANs, each beside a JSON file with the right values as printed.
//!
//!   cargo run --example corpus -- --seed 2026 --out ../fixtures/synthetic
//!
//! The same seed gives the same bytes, so the corpus in the repository can be checked against a
//! fresh run. Nothing here is real: every name is made up and every IBAN has a valid check sum
//! only so that a reader treats it as an IBAN.

use std::fs;
use std::path::{Path, PathBuf};

use finanzarchiv2_lib::money::Cents;
use finanzarchiv2_lib::pdf::{self, Pdfium};
use jiff::civil::{Date, date};
use lopdf::content::{Content, Operation};
use lopdf::{Document, Object, Stream, StringFormat, dictionary};
use pdfium_render::prelude::PdfRenderConfig;
use serde_json::{Value, json};

const SENDERS: [&str; 10] = [
    "Beispiel Stadtwerke GmbH",
    "Muster Versicherung AG",
    "Fiktiv Telekom GmbH",
    "Erfunden Möbel KG",
    "Probe Handwerk GmbH",
    "Demo Online-Shop GmbH",
    "Testhaus Energie AG",
    "Platzhalter Verlag GmbH",
    "Modell Werkstatt e.K.",
    "Exempel Fitness GmbH",
];
const EMPLOYER: &str = "Beispiel Arbeitgeber GmbH";
const BANK: &str = "Musterbank";
const MONTHS: [&str; 12] = [
    "Januar",
    "Februar",
    "März",
    "April",
    "Mai",
    "Juni",
    "Juli",
    "August",
    "September",
    "Oktober",
    "November",
    "Dezember",
];

/// xorshift64, so a seed fixes every value.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }

    fn cents(&mut self, from: i64, to: i64) -> i64 {
        from + i64::try_from(self.below(u64::try_from(to - from).unwrap_or(1))).unwrap_or(0)
    }
}

/// "1.234,56" as a German document prints it.
fn printed(cents: i64) -> String {
    Cents::new(cents)
        .to_string()
        .trim_end_matches(" €")
        .to_owned()
}

fn day(value: Date) -> String {
    value.strftime("%d.%m.%Y").to_string()
}

/// A German IBAN with a valid check sum around an invented bank code and account.
fn iban(rng: &mut Rng) -> String {
    let bban = format!(
        "{:08}{:010}",
        10_000_000 + rng.below(89_999_999),
        rng.below(9_999_999_999)
    );
    let digits = format!("{bban}131400");
    let remainder = digits.bytes().fold(0_u64, |rest, digit| {
        (rest * 10 + u64::from(digit - b'0')) % 97
    });
    let check = 98 - remainder;
    let raw = format!("DE{check:02}{bban}");
    raw.as_bytes()
        .chunks(4)
        .map(|chunk| String::from_utf8_lossy(chunk).into_owned())
        .collect::<Vec<_>>()
        .join(" ")
}

/// One text at a place on the page, in points from the lower left corner.
struct Text {
    x: i64,
    y: i64,
    size: i64,
    text: String,
}

fn text(x: i64, y: i64, size: i64, text: impl Into<String>) -> Text {
    Text {
        x,
        y,
        size,
        text: text.into(),
    }
}

fn latin1(text: &str) -> Vec<u8> {
    text.chars()
        .map(|c| u8::try_from(u32::from(c)).unwrap_or(b'?'))
        .collect()
}

/// A one-page A4 PDF with the texts in Helvetica, WinAnsi encoding so umlauts survive.
fn text_pdf(texts: &[Text]) -> Document {
    let mut doc = Document::with_version("1.7");
    let pages_id = doc.new_object_id();
    let font = doc.add_object(dictionary! {
        "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica", "Encoding" => "WinAnsiEncoding",
    });
    let resources = doc.add_object(dictionary! { "Font" => dictionary! { "F1" => font } });
    let mut operations = Vec::new();
    for item in texts {
        operations.extend([
            Operation::new("BT", vec![]),
            Operation::new("Tf", vec!["F1".into(), item.size.into()]),
            Operation::new("Td", vec![item.x.into(), item.y.into()]),
            Operation::new(
                "Tj",
                vec![Object::String(latin1(&item.text), StringFormat::Literal)],
            ),
            Operation::new("ET", vec![]),
        ]);
    }
    let content = doc.add_object(Stream::new(
        dictionary! {},
        Content { operations }.encode().unwrap_or_default(),
    ));
    finish(&mut doc, pages_id, resources, content);
    doc
}

fn finish(
    doc: &mut Document,
    pages_id: lopdf::ObjectId,
    resources: lopdf::ObjectId,
    content: lopdf::ObjectId,
) {
    let page = doc
        .add_object(dictionary! { "Type" => "Page", "Parent" => pages_id, "Contents" => content });
    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages", "Kids" => vec![page.into()], "Count" => 1, "Resources" => resources,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        }),
    );
    let catalog = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    doc.trailer.set("Root", catalog);
    let id = Object::String(b"finanzarchiv2-corpus".to_vec(), StringFormat::Hexadecimal);
    doc.trailer.set("ID", vec![id.clone(), id]);
}

fn bytes(mut doc: Document) -> Vec<u8> {
    let mut out = Vec::new();
    if doc.save_to(&mut out).is_err() {
        panic!("a generated PDF could not be written");
    }
    out
}

/// The page of a text PDF as a JPEG inside a PDF of its own: a scan without a text layer.
fn scanned(pdfium: &Pdfium, text_pdf: &[u8]) -> Vec<u8> {
    let rendered = pdfium
        .load_pdf_from_byte_slice(text_pdf, None)
        .and_then(|document| {
            document
                .pages()
                .get(0)?
                .render_with_config(&PdfRenderConfig::new().set_target_width(1240))?
                .as_image()
        })
        .unwrap_or_else(|_| panic!("the page could not be rendered for a scan"))
        .to_rgb8();
    let (width, height) = rendered.dimensions();
    let mut jpeg = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, 80)
        .encode_image(&rendered)
        .unwrap_or_else(|_| panic!("the scan could not be encoded"));
    let mut doc = Document::with_version("1.7");
    let pages_id = doc.new_object_id();
    let picture = doc.add_object(Stream::new(
        dictionary! {
            "Type" => "XObject", "Subtype" => "Image", "Width" => i64::from(width), "Height" => i64::from(height),
            "ColorSpace" => "DeviceRGB", "BitsPerComponent" => 8, "Filter" => "DCTDecode",
        },
        jpeg,
    ));
    let resources = doc.add_object(dictionary! { "XObject" => dictionary! { "Im1" => picture } });
    let operations = vec![
        Operation::new("q", vec![]),
        Operation::new(
            "cm",
            vec![
                595.into(),
                0.into(),
                0.into(),
                842.into(),
                0.into(),
                0.into(),
            ],
        ),
        Operation::new("Do", vec!["Im1".into()]),
        Operation::new("Q", vec![]),
    ];
    let content = doc.add_object(Stream::new(
        dictionary! {},
        Content { operations }.encode().unwrap_or_default(),
    ));
    finish(&mut doc, pages_id, resources, content);
    bytes(doc)
}

/// Attaches a Factur-X XML, as a ZUGFeRD invoice carries its data for machines.
fn attach_factur_x(doc: &mut Document, xml: &str) {
    let file = doc.add_object(Stream::new(
        dictionary! { "Type" => "EmbeddedFile", "Subtype" => "text#2Fxml" },
        xml.as_bytes().to_vec(),
    ));
    let spec = doc.add_object(dictionary! {
        "Type" => "Filespec", "F" => Object::string_literal("factur-x.xml"), "UF" => Object::string_literal("factur-x.xml"),
        "AFRelationship" => "Data", "EF" => dictionary! { "F" => file },
    });
    let names = dictionary! { "EmbeddedFiles" => dictionary! { "Names" => vec![Object::string_literal("factur-x.xml"), spec.into()] } };
    if let Ok(catalog) = doc.catalog_mut() {
        catalog.set("Names", names);
        catalog.set("AF", vec![spec.into()]);
    }
}

struct Writer {
    root: PathBuf,
}

impl Writer {
    fn write(&self, folder: &str, name: &str, pdf: &[u8], truth: Value) {
        let dir = self.root.join(folder);
        fs::create_dir_all(&dir)
            .unwrap_or_else(|_| panic!("{} could not be created", dir.display()));
        fs::write(dir.join(format!("{name}.pdf")), pdf)
            .unwrap_or_else(|_| panic!("{name}.pdf could not be written"));
        let json = serde_json::to_string_pretty(&truth).unwrap_or_default();
        fs::write(dir.join(format!("{name}.json")), format!("{json}\n"))
            .unwrap_or_else(|_| panic!("{name}.json could not be written"));
    }
}

fn invoices(rng: &mut Rng, pdfium: &Pdfium, out: &Writer) {
    for number in 1..=30 {
        let sender = SENDERS[usize::try_from(rng.below(10)).unwrap_or(0)];
        let issued = date(
            2026,
            i8::try_from(1 + rng.below(11)).unwrap_or(1),
            i8::try_from(1 + rng.below(28)).unwrap_or(1),
        );
        let due = issued
            .checked_add(jiff::Span::new().days(14))
            .unwrap_or(issued);
        let rate = if number % 6 == 0 { 7 } else { 19 };
        let net = rng.cents(500, 200_000);
        let vat = (net * rate + 50) / 100;
        let gross = net + vat;
        let invoice_number = format!("R-2026-{:04}", rng.below(9_000) + 1_000);
        let account = iban(rng);
        let texts = vec![
            text(60, 780, 16, sender),
            text(60, 762, 9, "Beispielweg 1, 12345 Musterstadt"),
            text(60, 700, 13, "Rechnung"),
            text(60, 680, 10, format!("Rechnungsnummer {invoice_number}")),
            text(60, 664, 10, format!("Rechnungsdatum {}", day(issued))),
            text(60, 620, 10, "Leistung"),
            text(420, 620, 10, "Betrag EUR"),
            text(60, 600, 10, "Leistungen laut Vertrag"),
            text(420, 600, 10, printed(net)),
            text(60, 560, 10, "Nettobetrag"),
            text(420, 560, 10, printed(net)),
            text(60, 544, 10, format!("Umsatzsteuer {rate} %")),
            text(420, 544, 10, printed(vat)),
            text(60, 524, 11, "Rechnungsbetrag"),
            text(420, 524, 11, printed(gross)),
            text(60, 480, 10, format!("Zahlbar bis {}", day(due))),
            text(60, 464, 10, format!("{BANK}, IBAN {account}")),
        ];
        let mut doc = text_pdf(&texts);
        let e_invoice = number == 30;
        if e_invoice {
            attach_factur_x(
                &mut doc,
                &factur_x(
                    &invoice_number,
                    issued,
                    due,
                    sender,
                    &account,
                    [net, vat, gross],
                    rate,
                ),
            );
        }
        let pdf = bytes(doc);
        let scan = number % 6 == 5;
        let pdf = if scan { scanned(pdfium, &pdf) } else { pdf };
        let truth = json!({
            "kind": "invoice",
            "text_layer": !scan,
            "e_invoice": e_invoice,
            "fields": {
                "sender": sender, "invoice_number": invoice_number, "invoice_date": day(issued),
                "net": printed(net), "vat_rate": rate.to_string(), "vat": printed(vat), "gross": printed(gross),
                "due_date": day(due), "iban": account,
            },
            "cents": { "net": net, "vat": vat, "gross": gross },
        });
        out.write("rechnungen", &format!("rechnung-{number:02}"), &pdf, truth);
    }
}

fn factur_x(
    number: &str,
    issued: Date,
    due: Date,
    sender: &str,
    account: &str,
    [net, vat, gross]: [i64; 3],
    rate: i64,
) -> String {
    let amount = |cents: i64| format!("{}.{:02}", cents / 100, cents % 100);
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<rsm:CrossIndustryInvoice xmlns:rsm="urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100" xmlns:ram="urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100" xmlns:udt="urn:un:unece:uncefact:data:standard:UnqualifiedDataType:100">
  <rsm:ExchangedDocument>
    <ram:ID>{number}</ram:ID>
    <ram:TypeCode>380</ram:TypeCode>
    <ram:IssueDateTime><udt:DateTimeString format="102">{issued}</udt:DateTimeString></ram:IssueDateTime>
  </rsm:ExchangedDocument>
  <rsm:SupplyChainTradeTransaction>
    <ram:ApplicableHeaderTradeAgreement>
      <ram:SellerTradeParty><ram:Name>{sender}</ram:Name></ram:SellerTradeParty>
    </ram:ApplicableHeaderTradeAgreement>
    <ram:ApplicableHeaderTradeSettlement>
      <ram:InvoiceCurrencyCode>EUR</ram:InvoiceCurrencyCode>
      <ram:SpecifiedTradeSettlementPaymentMeans><ram:TypeCode>58</ram:TypeCode><ram:PayeePartyCreditorFinancialAccount><ram:IBANID>{iban}</ram:IBANID></ram:PayeePartyCreditorFinancialAccount></ram:SpecifiedTradeSettlementPaymentMeans>
      <ram:ApplicableTradeTax><ram:CalculatedAmount>{vat}</ram:CalculatedAmount><ram:RateApplicablePercent>{rate}</ram:RateApplicablePercent></ram:ApplicableTradeTax>
      <ram:SpecifiedTradePaymentTerms><ram:DueDateDateTime><udt:DateTimeString format="102">{due}</udt:DateTimeString></ram:DueDateDateTime></ram:SpecifiedTradePaymentTerms>
      <ram:SpecifiedTradeSettlementHeaderMonetarySummation>
        <ram:TaxBasisTotalAmount>{net}</ram:TaxBasisTotalAmount>
        <ram:TaxTotalAmount currencyID="EUR">{vat}</ram:TaxTotalAmount>
        <ram:GrandTotalAmount>{gross}</ram:GrandTotalAmount>
      </ram:SpecifiedTradeSettlementHeaderMonetarySummation>
    </ram:ApplicableHeaderTradeSettlement>
  </rsm:SupplyChainTradeTransaction>
</rsm:CrossIndustryInvoice>
"#,
        issued = issued.strftime("%Y%m%d"),
        due = due.strftime("%Y%m%d"),
        iban = account.replace(' ', ""),
        net = amount(net),
        vat = amount(vat),
        gross = amount(gross),
    )
}

fn payslips(rng: &mut Rng, out: &Writer) {
    let base = rng.cents(320_000, 520_000);
    for month in 1..=12_usize {
        let gross = base + if month == 11 { base / 2 } else { 0 };
        let share = |percent_tenths: i64| (gross * percent_tenths + 500) / 1000;
        let wage_tax = (gross * i64::try_from(120 + rng.below(40)).unwrap_or(140) + 500) / 1000;
        let church = (wage_tax * 8 + 50) / 100;
        let (health, pension, unemployment, care) = (share(87), share(93), share(13), share(18));
        let net = gross - wage_tax - church - health - pension - unemployment - care;
        let label = MONTHS[month - 1];
        let rows: [(&str, i64); 9] = [
            ("Gesamtbrutto", gross),
            ("Lohnsteuer", wage_tax),
            ("Solidaritätszuschlag", 0),
            ("Kirchensteuer", church),
            ("Krankenversicherung", health),
            ("Rentenversicherung", pension),
            ("Arbeitslosenversicherung", unemployment),
            ("Pflegeversicherung", care),
            ("Auszahlungsbetrag", net),
        ];
        let mut texts = vec![
            text(60, 780, 14, EMPLOYER),
            text(60, 740, 12, format!("Verdienstabrechnung {label} 2026")),
            text(60, 722, 9, "Personalnummer 00042, Steuerklasse 1"),
        ];
        for (index, (name, cents)) in rows.iter().enumerate() {
            let y = 680 - 20 * i64::try_from(index).unwrap_or(0);
            texts.push(text(60, y, 10, *name));
            texts.push(text(420, y, 10, printed(*cents)));
        }
        let truth = json!({
            "kind": "payslip",
            "text_layer": true,
            "fields": {
                "employer": EMPLOYER, "period": format!("{label} 2026"), "brutto": printed(gross), "lohnsteuer": printed(wage_tax),
                "kirchensteuer": printed(church), "krankenversicherung": printed(health), "rentenversicherung": printed(pension),
                "arbeitslosenversicherung": printed(unemployment), "pflegeversicherung": printed(care), "netto": printed(net),
            },
            "cents": { "brutto": gross, "netto": net },
        });
        out.write(
            "gehalt",
            &format!("gehalt-2026-{month:02}"),
            &bytes(text_pdf(&texts)),
            truth,
        );
    }
}

fn statements(rng: &mut Rng, out: &Writer) {
    let account = iban(rng);
    let mut balance = rng.cents(50_000, 500_000);
    for number in 1..=6_i8 {
        let opening = balance;
        let from = date(2026, number, 1);
        let to = from.last_of_month();
        let mut bookings = Vec::new();
        for _ in 0..(4 + rng.below(5)) {
            let on = date(2026, number, i8::try_from(1 + rng.below(28)).unwrap_or(1));
            let income = rng.below(5) == 0;
            let amount = if income {
                rng.cents(10_000, 400_000)
            } else {
                -rng.cents(500, 90_000)
            };
            let party = if income {
                EMPLOYER
            } else {
                SENDERS[usize::try_from(rng.below(10)).unwrap_or(0)]
            };
            bookings.push((on, party, amount));
        }
        bookings.sort_by_key(|booking| booking.0);
        balance = opening + bookings.iter().map(|booking| booking.2).sum::<i64>();
        let layout_b = number % 2 == 0;
        let mut texts = vec![
            text(60, 780, 14, BANK),
            text(
                60,
                756,
                12,
                if layout_b {
                    format!("Umsatzübersicht {number}/2026")
                } else {
                    format!("Kontoauszug Nr. {number}/2026")
                },
            ),
            text(60, 738, 10, format!("IBAN {account}")),
            text(
                60,
                722,
                10,
                format!("Zeitraum {} bis {}", day(from), day(to)),
            ),
            text(
                60,
                700,
                10,
                if layout_b {
                    "Saldo alt"
                } else {
                    "Alter Kontostand"
                },
            ),
            text(420, 700, 10, statement_amount(opening, layout_b)),
        ];
        for (index, (on, party, amount)) in bookings.iter().enumerate() {
            let y = 670 - 18 * i64::try_from(index).unwrap_or(0);
            if layout_b {
                texts.push(text(60, y, 9, party.to_string()));
                texts.push(text(300, y, 9, day(*on)));
            } else {
                texts.push(text(60, y, 9, day(*on)));
                texts.push(text(140, y, 9, party.to_string()));
            }
            texts.push(text(420, y, 9, statement_amount(*amount, layout_b)));
        }
        let last = 670 - 18 * i64::try_from(bookings.len()).unwrap_or(0) - 10;
        texts.push(text(
            60,
            last,
            10,
            if layout_b {
                "Saldo neu"
            } else {
                "Neuer Kontostand"
            },
        ));
        texts.push(text(420, last, 10, statement_amount(balance, layout_b)));
        let truth = json!({
            "kind": "statement",
            "text_layer": true,
            "layout": if layout_b { "B" } else { "A" },
            "fields": {
                "iban": account, "period_from": day(from), "period_to": day(to),
                "opening": statement_amount(opening, layout_b), "closing": statement_amount(balance, layout_b),
            },
            "bookings": bookings.iter().map(|(on, party, amount)| json!({
                "date": day(*on), "party": party, "printed": statement_amount(*amount, layout_b), "cents": amount,
            })).collect::<Vec<_>>(),
            "cents": { "opening": opening, "closing": balance },
        });
        out.write(
            "kontoauszuege",
            &format!("kontoauszug-2026-{number:02}"),
            &bytes(text_pdf(&texts)),
            truth,
        );
    }
}

/// Layout A prints a sign ("-12,34"), layout B the German S for Soll and H for Haben ("12,34 S").
fn statement_amount(cents: i64, layout_b: bool) -> String {
    if layout_b {
        format!(
            "{} {}",
            printed(cents.abs()),
            if cents < 0 { "S" } else { "H" }
        )
    } else {
        printed(cents)
    }
}

/// Writes the whole corpus for a seed into `root`, which it empties first.
fn generate(seed: u64, root: &Path, pdfium: &Pdfium) {
    for folder in ["rechnungen", "gehalt", "kontoauszuege"] {
        let _ = fs::remove_dir_all(root.join(folder));
    }
    let out = Writer {
        root: root.to_path_buf(),
    };
    let mut rng = Rng(seed.max(1));
    invoices(&mut rng, pdfium, &out);
    payslips(&mut rng, &out);
    statements(&mut rng, &out);
}

fn engine() -> &'static Pdfium {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("pdfium");
    pdf::init(&[pdf::platform_folder(&root)])
        .unwrap_or_else(|_| panic!("PDFium is missing, run `node scripts/fetch-pdfium.mjs`"))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let value = |flag: &str| {
        args.iter()
            .position(|arg| arg == flag)
            .and_then(|index| args.get(index + 1))
            .cloned()
    };
    let seed = value("--seed")
        .and_then(|seed| seed.parse().ok())
        .unwrap_or(2026);
    let out = PathBuf::from(value("--out").unwrap_or_else(|| "fixtures/synthetic".into()));
    generate(seed, &out, engine());
    println!(
        "Korpus mit Startwert {seed} geschrieben nach {}",
        out.display()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files(dir: &Path) -> Vec<(String, Vec<u8>)> {
        let mut found = Vec::new();
        for folder in ["rechnungen", "gehalt", "kontoauszuege"] {
            let mut names: Vec<_> = fs::read_dir(dir.join(folder))
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .collect();
            names.sort();
            for path in names {
                found.push((
                    path.file_name().unwrap().to_string_lossy().into_owned(),
                    fs::read(&path).unwrap(),
                ));
            }
        }
        found
    }

    #[test]
    fn the_same_seed_gives_the_same_files() {
        let (first, second) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        generate(2026, first.path(), engine());
        generate(2026, second.path(), engine());
        let (a, b) = (files(first.path()), files(second.path()));
        assert_eq!(a.len(), 2 * (30 + 12 + 6));
        assert!(a == b, "two runs with one seed differ");
        let other = tempfile::tempdir().unwrap();
        generate(7, other.path(), engine());
        assert!(
            files(other.path()) != a,
            "another seed gives the same corpus"
        );
    }

    #[test]
    fn the_amounts_add_up() {
        let dir = tempfile::tempdir().unwrap();
        generate(2026, dir.path(), engine());
        for (name, bytes) in files(dir.path())
            .into_iter()
            .filter(|(name, _)| name.ends_with(".json"))
        {
            let truth: Value = serde_json::from_slice(&bytes).unwrap();
            let cents = &truth["cents"];
            match truth["kind"].as_str().unwrap() {
                "invoice" => assert_eq!(
                    cents["net"].as_i64().unwrap() + cents["vat"].as_i64().unwrap(),
                    cents["gross"].as_i64().unwrap(),
                    "{name}"
                ),
                "statement" => {
                    let sum: i64 = truth["bookings"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|b| b["cents"].as_i64().unwrap())
                        .sum();
                    assert_eq!(
                        cents["opening"].as_i64().unwrap() + sum,
                        cents["closing"].as_i64().unwrap(),
                        "{name}"
                    );
                }
                _ => assert!(
                    cents["netto"].as_i64().unwrap() < cents["brutto"].as_i64().unwrap(),
                    "{name}"
                ),
            }
        }
    }

    #[test]
    fn every_iban_has_a_valid_check_sum() {
        let mut rng = Rng(99);
        for _ in 0..50 {
            let value = iban(&mut rng).replace(' ', "");
            let moved = format!("{}{}", &value[4..], &value[..4]);
            let digits: String = moved
                .chars()
                .map(|c| {
                    if c.is_ascii_digit() {
                        c.to_string()
                    } else {
                        (u32::from(c) - 55).to_string()
                    }
                })
                .collect();
            let remainder = digits.bytes().fold(0_u64, |rest, digit| {
                (rest * 10 + u64::from(digit - b'0')) % 97
            });
            assert_eq!(remainder, 1, "{value}");
        }
    }
}
