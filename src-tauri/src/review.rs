//! "Prüfen" (FA-22, #17): what waits, grouped by its first reason, as questions the owner answers
//! mostly with yes or no. An answer is a value of the owner, and the document goes through the
//! checks again (knowledge/decision-checks.md).

use jiff::Timestamp;
use rusqlite::{Connection, params};

use crate::checks::{self, ACCEPT, AGREE, DUPLICATE, KIND, PRESENT, SOURCE, SUM, UNDONE};
use crate::kinds::Kind;
use crate::money::Cents;
use crate::readers::model::{Form, fields_of};

/// The reasons in the order a document is grouped by: its first failed check in this list.
const ORDER: [&str; 7] = [UNDONE, KIND, PRESENT, SOURCE, AGREE, SUM, DUPLICATE];

/// The key of a group: the check, and the field for the checks of a value (`check:agree gross`).
fn key(check: &str, field: Option<&str>) -> String {
    match field {
        Some(field) => format!("{check} {field}"),
        None => check.to_owned(),
    }
}

/// The German name of a group.
pub fn group_label(key: &str) -> String {
    let (check, field) = key
        .split_once(' ')
        .map_or((key, None), |(check, field)| (check, Some(field)));
    let name = field.map(field_label).unwrap_or("Wert");
    match check {
        UNDONE => "Ablegen rückgängig gemacht".into(),
        KIND => "Art unklar".into(),
        PRESENT => format!("{name} fehlt"),
        SOURCE => format!("{name} nur aus dem Bild"),
        AGREE => format!("{name} ungleich gelesen"),
        SUM => "Summe geht nicht auf".into(),
        DUPLICATE => "Mögliches Doppel".into(),
        LINK => "Zahlung zuordnen".into(),
        _ => "Sonstiges".into(),
    }
}

pub fn field_label(field: &str) -> &'static str {
    match field {
        "gross" => "Bruttobetrag",
        "net" => "Nettobetrag",
        "vat" => "Umsatzsteuer",
        "brutto" => "Brutto",
        "netto" => "Netto",
        "sender" => "Absender",
        "employer" => "Arbeitgeber",
        _ => "Wert",
    }
}

/// The first reason a waiting document waits for, as the key of its group.
fn first_reason(conn: &Connection, document: i64) -> rusqlite::Result<Option<String>> {
    let failed: Vec<(String, Option<String>)> = conn
        .prepare("SELECT check_id, field FROM field_checks WHERE document_id = ?1 AND passed = 0 AND check_id LIKE 'check:%'")?
        .query_map([document], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(ORDER.iter().find_map(|check| {
        failed
            .iter()
            .find(|(id, _)| id == check)
            .map(|(id, field)| key(id, field.as_deref()))
    }))
}

/// The field under which the owner sets a document aside: it stays unfiled and leaves "Prüfen".
pub const HOLD: &str = "hold";

/// The documents that wait for the owner, those set aside excepted.
fn waiting(conn: &Connection) -> rusqlite::Result<Vec<i64>> {
    conn.prepare(
        "SELECT id FROM documents d WHERE state = 'waiting' AND NOT EXISTS (
             SELECT 1 FROM field_values v WHERE v.document_id = d.id AND v.reader = 'owner' AND v.field = 'hold' AND v.value = '1')
         ORDER BY imported_at, id",
    )?
        .query_map([], |row| row.get(0))?
        .collect()
}

/// The group of the documents for which more than one booking fits (#24).
pub const LINK: &str = "link:choose";

/// The groups with how many documents wait in each, in the order of `ORDER`, then the link questions.
pub fn groups(conn: &Connection) -> rusqlite::Result<Vec<(String, u32)>> {
    let mut counts: Vec<(String, u32)> = Vec::new();
    for document in waiting(conn)? {
        let Some(key) = first_reason(conn, document)? else {
            continue;
        };
        match counts.iter_mut().find(|(have, _)| *have == key) {
            Some((_, count)) => *count += 1,
            None => counts.push((key, 1)),
        }
    }
    counts.sort_by_key(|(key, _)| {
        ORDER
            .iter()
            .position(|check| key.split(' ').next() == Some(*check))
    });
    let asked = u32::try_from(crate::links::asked(conn)?.len()).unwrap_or(u32::MAX);
    if asked > 0 {
        counts.push((LINK.to_owned(), asked));
    }
    Ok(counts)
}

/// One reading of a field, beside the others when the owner says "Nein".
#[derive(Debug, Clone, PartialEq)]
pub struct Reading {
    pub reader: String,
    pub printed: String,
    pub value: String,
    pub page: Option<u32>,
    /// left, bottom, right, top in PDF points.
    pub rect: Option<[f64; 4]>,
}

