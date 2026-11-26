//! The commands of "Prüfen" (#17).

use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use super::CommandError;
use crate::archive::Opened;
use crate::kinds::Kind;
use crate::review::{self, Answer, AnswerError};

/// A group of waiting documents with the same first reason.
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReviewGroup {
    /// The reason, for example `check:agree gross`.
    pub key: String,
    pub label: String,
    pub count: u32,
}

/// One reading of the field a question is about.
#[derive(Debug, Clone, Serialize, Type, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReviewReading {
    /// Who read it, in German: Code, Modell, Bild, E-Rechnung, du.
    pub reader: String,
    pub printed: String,
    /// The value as the owner reads it, an amount as "1.234,56 €".
    pub shown: String,
    pub page: Option<u32>,
    /// The line on the page as fractions of its width and height from the top left: left, top, width, height.
    pub mark: Option<Vec<f64>>,
}

/// A question about a waiting document.
#[derive(Debug, Clone, Serialize, Type, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReviewQuestion {
    pub document: u32,
    pub name: String,
    pub key: String,
    pub field: Option<String>,
    pub text: String,
    pub readings: Vec<ReviewReading>,
    pub page: u32,
}

/// What the owner answered.
#[derive(Debug, Clone, Deserialize, Type, PartialEq, Eq)]
#[serde(tag = "answer", rename_all = "camelCase")]
pub enum ReviewAnswer {
    Yes,
    No,
    Reading { index: u32 },
    Typed { text: String },
    Kind { kind: String },
}

fn with_ready<T>(
    archive: &Mutex<Opened>,
    run: impl FnOnce(&rusqlite::Connection) -> Result<T, CommandError>,
) -> Result<T, CommandError> {
    let opened = archive.lock().map_err(|_| CommandError::internal())?;
    let Opened::Ready(ready) = &*opened else {
        return Err(CommandError::archive_not_ready());
    };
    run(ready.connection())
}

/// The groups of "Prüfen" with their counts.
#[tauri::command]
#[specta::specta]
pub fn review_groups(
    archive: State<'_, Arc<Mutex<Opened>>>,
) -> Result<Vec<ReviewGroup>, CommandError> {
    with_ready(&archive, |conn| {
        review::groups(conn)
            .map_err(|_| CommandError::internal())?
            .into_iter()
            .map(|(key, count)| {
                Ok(ReviewGroup {
                    label: review::group_label(&key),
                    key,
                    count,
                })
            })
            .collect()
    })
}

fn shown(field: Option<&str>, value: &str) -> String {
    let amount = !matches!(
        field,
        Some("sender" | "employer" | "invoice_number" | "period" | "iban")
    );
    match value.parse::<i64>() {
        Ok(cents) if amount => crate::money::Cents::new(cents).to_string(),
        _ => value.to_owned(),
    }
}

/// The questions of a group.
#[tauri::command]
#[specta::specta]
pub fn review_questions(
    archive: State<'_, Arc<Mutex<Opened>>>,
    key: String,
) -> Result<Vec<ReviewQuestion>, CommandError> {
    with_ready(&archive, |conn| {
        let questions = review::questions(conn, &key).map_err(|_| CommandError::internal())?;
        questions
            .into_iter()
            .map(|question| {
                let size = question.page_size;
                Ok(ReviewQuestion {
                    document: u32::try_from(question.document)
                        .map_err(|_| CommandError::internal())?,
                    readings: question
                        .readings
                        .iter()
                        .map(|reading| ReviewReading {
                            reader: reading.reader.clone(),
                            printed: reading.printed.clone(),
                            shown: if question.field.as_deref() == Some("booking") {
                                reading.printed.clone()
                            } else {
                                shown(question.field.as_deref(), &reading.value)
                            },
                            page: reading.page,
                            mark: reading.rect.zip(size).map(
                                |([left, bottom, right, top], (width, height))| {
                                    vec![
                                        left / width,
                                        (height - top) / height,
                                        (right - left) / width,
                                        (top - bottom) / height,
                                    ]
                                },
                            ),
                        })
                        .collect(),
                    name: question.name,
                    key: question.key,
                    field: question.field,
                    text: question.text,
                    page: question.page,
                })
            })
            .collect()
    })
}

/// Takes the owner's answer and returns the document's new state, `filed` or `waiting`.
#[tauri::command]
#[specta::specta]
pub fn answer_question(
    archive: State<'_, Arc<Mutex<Opened>>>,
    document: u32,
    key: String,
    answer: ReviewAnswer,
) -> Result<String, CommandError> {
    let answer = match answer {
        ReviewAnswer::Yes => Answer::Yes,
        ReviewAnswer::No => Answer::No,
        ReviewAnswer::Reading { index } => {
            Answer::Reading(usize::try_from(index).unwrap_or(usize::MAX))
        }
        ReviewAnswer::Typed { text } => Answer::Typed(text),
        ReviewAnswer::Kind { kind } => Answer::Kind(Kind::from_code(&kind)),
    };
    with_ready(&archive, |conn| {
        review::answer(
            conn,
            i64::from(document),
            &key,
            &answer,
            jiff::Timestamp::now(),
        )
        .map_err(|error| match error {
            AnswerError::Refused(refused) => CommandError::refused(refused.to_string()),
            AnswerError::Database(_) => CommandError::internal(),
        })
    })
}

/// "Nein, das ist ein Doppel": deletes the document, its original goes to the `Papierkorb` beside
/// the archive. The documents that waited as its duplicate are checked again.
#[tauri::command]
#[specta::specta]
pub fn delete_duplicate(
    archive: State<'_, Arc<Mutex<Opened>>>,
    document: u32,
) -> Result<(), CommandError> {
    let opened = archive.lock().map_err(|_| CommandError::internal())?;
    let Opened::Ready(ready) = &*opened else {
        return Err(CommandError::archive_not_ready());
    };
    let root = crate::documents::archive_root(ready.path());
    review::delete_duplicate(
        ready.connection(),
        &root,
        i64::from(document),
        jiff::Timestamp::now(),
    )
    .map(|_| ())
    .map_err(|error| match error {
        AnswerError::Refused(refused) => CommandError::refused(refused.to_string()),
        AnswerError::Database(_) => CommandError::internal(),
    })
}
