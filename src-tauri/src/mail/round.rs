//! One round through a mailbox (#31, FA-35): every new mail is looked at once. Code filters first
//! by its head: a mail whose subject or sender names an invoice, or that carries attachments, is
//! fetched whole. Only a mail with a PDF attachment or a word of an invoice goes to the local
//! model, which says its kind from sender, subject, names of attachments and the start of the
//! text. The model is never asked how sure it is.

use std::collections::HashMap;
use std::fmt::Debug;

use futures_util::TryStreamExt;
use mail_parser::{MessageParser, MessagePart};
use serde_json::json;
use sha2::{Digest, Sha256};
use tokio::io::{AsyncRead, AsyncWrite};

use super::guard::ReadOnly;
use super::{MailError, classify};
use crate::downloads::{kind_word, money};
use crate::kinds::Kind;
use crate::ollama::{LocalModel, Question};

/// How far back the first round of a mailbox reads.
pub const REACH_DAYS: i64 = 365;
/// Heads fetched in one command.
const BATCH: usize = 50;
/// A larger mail is not fetched (Gmail takes 25 MB).
const LARGEST: u32 = 30 * 1024 * 1024;
/// The start of the text the model reads.
const TEXT_START: usize = 1500;

/// Words of an invoice in a subject or a sender, folded to lower case.
const WORDS: &[&str] = &[
    "rechnung",
    "invoice",
    "beleg",
    "quittung",
    "receipt",
    "zahlung",
    "payment",
    "abrechnung",
    "gutschrift",
    "kontoauszug",
    "bestellung",
    "order",
    "billing",
    "faktura",
];

/// Where a mailbox was left: the folder, its UIDVALIDITY and the last UID looked at.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Place {
    pub folder: Option<String>,
    pub uid_validity: Option<u32>,
    pub last_uid: u32,
}

/// A mail looked at, with its verdict in German and the files it gives to the intake.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Looked {
    pub hash: String,
    pub received_at: Option<String>,
    pub verdict: String,
    /// The kind the model gave a mail it took, the hint for the intake.
    pub kind: Option<Kind>,
    pub files: Vec<(String, Vec<u8>)>,
    /// PayPal's transaction code of a payment mail (#32).
    pub paypal_code: Option<String>,
    /// The orders an Amazon mail tells (#32).
    pub orders: Vec<crate::orders::Order>,
}

/// What a round asks of the archive and tells it (#31, #32).
pub trait Ledger {
    /// Whether a mail was looked at before, by its hash.
    fn known(&self, hash: &str) -> bool;
    /// Whether a PayPal payment with this transaction code was taken before.
    fn paid_before(&self, code: &str) -> bool;
    /// Whether the orders of a mail looked at before were read.
    fn orders_read(&self, hash: &str) -> bool;
    /// Keeps a mail looked at now.
    fn note(&mut self, looked: &Looked);
    /// Keeps the orders of a mail looked at before.
    fn note_orders(&mut self, hash: &str, orders: &[crate::orders::Order]);
}

/// What a round did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts {
    pub looked: u32,
    pub taken: u32,
    pub left_out: u32,
    /// Mails left for the next round, since the model did not answer.
    pub waiting: u32,
    /// Mails that went to the model.
    pub asked: u32,
    /// Orders of Amazon the mails told (#32).
    pub orders: u32,
}

/// "26.11.2026" of an RFC 3339 time, "2026-11-26T10:15:00+01:00".
fn day_of_rfc(time: &str) -> String {
    let day = time.get(..10).unwrap_or("");
    let mut parts = day.split('-');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(year), Some(month), Some(day)) => format!("{day}.{month}.{year}"),
        _ => String::new(),
    }
}

/// The head of a mail, as code judges it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Head {
    pub hash: String,
    pub from: String,
    pub subject: String,
    pub received_at: Option<String>,
    /// Anything but plain text or text with its HTML: it may carry a PDF.
    pub attached: bool,
}

