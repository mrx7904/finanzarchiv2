use std::sync::{Arc, Mutex};

use base64::Engine;
use tokio::net::TcpStream;

use super::*;
use crate::mail::fake::{assert_nothing_written, fake_server};
use crate::mail::reader;
use crate::readers::model::tests::Scripted;
use crate::testing;

/// An invented mail with CRLF line ends.
fn mail(id: &str, subject: &str, content_type: &str, body: &str) -> Vec<u8> {
    format!(
        "Message-ID: <{id}@example.com>\r\nFrom: Beispiel GmbH <info@example.com>\r\nTo: test@example.com\r\nDate: Thu, 26 Nov 2026 10:15:00 +0100\r\nSubject: {subject}\r\nMIME-Version: 1.0\r\nContent-Type: {content_type}\r\n\r\n{body}\r\n"
    )
    .into_bytes()
}

fn with_attachment(id: &str, subject: &str, name: &str, kind: &str, bytes: &[u8]) -> Vec<u8> {
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    let body = format!(
        "--grenze\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nAnbei die Unterlagen.\r\n--grenze\r\nContent-Type: {kind}; name=\"{name}\"\r\nContent-Disposition: attachment; filename=\"{name}\"\r\nContent-Transfer-Encoding: base64\r\n\r\n{encoded}\r\n--grenze--"
    );
    mail(id, subject, "multipart/mixed; boundary=\"grenze\"", &body)
}

fn invoice_pdf() -> Vec<u8> {
    testing::pdf(&["Rechnungsnummer R-1", "Rechnungsbetrag 11,90"])
}

/// Five mails: a newsletter, a PDF invoice, an invoice in the text, a shipping note, a photo.
fn five() -> Vec<Vec<u8>> {
    vec![
        mail(
            "a1",
            "Neuigkeiten im November",
            "text/plain; charset=utf-8",
            "Unser Newsletter.",
        ),
        with_attachment(
            "a2",
            "Ihre Unterlagen",
            "R-1.pdf",
            "application/pdf",
            &invoice_pdf(),
        ),
        mail(
            "a3",
            "Ihre Rechnung R-2026-11",
            "text/plain; charset=utf-8",
            "Rechnungsbetrag 23,80 EUR\r\nZahlbar bis 10.12.2026. Grüße aus Köln.",
        ),
        mail(
            "a4",
            "Ihre Bestellung wurde versandt",
            "text/plain; charset=utf-8",
            "Das Paket ist unterwegs.",
        ),
        with_attachment(
            "a5",
            "Fotos vom Ausflug",
            "foto.png",
            "image/png",
            testing::PNG,
        ),
    ]
}

/// A ledger in memory, for tests.
#[derive(Default)]
pub struct Book {
    pub all_known: bool,
    pub noted: Vec<Looked>,
    pub orders: Vec<(String, Vec<crate::orders::Order>)>,
    pub orders_known: Vec<String>,
}

impl Ledger for Book {
    fn known(&self, hash: &str) -> bool {
        self.all_known || self.noted.iter().any(|looked| looked.hash == hash)
    }

    fn paid_before(&self, code: &str) -> bool {
        self.noted
            .iter()
            .any(|looked| looked.paypal_code.as_deref() == Some(code) && !looked.files.is_empty())
    }

    fn orders_read(&self, hash: &str) -> bool {
        self.orders_known.iter().any(|known| known == hash)
            || self.orders.iter().any(|(known, _)| known == hash)
    }

    fn note(&mut self, looked: &Looked) {
        self.noted.push(looked.clone());
    }

    fn note_orders(&mut self, hash: &str, orders: &[crate::orders::Order]) {
        self.orders.push((hash.to_owned(), orders.to_vec()));
    }
}

