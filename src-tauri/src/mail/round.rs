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
use crate::pdf::Pdfium;

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
    pub files: Vec<(String, Vec<u8>)>,
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
    let base = name.unwrap_or(fallback);
    let base = base
        .strip_suffix(".pdf")
        .or_else(|| base.strip_suffix(".PDF"))
        .unwrap_or(base);
    let safe = crate::naming::safe(base);
    let safe = if safe.is_empty() {
        fallback.to_owned()
    } else {
        safe
    };
    format!("{safe}.pdf")
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
        files: Vec::new(),
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

/// Judges a whole mail. `None` when it has to wait for the model or for PDFium.
pub fn judge(
    pdfium: Option<&Pdfium>,
    model: Option<&dyn LocalModel>,
    head: &Head,
    raw: &[u8],
    asked: &mut u32,
) -> Option<Looked> {
    let parser = MessageParser::default();
    let Some(message) = parser.parse(raw) else {
        return Some(looked(head, "Die Mail ließ sich nicht lesen."));
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
    if pdfs.is_empty() && !names_invoice(head) {
        return Some(looked(
            head,
            "Kein Zeichen einer Rechnung, nicht übernommen.",
        ));
    }
    let text = message.body_text(0).unwrap_or_default().into_owned();
    let names: Vec<String> = pdfs.iter().map(|(name, _)| name.clone()).collect();
    let model = model?;
    *asked += 1;
    let kind = ask_kind(model, head, &names, &text)?;
    if !money(kind) {
        return Some(looked(head, "Laut Modell kein Beleg, nicht übernommen."));
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
            files: pdfs,
            ..looked(head, "")
        });
    }
    let head_lines = [
        format!("Von: {}", head.from),
        format!("Datum: {}", day_and_time(&message)),
        format!("Betreff: {}", head.subject),
    ];
    let pdf = super::render::mail_pdf(pdfium?, &head_lines, &text).ok()?;
    let name = pdf_name(
        Some(head.subject.as_str()).filter(|subject| !subject.trim().is_empty()),
        "Mail",
    );
    Some(Looked {
        verdict: format!(
            "Übernommen als PDF der Mail (laut Modell {}).",
            kind_word(kind)
        ),
        files: vec![(name, pdf)],
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

/// Reads one mailbox from where it was left. `known` says whether a mail was looked at before,
/// `note` keeps a mail looked at now. Returns where the mailbox is left and what the round did.
#[allow(clippy::too_many_arguments)]
pub async fn read<S>(
    stream: S,
    address: &str,
    password: &str,
    place: &Place,
    since: jiff::civil::Date,
    pdfium: Option<&Pdfium>,
    model: Option<&dyn LocalModel>,
    known: &dyn Fn(&str) -> bool,
    note: &mut dyn FnMut(&Looked),
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
            if known(&head.hash) {
                last = *uid;
                continue;
            }
            let verdict = if *size > LARGEST {
                Some(looked(head, "Zu groß zum Lesen, nicht übernommen."))
            } else if head.attached || names_invoice(head) {
                let whole: Vec<_> = session
                    .uid_fetch(uid.to_string(), "(UID BODY.PEEK[])")
                    .await
                    .map_err(classify)?
                    .try_collect()
                    .await
                    .map_err(classify)?;
                match whole.first().and_then(|fetch| fetch.body()) {
                    Some(raw) => judge(pdfium, model, head, raw, &mut counts.asked),
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
            note(&verdict);
            counts.looked += 1;
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
