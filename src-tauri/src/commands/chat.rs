//! The commands of the chat (#42): the chats, their messages, asking, deleting. The model works on
//! a read-only connection of its own, the archive is locked only to read the history and to keep
//! the question and the answer.

use std::sync::{Arc, Mutex};

use serde::Serialize;
use specta::Type;
use tauri::State;

use super::CommandError;
use crate::archive::Opened;
use crate::chat::{self, Message, Source};
use crate::intake::Intake;

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChatRow {
    pub id: u32,
    pub title: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChatSource {
    pub document: Option<u32>,
    pub booking: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessageRow {
    pub id: u32,
    /// "owner" or "archive".
    pub role: String,
    pub text: String,
    pub sources: Vec<ChatSource>,
    pub asked_at: String,
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChatAnswer {
    pub chat: u32,
    pub answer: ChatMessageRow,
}

fn id32(id: i64) -> Result<u32, CommandError> {
    u32::try_from(id).map_err(|_| CommandError::internal())
}

fn message_row(message: &Message) -> Result<ChatMessageRow, CommandError> {
    Ok(ChatMessageRow {
        id: id32(message.id)?,
        role: message.role.clone(),
        text: message.text.clone(),
        sources: message
            .sources
            .iter()
            .map(|source| {
                Ok(match source {
                    Source::Document(id) => ChatSource {
                        document: Some(id32(*id)?),
                        booking: None,
                    },
                    Source::Booking(id) => ChatSource {
                        document: None,
                        booking: Some(id32(*id)?),
                    },
                })
            })
            .collect::<Result<_, CommandError>>()?,
        asked_at: message.asked_at.clone(),
    })
}

fn with_ready<T>(
    archive: &Mutex<Opened>,
    run: impl FnOnce(&crate::archive::Archive) -> Result<T, CommandError>,
) -> Result<T, CommandError> {
    let opened = archive.lock().map_err(|_| CommandError::internal())?;
    let Opened::Ready(ready) = &*opened else {
        return Err(CommandError::archive_not_ready());
    };
    run(ready)
}

#[tauri::command]
#[specta::specta]
pub fn list_chats(archive: State<'_, Arc<Mutex<Opened>>>) -> Result<Vec<ChatRow>, CommandError> {
    with_ready(&archive, |ready| {
        chat::chats(ready.connection())
            .map_err(|_| CommandError::internal())?
            .into_iter()
            .map(|one| {
                Ok(ChatRow {
                    id: id32(one.id)?,
                    title: one.title,
                    created_at: one.created_at,
                })
            })
            .collect()
    })
}

#[tauri::command]
#[specta::specta]
pub fn chat_messages(
    archive: State<'_, Arc<Mutex<Opened>>>,
    chat: u32,
) -> Result<Vec<ChatMessageRow>, CommandError> {
    with_ready(&archive, |ready| {
        chat::messages(ready.connection(), i64::from(chat))
            .map_err(|_| CommandError::internal())?
            .iter()
            .map(message_row)
            .collect()
    })
}

#[tauri::command]
#[specta::specta]
pub fn delete_chat(archive: State<'_, Arc<Mutex<Opened>>>, chat: u32) -> Result<(), CommandError> {
    with_ready(&archive, |ready| {
        chat::delete(ready.connection(), i64::from(chat)).map_err(|_| CommandError::internal())
    })
}

/// Asks the archive, in a new chat or one that exists. The answer is checked before it is kept.
#[tauri::command]
#[specta::specta]
pub async fn ask_chat(
    archive: State<'_, Arc<Mutex<Opened>>>,
    intake: State<'_, Arc<Intake>>,
    chat: Option<u32>,
    question: String,
) -> Result<ChatAnswer, CommandError> {
    let question = question.trim().to_owned();
    if question.is_empty() {
        return Err(CommandError::refused("Die Frage ist leer.".into()));
    }
    let model = intake.model().ok_or_else(|| {
        CommandError::refused("Das Modell ist nicht bereit. Läuft Ollama?".into())
    })?;
    let (path, history) = with_ready(&archive, |ready| {
        let history = match chat {
            Some(chat) => chat::history(ready.connection(), i64::from(chat))
                .map_err(|_| CommandError::internal())?,
            None => Vec::new(),
        };
        Ok((ready.path().to_path_buf(), history))
    })?;
    let asked = question.clone();
    let answer = tauri::async_runtime::spawn_blocking(move || {
        let conn = chat::read_only(&path).map_err(|_| CommandError::internal())?;
        let today = jiff::tz::Offset::UTC
            .to_datetime(jiff::Timestamp::now())
            .date();
        chat::answer(&conn, model.as_ref(), &asked, &history, today)
            .map_err(|error| CommandError::refused(error.to_string()))
    })
    .await
    .map_err(|_| CommandError::internal())??;
    let archive = Arc::clone(&archive);
    with_ready(&archive, |ready| {
        let conn = ready.connection();
        let now = jiff::Timestamp::now();
        let chat = match chat {
            Some(chat) => i64::from(chat),
            None => chat::new_chat(conn, &question, now).map_err(|_| CommandError::internal())?,
        };
        chat::add(conn, chat, "owner", &question, &[], now)
            .map_err(|_| CommandError::internal())?;
        let id = chat::add(conn, chat, "archive", &answer.text, &answer.sources, now)
            .map_err(|_| CommandError::internal())?;
        let kept = chat::messages(conn, chat)
            .map_err(|_| CommandError::internal())?
            .into_iter()
            .find(|message| message.id == id)
            .ok_or_else(CommandError::internal)?;
        Ok(ChatAnswer {
            chat: id32(chat)?,
            answer: message_row(&kept)?,
        })
    })
}