fn kind(code: &str) -> Result<String, crate::ollama::ModelError> {
    Ok(format!(r#"{{"kind":"{code}"}}"#))
}

fn since() -> jiff::civil::Date {
    jiff::civil::date(2026, 1, 1)
}

#[test]
fn the_head_tells_a_word_of_an_invoice_and_attachments() {
    let five = five();
    let heads: Vec<Head> = five.iter().map(|raw| head_of(raw)).collect();
    assert_eq!(heads[2].subject, "Ihre Rechnung R-2026-11");
    assert_eq!(heads[2].from, "Beispiel GmbH <info@example.com>");
    assert!(!heads[0].attached && heads[1].attached && heads[4].attached);
    assert!(!names_invoice(&heads[0]));
    let billing = head_of(b"From: Billing <rechnung@example.com>\r\nSubject: Ihr Monat\r\n\r\n");
    assert!(names_invoice(&billing), "a sender can name it too");
    assert!(names_invoice(&heads[2]));
    assert_ne!(heads[0].hash, heads[1].hash);
    assert_eq!(
        heads[0].hash,
        head_of(&five[0]).hash,
        "the same mail, the same hash"
    );
    assert_eq!(heads[0].hash.len(), 64);
}

#[test]
fn an_html_mail_becomes_lines_as_a_printed_invoice_reads() {
    let html = r#"<html><head><style>td { color: red }</style><title>Rechnung</title></head><body>
<div style="display:none">Vorschau&zwnj;&nbsp;&zwnj;&nbsp;</div>
<p>Hallo&nbsp;Max,<br>vielen Dank f&uuml;r Ihre Bestellung.</p>
<table><tr><td>Artikel</td><td>Menge</td><td>Preis</td></tr>
<tr><td>Kaffee</td><td>2</td><td>19,98&nbsp;&euro;</td></tr>
<tr><th>Gesamt</th><td></td><td><b>23,80 &euro;</b></td></tr></table>
<p><a href="https://example.com/track?id=1"><img src="https://example.com/logo.png" alt="Logo"></a></p>
<!-- ein Kommentar --><script>alert(1)</script>
<p>-----------</p><p>Mehr unter https://example.com/konto &amp; www.example.com</p></body></html>"#;
    assert_eq!(
        crate::mail::text::html_to_text(html),
        "Hallo Max,\nvielen Dank für Ihre Bestellung.\n\nArtikel Menge Preis\nKaffee 2 19,98 €\nGesamt 23,80 €\n\nMehr unter &"
    );
}

#[test]
fn a_mail_file_carries_its_head_and_text_and_is_known_by_its_mark() {
    let file = crate::mail::text::mail_file(
        &[
            "Von: Beispiel GmbH <info@example.com>".to_owned(),
            "Betreff: Ihre\u{200b} Rechnung".to_owned(),
        ],
        "Gesamt 23,80 €",
    );
    assert_eq!(
        file,
        "FINANZARCHIV-MAIL 1\nVon: Beispiel GmbH <info@example.com>\nBetreff: Ihre Rechnung\n\nGesamt 23,80 €\n"
    );
    assert_eq!(
        crate::documents::FileType::detect(file.as_bytes()),
        Some(crate::documents::FileType::Mail)
    );
    assert_eq!(
        crate::documents::FileType::detect(b"FINANZARCHIV-MAIL 10\nx"),
        None,
        "only the mark itself"
    );
}

#[tokio::test]
async fn a_round_takes_invoices_and_looks_at_every_mail_once() {
    let (at, seen) = fake_server(five()).await;
    let model = Scripted::new(vec![kind("invoice"), kind("invoice"), kind("other")]);
    let mut book = Book::default();
    let (place, counts) = read(
        TcpStream::connect(at).await.unwrap(),
        "test@example.com",
        "app-passwort",
        &Place::default(),
        since(),
        Some(&model),
        &mut book,
    )
    .await
    .unwrap();
    assert_eq!(
        counts,
        Counts {
            looked: 5,
            taken: 2,
            left_out: 3,
            waiting: 0,
            asked: 3,
            orders: 0
        }
    );
    assert_eq!(
        place,
        Place {
            folder: Some("[Gmail]/Alle Nachrichten".into()),
            uid_validity: Some(7),
            last_uid: 5
        }
    );
    let verdicts: Vec<&str> = book
        .noted
        .iter()
        .map(|looked| looked.verdict.as_str())
        .collect();
    assert_eq!(
        verdicts,
        [
            "Kein Zeichen einer Rechnung, nicht übernommen.",
            "Übernommen: der PDF-Anhang (laut Modell eine Rechnung).",
            "Übernommen als Text der Mail (laut Modell eine Rechnung).",
            "Laut Modell kein Beleg, nicht übernommen.",
            "Kein Zeichen einer Rechnung, nicht übernommen.",
        ]
    );
    assert_eq!(book.noted[1].files, [("R-1.pdf".to_owned(), invoice_pdf())]);
    let (name, made) = &book.noted[2].files[0];
    assert_eq!(name, "Ihre Rechnung R-2026-11.txt");
    assert_eq!(
        String::from_utf8(made.clone()).unwrap(),
        "FINANZARCHIV-MAIL 1\nVon: Beispiel GmbH <info@example.com>\nAn: test@example.com\nDatum: 26.11.2026 10:15\nBetreff: Ihre Rechnung R-2026-11\n\nRechnungsbetrag 23,80 EUR\nZahlbar bis 10.12.2026. Grüße aus Köln.\n"
    );
    assert_eq!(
        (book.noted[1].kind, book.noted[2].kind, book.noted[3].kind),
        (Some(Kind::Invoice), Some(Kind::Invoice), None)
    );
    assert_eq!(
        book.noted[0].received_at.as_deref(),
        Some("2026-11-26T10:15:00+01:00")
    );

    // The model read sender, subject, the names of the attachments and the text, nothing else.
    let prompts = model.prompts.lock().unwrap().clone();
    assert!(prompts[0].contains("Anhänge: R-1.pdf"), "{}", prompts[0]);
    assert!(prompts[1].contains("Betreff: Ihre Rechnung R-2026-11"));
    assert!(!prompts.iter().any(|prompt| prompt.contains("sicher")));

    let whole = seen
        .lock()
        .unwrap()
        .iter()
        .filter(|line| line.contains("BODY.PEEK[]"))
        .count();
    assert_eq!(whole, 4, "the newsletter was judged by its head alone");
    assert_nothing_written(&seen);

    // The next round starts after the last mail and finds nothing new.
    let again = Scripted::new(vec![]);
    let (next, counts) = read(
        TcpStream::connect(at).await.unwrap(),
        "test@example.com",
        "app-passwort",
        &place,
        since(),
        Some(&again),
        &mut Book::default(),
    )
    .await
    .unwrap();
    assert_eq!((next.last_uid, counts.looked), (5, 0));
    assert!(again.prompts.lock().unwrap().is_empty());
}

#[tokio::test]
async fn without_the_model_the_rest_waits_and_a_known_mail_is_not_asked_about() {
    let (at, _) = fake_server(five()).await;
    let mut book = Book::default();
    let (place, counts) = read(
        TcpStream::connect(at).await.unwrap(),
        "test@example.com",
        "app-passwort",
        &Place::default(),
        since(),
        None,
        &mut book,
    )
    .await
    .unwrap();
    assert_eq!(
        (book.noted.len(), counts.waiting, place.last_uid),
        (1, 4, 1)
    );

    let model = Scripted::new(vec![]);
    let (place, counts) = read(
        TcpStream::connect(at).await.unwrap(),
        "test@example.com",
        "app-passwort",
        &Place::default(),
        since(),
        Some(&model),
        &mut Book {
            all_known: true,
            ..Book::default()
        },
    )
    .await
    .unwrap();
    assert_eq!((counts.looked, place.last_uid), (0, 5));
    assert!(model.prompts.lock().unwrap().is_empty());
}

#[test]
fn a_mail_is_kept_once_and_its_staged_pdfs_go_once_the_intake_took_them() {
    let dir = tempfile::tempdir().unwrap();
    let archive = Arc::new(Mutex::new(
        crate::archive::open(&dir.path().join(crate::archive::FILE_NAME), testing::t0()).unwrap(),
    ));
    let guard = archive.lock().unwrap();
    let crate::archive::Opened::Ready(ready) = &*guard else {
        panic!("not ready")
    };
    let conn = ready.connection();
    let staging = dir.path().join(reader::STAGING);
    let head = head_of(&five()[1]);
    let looked = Looked {
        hash: head.hash.clone(),
        received_at: head.received_at.clone(),
        verdict: "Übernommen: der PDF-Anhang (laut Modell eine Rechnung).".into(),
        kind: Some(Kind::Invoice),
        files: vec![("R-1.pdf".into(), invoice_pdf())],
        paypal_code: None,
        orders: Vec::new(),
    };
    assert!(!reader::is_known(conn, &head.hash).unwrap());
    conn.execute(
        "INSERT INTO mailboxes (address, host, added_at) VALUES ('test@example.com', 'imap.gmail.com', '2026-11-26T00:00:00Z')",
        [],
    )
    .unwrap();
    let mailbox = conn.last_insert_rowid();
    let paths = reader::keep(conn, &staging, mailbox, &looked, testing::t0()).unwrap();
    assert_eq!(paths.len(), 1);
    assert!(reader::is_known(conn, &head.hash).unwrap());
    let stored: (String, i64) = conn
        .query_row(
            "SELECT verdict, taken FROM mail_messages WHERE message_hash = ?1",
            [&head.hash],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(
        stored,
        (
            "Übernommen: der PDF-Anhang (laut Modell eine Rechnung).".into(),
            1
        )
    );

    crate::intake::jobs::enqueue(conn, &paths, crate::documents::Source::Mail, testing::t0())
        .unwrap();
    reader::tidy(conn, &staging).unwrap();
    assert!(paths[0].exists(), "still queued");
    conn.execute("UPDATE intake_jobs SET stage = 'stored'", [])
        .unwrap();
    reader::tidy(conn, &staging).unwrap();
    assert!(!paths[0].exists(), "taken by the intake");
    assert!(!paths[0].parent().unwrap().exists());
}

#[test]
fn a_date_is_written_as_search_since_takes_it() {
    assert_eq!(imap_date(jiff::civil::date(2026, 11, 1)), "1-Nov-2026");
    assert_eq!(imap_date(jiff::civil::date(2025, 3, 24)), "24-Mar-2025");
}

fn from(sender: &str, id: &str, subject: &str, body: &str) -> Vec<u8> {
    format!(
        "Message-ID: <{id}@example.com>\r\nFrom: {sender}\r\nTo: test@example.com\r\nDate: Tue, 24 Nov 2026 09:30:00 +0100\r\nSubject: {subject}\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n{body}\r\n"
    )
    .into_bytes()
}

const PAYMENT: &str = "Sie haben 9,99 € EUR an Beispiel Musik AB gezahlt\r\nHändler Beispiel Musik AB\r\nTransaktionsdatum 24.11.2026\r\nGesamtbetrag 9,99 € EUR\r\nTransaktionscode: 1AB23456CD7890123";

#[tokio::test]
async fn paypal_is_read_by_code_once_and_amazon_tells_its_orders() {
    let mails = vec![
        from(
            "PayPal <service@paypal.de>",
            "p1",
            "Beleg für Ihre Zahlung an Beispiel Musik AB",
            PAYMENT,
        ),
        from(
            "PayPal <service@paypal.de>",
            "p2",
            "Beleg für Ihre PayPal-Zahlung",
            PAYMENT,
        ),
        from(
            "PayPal <paypal@example.com>",
            "p3",
            "Beleg für Ihre Zahlung an Beispiel Musik AB",
            PAYMENT,
        ),
        from(
            "Amazon.de <bestellbestaetigung@amazon.de>",
            "a1",
            "Bestellt: „Beispiel Buch“",
            "Bestellnr. 302-1234567-7654321\r\nGesamtbetrag 23,80 €",
        ),
        from(
            "Amazon.de <versand@amazon.de>",
            "a2",
            "Versandt: „Beispiel Buch“",
            "Bestellnr. 028-7654321-1234567",
        ),
    ];
    let (at, seen) = fake_server(mails.clone()).await;
    // The forged sender goes to the model, which says other.
    let model = Scripted::new(vec![kind("other")]);
    let mut book = Book::default();
    let (_, counts) = read(
        TcpStream::connect(at).await.unwrap(),
        "test@example.com",
        "app-passwort",
        &Place::default(),
        since(),
        Some(&model),
        &mut book,
    )
    .await
    .unwrap();
    let verdicts: Vec<&str> = book
        .noted
        .iter()
        .map(|looked| looked.verdict.as_str())
        .collect();
    assert_eq!(
        verdicts,
        [
            "Übernommen als Text der Mail (eine PayPal-Zahlung, vom Code gelesen).",
            "Eine zweite Mail derselben PayPal-Zahlung, nicht übernommen.",
            "Laut Modell kein Beleg, nicht übernommen.",
            "Eine Bestellung bei Amazon (302-1234567-7654321), die Rechnung lädst du bei Amazon herunter.",
            "Eine Bestellung bei Amazon (028-7654321-1234567), die Rechnung lädst du bei Amazon herunter.",
        ]
    );
    assert_eq!(book.noted[0].kind, Some(Kind::Receipt));
    assert_eq!(
        book.noted[0].paypal_code.as_deref(),
        Some("1AB23456CD7890123")
    );
    assert_eq!(
        model.prompts.lock().unwrap().len(),
        1,
        "PayPal and Amazon by code, only the forgery asked"
    );
    assert_eq!(
        book.noted[3].orders[0].total,
        Some(crate::money::Cents::new(2380))
    );
    assert_eq!(book.noted[3].orders[0].day.as_deref(), Some("2026-11-24"));
    assert_eq!((counts.taken, counts.orders), (1, 2));
    assert_nothing_written(&seen);

    // A mail of Amazon looked at before #32 is read once more for its orders, nothing else.
    let mut known = Book {
        all_known: true,
        orders_known: vec![head_of(&mails[4]).hash],
        ..Book::default()
    };
    let (_, counts) = read(
        TcpStream::connect(at).await.unwrap(),
        "test@example.com",
        "app-passwort",
        &Place::default(),
        since(),
        None,
        &mut known,
    )
    .await
    .unwrap();
    assert!(known.noted.is_empty());
    assert_eq!(known.orders.len(), 1);
    assert_eq!(known.orders[0].1[0].number, "302-1234567-7654321");
    assert_eq!(known.orders[0].1[0].day.as_deref(), Some("2026-11-24"));
    assert_eq!(counts.orders, 1);
}