/// The hash of a mail: of its Message-ID, or of sender, date and subject when it has none.
fn hash_of(message_id: Option<&str>, from: &str, date: &str, subject: &str) -> String {
    let source = match message_id.map(str::trim).filter(|id| !id.is_empty()) {
        Some(id) => format!("id:{id}"),
        None => format!("head:{from}\n{date}\n{subject}"),
    };
    crate::documents::hex(&Sha256::digest(source.as_bytes()))
}

fn sender(message: &mail_parser::Message<'_>) -> String {
    message
        .from()
        .and_then(|list| list.first())
        .map(|mailbox| match (mailbox.name(), mailbox.address()) {
            (Some(name), Some(address)) => format!("{name} <{address}>"),
            (None, Some(address)) => address.to_owned(),
            (Some(name), None) => name.to_owned(),
            (None, None) => String::new(),
        })
        .unwrap_or_default()
}

/// Reads the head of a mail from its header lines.
pub fn head_of(header: &[u8]) -> Head {
    let parser = MessageParser::default();
    let Some(message) = parser.parse(header) else {
        return Head {
            hash: hash_of(None, "", "", &String::from_utf8_lossy(header)),
            from: String::new(),
            subject: String::new(),
            received_at: None,
            attached: false,
        };
    };
    let from = sender(&message);
    let subject = message.subject().unwrap_or("").to_owned();
    let date = message.date().map(|date| date.to_rfc3339());
    let attached = message.content_type().is_none_or(|kind| {
        let main = kind.ctype().to_ascii_lowercase();
        let sub = kind.subtype().unwrap_or("").to_ascii_lowercase();
        !(main == "text" || (main == "multipart" && sub == "alternative"))
    });
    Head {
        hash: hash_of(
            message.message_id(),
            &from,
            date.as_deref().unwrap_or(""),
            &subject,
        ),
        from,
        subject,
        received_at: date,
        attached,
    }
}

/// Whether subject or sender names an invoice.
pub fn names_invoice(head: &Head) -> bool {
    let text = format!("{} {}", head.subject, head.from).to_lowercase();
    WORDS.iter().any(|word| text.contains(word))
}

fn is_pdf(part: &MessagePart<'_>) -> bool {
    let named = part
        .attachment_name()
        .is_some_and(|name| name.to_ascii_lowercase().ends_with(".pdf"));
    (named || part.is_content_type("application", "pdf")) && part.decoded().starts_with(b"%PDF")
}

fn pdf_name(name: Option<&str>, fallback: &str) -> String {
    file_name(name, fallback, "pdf")
}

/// A safe file name with the extension, from a name or the fallback.
fn file_name(name: Option<&str>, fallback: &str, extension: &str) -> String {
    let base = name.unwrap_or(fallback);
    let dotted = format!(".{extension}");
    let base = if base.to_ascii_lowercase().ends_with(&dotted) {
        &base[..base.len() - dotted.len()]
    } else {
        base
    };
    let safe = crate::naming::safe(base);
    let safe = if safe.is_empty() {
        fallback.to_owned()
    } else {
        safe
    };
    format!("{safe}.{extension}")
}

const SYSTEM: &str = "Du sortierst Mails. Sage nur, welche Art die Mail ist: eine Rechnung (invoice), \
eine Gehaltsabrechnung (payslip), ein Zahlungsbeleg oder eine Quittung (receipt), ein Kontoauszug (statement) \
oder etwas anderes (other), etwa Werbung, ein Newsletter, eine Versandbestätigung oder eine Nachricht. \
Eine Mail, die eine Rechnung im Anhang trägt, ist eine Rechnung. Erfinde nichts.";