/// A question about one waiting document.
#[derive(Debug, Clone, PartialEq)]
pub struct Question {
    pub document: i64,
    pub name: String,
    pub key: String,
    pub field: Option<String>,
    pub text: String,
    pub readings: Vec<Reading>,
    /// The page to show and its size in PDF points, for marking the line.
    pub page: u32,
    pub page_size: Option<(f64, f64)>,
}

fn reader_label(reader: &str) -> String {
    match reader.split_once(':').map_or(reader, |(family, _)| family) {
        "owner" => "du".into(),
        "e-invoice" => "E-Rechnung".into(),
        "code" => "Code".into(),
        "model" => "Modell".into(),
        "image" => "Bild".into(),
        other => other.into(),
    }
}

fn readings(conn: &Connection, document: i64, field: &str) -> rusqlite::Result<Vec<Reading>> {
    let rows: Vec<Reading> = conn
        .prepare(
            "SELECT reader, printed, value, page, left, bottom, right, top FROM field_values
             WHERE document_id = ?1 AND field = ?2 ORDER BY id",
        )?
        .query_map(params![document, field], |row| {
            let rect = match (
                row.get::<_, Option<f64>>(4)?,
                row.get::<_, Option<f64>>(5)?,
                row.get::<_, Option<f64>>(6)?,
                row.get::<_, Option<f64>>(7)?,
            ) {
                (Some(left), Some(bottom), Some(right), Some(top)) => {
                    Some([left, bottom, right, top])
                }
                _ => None,
            };
            Ok(Reading {
                reader: reader_label(&row.get::<_, String>(0)?),
                printed: row.get(1)?,
                value: row.get(2)?,
                page: row.get(3)?,
                rect,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    // One reading per distinct value, the first reader that gave it.
    let mut distinct: Vec<Reading> = Vec::new();
    for reading in rows {
        if !distinct.iter().any(|have| have.value == reading.value) {
            distinct.push(reading);
        }
    }
    Ok(distinct)
}

fn amount_of(value: &str) -> String {
    value
        .parse::<i64>()
        .map_or_else(|_| value.to_owned(), |cents| Cents::new(cents).to_string())
}

fn kind_of(conn: &Connection, document: i64) -> rusqlite::Result<Kind> {
    conn.query_row(
        "SELECT kind FROM documents WHERE id = ?1",
        [document],
        |row| row.get::<_, String>(0),
    )
    .map(|code| Kind::from_code(&code))
}

/// The field a question is about: the field of the check, or the decisive value of a sum.
fn field_of(key: &str, kind: Kind) -> Option<String> {
    match key.split_once(' ') {
        Some((_, field)) => Some(field.to_owned()),
        None if key == SUM => checks::decisive(kind)
            .last()
            .map(|field| (*field).to_owned()),
        None => None,
    }
}

fn question(conn: &Connection, document: i64, key: &str) -> rusqlite::Result<Question> {
    let name: String = conn.query_row(
        "SELECT coalesce(name, original_name) FROM documents WHERE id = ?1",
        [document],
        |row| row.get(0),
    )?;
    let kind = kind_of(conn, document)?;
    let chosen = checks::chosen(&checks::values(conn, document)?);
    let sender = chosen
        .get("sender")
        .or_else(|| chosen.get("employer"))
        .map(|seen| format!(" von „{}“", seen.printed))
        .unwrap_or_default();
    let field = field_of(key, kind);
    let readings = match &field {
        Some(field) => readings(conn, document, field)?,
        None => Vec::new(),
    };
    let check = key.split(' ').next().unwrap_or(key);
    let text = match (check, &field, readings.first()) {
        (KIND, _, _) => "Welche Art hat das Dokument?".to_owned(),
        (UNDONE, _, _) => {
            "Du hast das Ablegen rückgängig gemacht. Soll es doch abgelegt werden?".to_owned()
        }
        (DUPLICATE, _, _) => format!("Ist das ein eigenes Dokument{sender} und kein Doppel?"),
        (SUM, _, _) => {
            let get = |field: &str| {
                chosen
                    .get(field)
                    .map(|seen| amount_of(&seen.value))
                    .unwrap_or_else(|| "–".into())
            };
            if kind == Kind::Payslip {
                format!(
                    "Stimmt das Netto {}{sender}, obwohl Brutto {} minus Abzüge nicht aufgeht?",
                    get("netto"),
                    get("brutto")
                )
            } else {
                format!(
                    "Stimmen Netto {}, Steuer {} und Brutto {}{sender} so?",
                    get("net"),
                    get("vat"),
                    get("gross")
                )
            }
        }
        (_, Some(field), Some(first)) => format!(
            "Ist {} der {}{sender}?",
            amount_of(&first.value),
            field_label(field)
        ),
        (_, Some(field), None) => format!("Wie hoch ist der {}{sender}?", field_label(field)),
        _ => "Soll das Dokument abgelegt werden?".to_owned(),
    };
    let page = readings
        .first()
        .and_then(|reading| reading.page)
        .unwrap_or(1);
    let page_size = conn
        .query_row(
            "SELECT width, height FROM pages WHERE document_id = ?1 AND number = ?2",
            params![document, page],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();
    Ok(Question {
        document,
        name,
        key: key.to_owned(),
        field,
        text,
        readings,
        page,
        page_size,
    })
}

/// A link question: which of the fitting bookings paid the document (#24).
fn link_question(conn: &Connection, document: i64) -> rusqlite::Result<Option<Question>> {
    let Some(candidates) = crate::links::question(conn, document)? else {
        return Ok(None);
    };
    let name: String = conn.query_row(
        "SELECT coalesce(name, original_name) FROM documents WHERE id = ?1",
        [document],
        |row| row.get(0),
    )?;
    let chosen = checks::chosen(&checks::values(conn, document)?);
    let sender = chosen
        .get("sender")
        .or_else(|| chosen.get("employer"))
        .map(|seen| format!(" von „{}“", seen.printed))
        .unwrap_or_default();
    let readings = candidates
        .iter()
        .map(|(id, day, cents, other)| {
            let day = day
                .parse::<jiff::civil::Date>()
                .map_or_else(|_| day.clone(), |day| day.strftime("%d.%m.%Y").to_string());
            Reading {
                reader: "Buchung".into(),
                printed: format!(
                    "{day} · {} · {}",
                    Cents::new(*cents),
                    other.as_deref().unwrap_or("ohne Gegenseite")
                ),
                value: id.to_string(),
                page: None,
                rect: None,
            }
        })
        .collect();
    Ok(Some(Question {
        document,
        name,
        key: LINK.to_owned(),
        field: Some("booking".into()),
        text: format!("Welche Buchung hat das Dokument{sender} bezahlt?"),
        readings,
        page: 1,
        page_size: None,
    }))
}

/// The questions of a group, oldest document first.
pub fn questions(conn: &Connection, key: &str) -> rusqlite::Result<Vec<Question>> {
    if key == LINK {
        let mut found = Vec::new();
        for document in crate::links::asked(conn)? {
            found.extend(link_question(conn, document)?);
        }
        return Ok(found);
    }
    let mut found = Vec::new();
    for document in waiting(conn)? {
        if first_reason(conn, document)?.as_deref() == Some(key) {
            found.push(question(conn, document, key)?);
        }
    }
    Ok(found)
}

/// What the owner answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    Yes,
    No,
    Reading(usize),
    Typed(String),
    Kind(Kind),
}

/// Why an answer was not taken. German, for the owner.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Refused {
    #[error("{0}")]
    Value(String),
    #[error("Diese Antwort passt nicht zu der Frage.")]
    Mismatch,
}

#[derive(Debug, thiserror::Error)]
pub enum AnswerError {
    #[error(transparent)]
    Refused(#[from] Refused),
    #[error(transparent)]
    Database(#[from] rusqlite::Error),
}

/// The value the owner typed, parsed as the field's form. An amount through `Cents::parse_de`.
fn typed(form: Form, text: &str) -> Result<(String, String), Refused> {
    let text = text.trim();
    match form {
        Form::Amount => Cents::parse_de(text.trim_end_matches('€').trim())
            .map(|cents| (text.to_owned(), cents.value().to_string()))
            .map_err(|error| Refused::Value(error.to_string())),
        Form::Date => crate::readers::parse::first_date(text)
            .map(|(printed, date)| (printed, date.to_string()))
            .ok_or_else(|| {
                Refused::Value(format!(
                    "„{text}“ ist kein Datum. Erwartet wird etwa „20.11.2026“."
                ))
            }),
        _ if text.is_empty() => Err(Refused::Value("Kein Wert angegeben.".into())),
        _ => Ok((text.to_owned(), text.to_owned())),
    }
}

fn store_owner(
    conn: &Connection,
    document: i64,
    field: &str,
    printed: &str,
    value: &str,
    from: Option<&Reading>,
    now: Timestamp,
) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM field_values WHERE document_id = ?1 AND field = ?2 AND reader = 'owner'",
        params![document, field],
    )?;
    let rect = from.and_then(|reading| reading.rect);
    conn.execute(
        "INSERT INTO field_values (document_id, field, printed, value, reader, page, left, bottom, right, top, read_at)
         VALUES (?1, ?2, ?3, ?4, 'owner', ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            document,
            field,
            printed,
            value,
            from.and_then(|reading| reading.page),
            rect.map(|r| r[0]),
            rect.map(|r| r[1]),
            rect.map(|r| r[2]),
            rect.map(|r| r[3]),
            now.to_string()
        ],
    )?;
    Ok(())
}

/// Takes the owner's answer to the question of a group about a document, stores it as a value of
/// the owner and runs the checks again. Returns the document's new state.
pub fn answer(
    conn: &Connection,
    document: i64,
    key: &str,
    answer: &Answer,
    now: Timestamp,
) -> Result<String, AnswerError> {
    if key == LINK {
        let index = match answer {
            Answer::Yes => Some(0),
            Answer::Reading(index) => Some(*index),
            Answer::No => None,
            _ => return Err(Refused::Mismatch.into()),
        };
        let chosen = match index {
            Some(index) => {
                let question = link_question(conn, document)?.ok_or(Refused::Mismatch)?;
                let reading = question.readings.get(index).ok_or(Refused::Mismatch)?;
                Some(
                    reading
                        .value
                        .parse::<i64>()
                        .map_err(|_| Refused::Mismatch)?,
                )
            }
            None => None,
        };
        crate::links::answer(conn, document, chosen, now)?;
        return Ok("filed".into());
    }
    let tx = conn.unchecked_transaction()?;
    let kind = kind_of(&tx, document)?;
    let check = key.split(' ').next().unwrap_or(key);
    let accept = |tx: &Connection| {
        store_owner(
            tx,
            document,
            &format!("{ACCEPT}{check}"),
            "ja",
            "1",
            None,
            now,
        )
    };
    match (check, answer) {
        (KIND, Answer::Kind(chosen)) => {
            crate::documents::set_kind(&tx, document, *chosen, &["owner"])?;
            if *chosen == Kind::Other {
                accept(&tx)?;
            }
            // The readers of the new kind read it again, the owner's answers stay.
            crate::readers::read_document(&tx, document, *chosen, now)?;
        }
        (KIND, _) => return Err(Refused::Mismatch.into()),
        (UNDONE | DUPLICATE, Answer::Yes) => accept(&tx)?,
        // "Nein, nicht ablegen": the document is set aside, unfiled, and leaves "Prüfen".
        (UNDONE, Answer::No) => store_owner(&tx, document, HOLD, "zurückgehalten", "1", None, now)?,
        // A duplicate is answered with "Nein" by deleting it, `delete_duplicate`.
        (DUPLICATE, _) => return Err(Refused::Mismatch.into()),
        (SUM, Answer::Yes) => accept(&tx)?,
        (_, Answer::No) => {}
        (_, answer) => {
            let field = field_of(key, kind).ok_or(Refused::Mismatch)?;
            let readings = readings(&tx, document, &field)?;
            let form = fields_of(kind)
                .iter()
                .find(|(name, _, _)| *name == field)
                .map_or(Form::Text, |(_, _, form)| *form);
            let (printed, value, from) = match answer {
                Answer::Yes => {
                    let first = readings.first().ok_or(Refused::Mismatch)?;
                    (first.printed.clone(), first.value.clone(), Some(first))
                }
                Answer::Reading(index) => {
                    let reading = readings.get(*index).ok_or(Refused::Mismatch)?;
                    (
                        reading.printed.clone(),
                        reading.value.clone(),
                        Some(reading),
                    )
                }
                Answer::Typed(text) => {
                    let (printed, value) = typed(form, text)?;
                    (printed, value, None)
                }
                _ => return Err(Refused::Mismatch.into()),
            };
            store_owner(&tx, document, &field, &printed, &value, from, now)?;
        }
    }
    let verdict = checks::decide(&tx, document, now)?;
    tx.commit()?;
    Ok(if verdict.filed() {
        "filed".into()
    } else {
        "waiting".into()
    })
}

/// "Nein, das ist ein Doppel": the document leaves the archive (its original goes to the
/// `Papierkorb`), and the documents that waited as its duplicate are checked again.
pub fn delete_duplicate(
    conn: &Connection,
    root: &std::path::Path,
    document: i64,
    now: Timestamp,
) -> Result<Option<std::path::PathBuf>, AnswerError> {
    if first_reason(conn, document)?.as_deref() != Some(DUPLICATE) {
        return Err(Refused::Mismatch.into());
    }
    let moved = crate::documents::delete(conn, root, document)?;
    let twins: Vec<i64> = conn
        .prepare(
            "SELECT DISTINCT document_id FROM field_checks WHERE check_id = ?1 AND passed = 0",
        )?
        .query_map([DUPLICATE], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    for twin in twins {
        checks::decide(conn, twin, now)?;
    }
    Ok(moved)
}

#[cfg(test)]
mod tests;