/// Asks the model for the kind of a mail. `None` when it did not answer.
pub fn ask_kind(
    model: &dyn LocalModel,
    head: &Head,
    attachments: &[String],
    text: &str,
) -> Option<Kind> {
    let schema = json!({
        "type": "object",
        "properties": { "kind": { "type": "string", "enum": ["invoice", "payslip", "receipt", "statement", "other"] } },
        "required": ["kind"],
    });
    let start: String = text.chars().take(TEXT_START).collect();
    let prompt = format!(
        "Absender: {}\nBetreff: {}\nAnhänge: {}\nText:\n{}",
        head.from,
        head.subject,
        if attachments.is_empty() {
            "keine".to_owned()
        } else {
            attachments.join(", ")
        },
        start
    );
    let answer = model
        .ask(&Question {
            system: SYSTEM,
            prompt: &prompt,
            schema: &schema,
            images: &[],
        })
        .ok()?;
    let answer: serde_json::Value = serde_json::from_str(&answer).ok()?;
    Some(Kind::from_code(answer["kind"].as_str()?))
}

fn looked(head: &Head, verdict: &str) -> Looked {
    Looked {
        hash: head.hash.clone(),
        received_at: head.received_at.clone(),
        verdict: verdict.to_owned(),
        kind: None,
        files: Vec::new(),
        paypal_code: None,
        orders: Vec::new(),
    }
}

fn day_and_time(message: &mail_parser::Message<'_>) -> String {
    message
        .date()
        .map(|d| {
            format!(
                "{:02}.{:02}.{} {:02}:{:02}",
                d.day, d.month, d.year, d.hour, d.minute
            )
        })
        .unwrap_or_default()
}

/// The lines of the text of a mail as a reader takes them.
fn lines_of(file: &str) -> Vec<crate::readers::Line> {
    file.lines()
        .skip(1)
        .enumerate()
        .map(|(at, text)| crate::readers::Line {
            page: 1,
            text: text.to_owned(),
            left: 0.0,
            bottom: -(at as f64) - 1.0,
            right: 0.0,
            top: -(at as f64),
        })
        .collect()
}

/// Judges a whole mail. `None` when it has to wait for the model.
pub fn judge(
    model: Option<&dyn LocalModel>,
    head: &Head,
    raw: &[u8],
    asked: &mut u32,
    paid_before: &dyn Fn(&str) -> bool,
) -> Option<Looked> {
    let parser = MessageParser::default();
    let Some(message) = parser.parse(raw) else {
        return Some(looked(head, "Die Mail ließ sich nicht lesen."));
    };
    let text = super::text::body_of(&message);
    let recipient = message
        .to()
        .and_then(|list| list.first())
        .and_then(|mailbox| mailbox.address())
        .unwrap_or("")
        .to_owned();
    let mut head_lines = vec![
        format!("Von: {}", head.from),
        format!("An: {recipient}"),
        format!("Datum: {}", day_and_time(&message)),
        format!("Betreff: {}", head.subject),
    ];
    let attached: Vec<&str> = message
        .attachments()
        .filter_map(|part| part.attachment_name())
        .collect();
    if !attached.is_empty() {
        head_lines.push(format!("Anhänge: {}", attached.join(", ")));
    }
    let file = super::text::mail_file(&head_lines, &text);
    let lines = lines_of(&file);
    let text_name = || {
        file_name(
            Some(head.subject.as_str()).filter(|subject| !subject.trim().is_empty()),
            "Mail",
            "txt",
        )
    };
    // A payment mail of PayPal is read by code, never asked about, and taken once (#32).
    if let Some(payment) = crate::readers::paypal::read_payment(&lines)
        && payment.complete()
    {
        let code = payment.code_text();
        if code.as_deref().is_some_and(paid_before) {
            return Some(Looked {
                paypal_code: code,
                ..looked(
                    head,
                    "Eine zweite Mail derselben PayPal-Zahlung, nicht übernommen.",
                )
            });
        }
        return Some(Looked {
            verdict: "Übernommen als Text der Mail (eine PayPal-Zahlung, vom Code gelesen).".into(),
            kind: Some(Kind::Receipt),
            files: vec![(text_name(), file.clone().into_bytes())],
            paypal_code: code,
            ..looked(head, "")
        });
    }
    // A mail of Amazon tells its orders (#32); without a PDF it is no document.
    let orders = if crate::orders::from_amazon(&head.from) {
        let texts: Vec<String> = file.lines().skip(1).map(str::to_owned).collect();
        crate::orders::read(&texts)
    } else {
        Vec::new()
    };
    let pdfs: Vec<(String, Vec<u8>)> = message
        .attachments()
        .filter(is_pdf)
        .map(|part| {
            (
                pdf_name(part.attachment_name(), "Anhang"),
                part.decoded().into_owned(),
            )
        })
        .collect();
    if pdfs.is_empty() && !orders.is_empty() {
        let numbers: Vec<&str> = orders.iter().map(|order| order.number.as_str()).collect();
        return Some(Looked {
            orders: orders.clone(),
            ..looked(
                head,
                &format!(
                    "Eine Bestellung bei Amazon ({}), die Rechnung lädst du bei Amazon herunter.",
                    numbers.join(", ")
                ),
            )
        });
    }
    if pdfs.is_empty() && !names_invoice(head) {
        return Some(looked(
            head,
            "Kein Zeichen einer Rechnung, nicht übernommen.",
        ));
    }
    let names: Vec<String> = pdfs.iter().map(|(name, _)| name.clone()).collect();
    let model = model?;
    *asked += 1;
    let kind = ask_kind(model, head, &names, &text)?;
    if !money(kind) {
        return Some(Looked {
            orders,
            ..looked(head, "Laut Modell kein Beleg, nicht übernommen.")
        });
    }
    if !pdfs.is_empty() {
        let count = pdfs.len();
        return Some(Looked {
            verdict: format!(
                "Übernommen: {} (laut Modell {}).",
                if count == 1 {
                    "der PDF-Anhang".to_owned()
                } else {
                    format!("{count} PDF-Anhänge")
                },
                kind_word(kind)
            ),
            kind: Some(kind),
            files: pdfs,
            orders,
            ..looked(head, "")
        });
    }
    let name = text_name();
    Some(Looked {
        verdict: format!(
            "Übernommen als Text der Mail (laut Modell {}).",
            kind_word(kind)
        ),
        kind: Some(kind),
        files: vec![(name, file.into_bytes())],
        orders,
        ..looked(head, "")
    })
}

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// A date as `SEARCH SINCE` takes it, `1-Nov-2026`.
pub fn imap_date(date: jiff::civil::Date) -> String {
    let month = MONTHS[usize::try_from(date.month() - 1).unwrap_or(0)];
    format!("{}-{month}-{}", date.day(), date.year())
}

/// Reads one mailbox from where it was left, asking and telling the archive through `ledger`.
/// Returns where the mailbox is left and what the round did.
pub async fn read<S>(
    stream: S,
    address: &str,
    password: &str,
    place: &Place,
    since: jiff::civil::Date,
    model: Option<&dyn LocalModel>,
    ledger: &mut dyn Ledger,
) -> Result<(Place, Counts), MailError>
where
    S: AsyncRead + AsyncWrite + Unpin + Debug + Send,
{
    let mut client = async_imap::Client::new(ReadOnly::new(stream));
    match client.read_response().await {
        Ok(Some(_greeting)) => {}
        Ok(None) | Err(_) => return Err(MailError::Lost),
    }
    let mut session = client
        .login(address, password)
        .await
        .map_err(|(error, _)| classify(error))?;

    // Gmail keeps every mail under "All Mail", whatever its labels, in the language of the account.
    let names: Vec<_> = session
        .list(Some(""), Some("*"))
        .await
        .map_err(classify)?
        .try_collect()
        .await
        .map_err(classify)?;
    let folder = names
        .iter()
        .find(|name| {
            name.attributes()
                .iter()
                .any(|attribute| matches!(attribute, async_imap::types::NameAttribute::All))
        })
        .map_or_else(|| "INBOX".to_owned(), |name| name.name().to_owned());
    let opened = session.examine(&folder).await.map_err(classify)?;
    let same = place.folder.as_deref() == Some(folder.as_str())
        && place.uid_validity.is_some()
        && place.uid_validity == opened.uid_validity
        && place.last_uid > 0;
    let mut last = if same { place.last_uid } else { 0 };
    let query = if same {
        format!("UID {}:*", last + 1)
    } else {
        format!("SINCE {}", imap_date(since))
    };
    let mut uids: Vec<u32> = session
        .uid_search(&query)
        .await
        .map_err(classify)?
        .into_iter()
        .filter(|uid| *uid > last)
        .collect();
    uids.sort_unstable();

    let mut counts = Counts::default();
    'round: for batch in uids.chunks(BATCH) {
        let set: Vec<String> = batch.iter().map(u32::to_string).collect();
        let fetched: Vec<_> = session
            .uid_fetch(set.join(","), "(UID RFC822.SIZE BODY.PEEK[HEADER])")
            .await
            .map_err(classify)?
            .try_collect()
            .await
            .map_err(classify)?;
        let heads: HashMap<u32, (u32, Head)> = fetched
            .iter()
            .filter_map(|fetch| {
                Some((
                    fetch.uid?,
                    (fetch.size.unwrap_or(0), head_of(fetch.header()?)),
                ))
            })
            .collect();
        for uid in batch {
            let Some((size, head)) = heads.get(uid) else {
                // Gone since the search: nothing to look at.
                last = *uid;
                continue;
            };
            let amazon = crate::orders::from_amazon(&head.from);
            if ledger.known(&head.hash) {
                // A mail of Amazon looked at before #32 is read once more for its orders.
                if amazon && *size <= LARGEST && !ledger.orders_read(&head.hash) {
                    let whole: Vec<_> = session
                        .uid_fetch(uid.to_string(), "(UID BODY.PEEK[])")
                        .await
                        .map_err(classify)?
                        .try_collect()
                        .await
                        .map_err(classify)?;
                    let orders = whole
                        .first()
                        .and_then(|fetch| fetch.body())
                        .and_then(|raw| {
                            MessageParser::default()
                                .parse(raw)
                                .map(|message| super::text::body_of(&message))
                        })
                        .map(|body| {
                            let mut texts = vec![format!(
                                "Datum: {}",
                                head.received_at
                                    .as_deref()
                                    .map(day_of_rfc)
                                    .unwrap_or_default()
                            )];
                            texts.extend(body.lines().map(str::to_owned));
                            crate::orders::read(&texts)
                        })
                        .unwrap_or_default();
                    counts.orders += u32::try_from(orders.len()).unwrap_or(0);
                    ledger.note_orders(&head.hash, &orders);
                }
                last = *uid;
                continue;
            }
            let paypal = head.from.to_lowercase().contains("paypal");
            let verdict = if *size > LARGEST {
                Some(looked(head, "Zu groß zum Lesen, nicht übernommen."))
            } else if head.attached || names_invoice(head) || amazon || paypal {
                let whole: Vec<_> = session
                    .uid_fetch(uid.to_string(), "(UID BODY.PEEK[])")
                    .await
                    .map_err(classify)?
                    .try_collect()
                    .await
                    .map_err(classify)?;
                match whole.first().and_then(|fetch| fetch.body()) {
                    Some(raw) => judge(model, head, raw, &mut counts.asked, &|code| {
                        ledger.paid_before(code)
                    }),
                    None => Some(looked(head, "Die Mail ließ sich nicht lesen.")),
                }
            } else {
                Some(looked(
                    head,
                    "Kein Zeichen einer Rechnung, nicht übernommen.",
                ))
            };
            let Some(verdict) = verdict else {
                // The model did not answer: this mail and every later one wait for the next round.
                counts.waiting = u32::try_from(uids.iter().filter(|later| **later >= *uid).count())
                    .unwrap_or(u32::MAX);
                break 'round;
            };
            ledger.note(&verdict);
            counts.looked += 1;
            counts.orders += u32::try_from(verdict.orders.len()).unwrap_or(0);
            if verdict.files.is_empty() {
                counts.left_out += 1;
            } else {
                counts.taken += 1;
            }
            last = *uid;
        }
    }
    let _ = session.logout().await;
    Ok((
        Place {
            folder: Some(folder),
            uid_validity: opened.uid_validity,
            last_uid: last,
        },
        counts,
    ))
}

#[cfg(test)]
mod tests;
