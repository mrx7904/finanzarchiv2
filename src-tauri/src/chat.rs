//! The chat over the archive (#42, FA-45), taken over from 1.0 (`decision-chat.md` there): the
//! local model reaches data only through fixed lookups that reuse the core's own reads, on a
//! connection that opens the archive read-only. Code formats every amount of a lookup, checks every
//! amount of the answer against them, and the sources of an answer are the documents and bookings
//! the lookups returned (knowledge/decision-chat.md).

use std::collections::BTreeSet;
use std::path::Path;

use jiff::Timestamp;
use jiff::civil::Date;
use rusqlite::{Connection, OpenFlags, params};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::money::Cents;
use crate::ollama::{LocalModel, ModelError, Question};

/// The answer when every lookup found nothing.
pub const NOTHING_FOUND: &str = "Dazu habe ich im Archiv nichts gefunden.";
/// The answer when the model kept writing amounts no lookup returned.
pub const NO_SURE_NUMBERS: &str =
    "Die Zahlen dazu kann ich nicht sicher wiedergeben. Die Quellen stehen unten.";
/// At most this many lookups per question.
pub const MAX_LOOKUPS: usize = 4;
/// The turns of the chat that go with a question, so "Und 2025?" works.
pub const HISTORY: usize = 6;

/// The lookups the model may make, with what each takes.
pub const TOOLS: [(&str, &str); 9] = [
    (
        "einnahmen_ausgaben",
        "Einnahmen, Ausgaben und Saldo eines Jahres (year) oder Monats (year, month), mit den größten Kategorien",
    ),
    (
        "kategorie",
        "Ausgaben oder Einnahmen einer Kategorie (query) in einem Jahr (year)",
    ),
    (
        "partei",
        "Was an einen Zahlungspartner ging oder von ihm kam (query), in einem Jahr (year)",
    ),
    (
        "buchungen",
        "Buchungen mit Wörtern (query) in Partner oder Verwendungszweck, optional year und month",
    ),
    (
        "dokumente",
        "Dokumente mit Wörtern (query) im Namen oder im Text",
    ),
    (
        "gehalt",
        "Brutto, Netto und Abzüge der Lohnabrechnungen eines Jahres (year)",
    ),
    (
        "regelmaessig",
        "Laufende regelmäßige Zahlungen wie Abos, Verträge, Miete",
    ),
    (
        "steuern",
        "Was für die Steuererklärung eines Jahres (year) gesammelt ist",
    ),
    (
        "kontostand",
        "Der letzte bekannte Kontostand jedes Kontos und aller zusammen",
    ),
];

/// A document or a booking an answer rests on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Source {
    Document(i64),
    Booking(i64),
}

/// What one lookup returned: its lines for the model, the amounts as code wrote them, its sources.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Found {
    pub lines: Vec<String>,
    pub amounts: BTreeSet<String>,
    pub sources: Vec<Source>,
    pub empty: bool,
    /// The lookup touched taxes: the answer gets the notice.
    pub taxes: bool,
}

impl Found {
    fn money(&mut self, cents: i64) -> String {
        let text = Cents::new(cents).to_string();
        self.amounts.insert(normal(&text));
        text
    }

    fn line(&mut self, line: String) {
        self.lines.push(line);
    }
}

/// An amount as the check compares it: without sign and spaces.
fn normal(amount: &str) -> String {
    amount
        .trim_start_matches(['-', '+', '−'])
        .replace([' ', '\u{a0}'], "")
}

static AMOUNT: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"\d{1,3}(?:\.\d{3})*,\d{2}\s?€")
        .unwrap_or_else(|_| unreachable!("a fixed pattern"))
});

/// The amounts an answer names that no lookup returned.
pub fn unchecked(answer: &str, amounts: &BTreeSet<String>) -> Vec<String> {
    AMOUNT
        .find_iter(answer)
        .map(|found| normal(found.as_str()))
        .filter(|amount| !amounts.contains(amount))
        .collect()
}

/// What the model asked for.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct Step {
    pub action: String,
    #[serde(default)]
    pub tool: Option<String>,
    #[serde(default)]
    pub year: Option<i32>,
    #[serde(default)]
    pub month: Option<i8>,
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
}

fn words(query: Option<&str>) -> Vec<String> {
    query
        .unwrap_or("")
        .split_whitespace()
        .map(|word| {
            word.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|word| word.chars().count() >= 2)
        .take(3)
        .collect()
}

fn contains_all(text: &str, words: &[String]) -> bool {
    let text = text.to_lowercase();
    words.iter().all(|word| text.contains(word.as_str()))
}

/// Runs one lookup on the read-only connection.
pub fn lookup(conn: &Connection, step: &Step, today: Date) -> rusqlite::Result<Found> {
    let year = step.year.unwrap_or_else(|| i32::from(today.year()));
    let mut found = Found::default();
    match step.tool.as_deref().unwrap_or("") {
        "einnahmen_ausgaben" => {
            let evaluated = crate::evaluations::year(conn, year)?;
            match step.month.filter(|month| (1..=12).contains(month)) {
                Some(month) => {
                    let at = usize::from(month.unsigned_abs()) - 1;
                    let (income, spending) = (evaluated.income[at], evaluated.spending[at]);
                    found.empty = income.count + spending.count == 0;
                    let (i, s, b) = (
                        found.money(income.cents),
                        found.money(spending.cents),
                        found.money(income.cents - spending.cents),
                    );
                    found.line(format!(
                        "{month:02}/{year}: Einnahmen {i}, Ausgaben {s}, Saldo {b}"
                    ));
                }
                None => {
                    let income = evaluated.total(crate::evaluations::Side::Income);
                    let spending = evaluated.total(crate::evaluations::Side::Spending);
                    found.empty = income.count + spending.count == 0;
                    let (i, s, b) = (
                        found.money(income.cents),
                        found.money(spending.cents),
                        found.money(income.cents - spending.cents),
                    );
                    found.line(format!("{year}: Einnahmen {i} ({} Buchungen), Ausgaben {s} ({} Buchungen), Saldo {b}", income.count, spending.count));
                    for part in evaluated
                        .categories
                        .iter()
                        .filter(|part| part.side == crate::evaluations::Side::Spending)
                        .take(5)
                    {
                        let cents = found.money(part.sum.cents);
                        let name = part.name.clone().unwrap_or_else(|| "Ohne Kategorie".into());
                        found.line(format!("Ausgaben {name}: {cents}"));
                    }
                }
            }
        }
        "kategorie" | "partei" => {
            let evaluated = crate::evaluations::year(conn, year)?;
            let wanted = words(step.query.as_deref());
            let parts = if step.tool.as_deref() == Some("kategorie") {
                &evaluated.categories
            } else {
                &evaluated.parties
            };
            let matching: Vec<_> = parts
                .iter()
                .filter(|part| {
                    part.name
                        .as_deref()
                        .is_some_and(|name| !wanted.is_empty() && contains_all(name, &wanted))
                })
                .collect();
            found.empty = matching.is_empty();
            for part in matching {
                let side = if part.side == crate::evaluations::Side::Income {
                    "Einnahmen"
                } else {
                    "Ausgaben"
                };
                let cents = found.money(part.sum.cents);
                found.line(format!(
                    "{year} {side} {}: {cents} in {} Buchungen",
                    part.name.clone().unwrap_or_default(),
                    part.sum.count
                ));
            }
            if found.empty && step.tool.as_deref() == Some("kategorie") {
                let names: Vec<String> = crate::categorize::categories(conn)?
                    .into_iter()
                    .map(|(_, name, _)| name)
                    .collect();
                found.line(format!(
                    "Keine passende Kategorie. Bekannte Kategorien: {}",
                    names.join(", ")
                ));
            }
        }
        "buchungen" => {
            let wanted = words(step.query.as_deref());
            let prefix = match step.month.filter(|month| (1..=12).contains(month)) {
                Some(month) => format!("{year:04}-{month:02}"),
                None => step
                    .year
                    .map(|year| format!("{year:04}"))
                    .unwrap_or_default(),
            };
            let rows: Vec<(i64, String, i64, String, String)> = conn
                .prepare(
                    "SELECT b.id, b.day, b.amount_cents, coalesce(p.name, b.other_side, ''), b.purpose FROM bookings b
                     LEFT JOIN parties p ON p.id = b.party_id
                     WHERE b.replaced_by IS NULL AND b.day LIKE ?1 || '%' ORDER BY b.day DESC, b.id DESC",
                )?
                .query_map([prefix], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)))?
                .collect::<rusqlite::Result<_>>()?;
            let hits: Vec<_> = rows
                .into_iter()
                .filter(|(_, _, _, party, purpose)| {
                    !wanted.is_empty() && contains_all(&format!("{party} {purpose}"), &wanted)
                })
                .collect();
            found.empty = hits.is_empty();
            let sum: i64 = hits.iter().map(|(_, _, cents, _, _)| cents).sum();
            if !hits.is_empty() {
                let total = found.money(sum);
                found.line(format!("{} Buchungen, zusammen {total}", hits.len()));
            }
            for (id, day, cents, party, _) in hits.iter().take(10) {
                let amount = found.money(*cents);
                found.line(format!("{day} {party}: {amount}"));
                found.sources.push(Source::Booking(*id));
            }
        }
        "dokumente" => {
            let wanted = words(step.query.as_deref());
            let documents: Vec<(i64, String, String, String)> = conn
                .prepare("SELECT id, coalesce(name, original_name), kind, state FROM documents ORDER BY id DESC")?
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))?
                .collect::<rusqlite::Result<_>>()?;
            let mut hits = 0;
            for (id, name, kind, state) in documents {
                if wanted.is_empty() || hits >= 8 {
                    break;
                }
                let text: String = conn
                    .prepare(
                        "SELECT group_concat(text, ' ') FROM page_lines WHERE document_id = ?1",
                    )?
                    .query_row([id], |row| row.get::<_, Option<String>>(0))?
                    .unwrap_or_default();
                if !contains_all(&format!("{name} {text}"), &wanted) {
                    continue;
                }
                hits += 1;
                let chosen = crate::checks::chosen(&crate::checks::values(conn, id)?);
                let gross = chosen
                    .get("gross")
                    .and_then(|seen| seen.value.parse::<i64>().ok());
                let mut line = format!("{name} ({kind})");
                if let Some(gross) = gross {
                    let amount = found.money(gross);
                    line.push_str(&format!(", {amount}"));
                }
                if let Some(date) = chosen.get("invoice_date") {
                    line.push_str(&format!(", vom {}", date.value));
                }
                if state != "filed" {
                    line.push_str(", wartet auf Prüfung");
                }
                found.line(line);
                found.sources.push(Source::Document(id));
            }
            found.empty = hits == 0;
        }
        "gehalt" => {
            let years = crate::salary::years(conn)?;
            let this = years.iter().find(|one| one.year == year);
            found.empty = this.is_none();
            if let Some(this) = this {
                for (field, label) in crate::salary::FIELDS {
                    if let Some(cents) = this.totals.get(field) {
                        let amount = found.money(*cents);
                        found.line(format!("{year} {label} zusammen: {amount}"));
                    }
                }
                found.line(format!(
                    "{} Monate, davon {} ohne Abrechnung",
                    this.months.len(),
                    this.missing()
                ));
                for month in &this.months {
                    found
                        .sources
                        .extend(month.payslips.iter().map(|id| Source::Document(*id)));
                }
            }
        }
        "regelmaessig" => {
            let all = crate::recurring::found(conn, today)?;
            let running: Vec<_> = all.iter().filter(|one| !one.ended).collect();
            found.empty = running.is_empty();
            for one in running {
                let (amount, year_amount) = (found.money(one.cents), found.money(one.per_year));
                let mut line = format!(
                    "{}: {} {amount}, im Jahr {year_amount}",
                    one.name,
                    one.rhythm.name()
                );
                if let Some(change) = one.change_text() {
                    found
                        .amounts
                        .extend(AMOUNT.find_iter(&change).map(|m| normal(m.as_str())));
                    line.push_str(&format!(", {change}"));
                }
                found.line(line);
                found
                    .sources
                    .extend(one.bookings.last().map(|id| Source::Booking(*id)));
            }
        }
        "steuern" => {
            let collected = crate::taxes::year(conn, year)?;
            found.taxes = true;
            found.empty = collected.sections.is_empty();
            for section in &collected.sections {
                let amount = found.money(section.counted);
                found.line(format!(
                    "{} ({}): {amount}",
                    section.category.label, section.category.elster
                ));
                for entry in &section.entries {
                    found.sources.push(match entry.on {
                        crate::taxes::On::Document(id) => Source::Document(id),
                        crate::taxes::On::Booking(id) => Source::Booking(id),
                    });
                }
            }
            if !found.empty {
                let total = found.money(collected.werbungskosten);
                found.line(format!("Werbungskosten zusammen voraussichtlich {total}"));
            }
        }
        "kontostand" => {
            let (accounts, total) = crate::balances::all(conn)?;
            found.empty = accounts.is_empty();
            for account in &accounts {
                if let Some(last) = account.days.last() {
                    let amount = found.money(last.shown().value());
                    found.line(format!("{} am {}: {amount}", account.name, last.day));
                }
            }
            if let Some((day, cents)) = total.last() {
                let amount = found.money(cents.value());
                found.line(format!("Alle Konten am {day}: {amount}"));
            }
        }
        other => {
            found.empty = true;
            found.line(format!("Die Abfrage {other} gibt es nicht."));
        }
    }
    Ok(found)
}

fn schema(answer_only: bool) -> Value {
    let tools: Vec<&str> = TOOLS.iter().map(|(name, _)| *name).collect();
    let actions = if answer_only {
        json!(["answer"])
    } else {
        json!(["lookup", "answer"])
    };
    json!({
        "type": "object",
        "properties": {
            "action": { "enum": actions },
            "tool": { "enum": tools },
            "year": { "type": "integer" },
            "month": { "type": "integer" },
            "query": { "type": "string" },
            "text": { "type": "string" },
        },
        "required": ["action"],
    })
}

fn system(today: Date) -> String {
    let tools: Vec<String> = TOOLS
        .iter()
        .map(|(name, what)| format!("- {name}: {what}"))
        .collect();
    format!(
        "Du beantwortest Fragen zum Finanzarchiv eines Privathaushalts, auf Deutsch, kurz und freundlich. \
Heute ist der {}. Ohne Jahr ist das laufende Jahr {} gemeint. \
Schlage zuerst nach (action lookup mit tool und seinen Angaben), dann antworte (action answer mit text). \
Nenne nur Beträge, die genau so in den Ergebnissen stehen, rechne nicht selbst. \
Gib keine Steuer-, Anlage- oder Finanzberatung.\nAbfragen:\n{}",
        today.strftime("%d.%m.%Y"),
        today.year(),
        tools.join("\n")
    )
}

/// The answer of the chat, checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    pub text: String,
    pub sources: Vec<Source>,
    /// The lookups made, as "tool(query, year, month)".
    pub lookups: Vec<String>,
}

fn ask_step(
    model: &dyn LocalModel,
    system: &str,
    prompt: &str,
    answer_only: bool,
) -> Result<Step, ModelError> {
    let schema = schema(answer_only);
    let text = model.ask(&Question {
        system,
        prompt,
        schema: &schema,
        images: &[],
    })?;
    serde_json::from_str(&text).map_err(|error| ModelError::BadAnswer(error.to_string()))
}

/// Answers a question: the model looks up, at most `MAX_LOOKUPS` times, then answers, and code
/// checks the answer. `history` is the chat so far, owner and archive in turn.
pub fn answer(
    conn: &Connection,
    model: &dyn LocalModel,
    question: &str,
    history: &[(String, String)],
    today: Date,
) -> Result<Answer, ModelError> {
    let system = system(today);
    let mut prompt = String::new();
    for (owner, archive) in history.iter().rev().take(HISTORY).rev() {
        prompt.push_str(&format!("Frage: {owner}\nAntwort: {archive}\n"));
    }
    prompt.push_str(&format!("Frage: {question}\n"));
    let mut amounts = BTreeSet::new();
    let mut sources: Vec<Source> = Vec::new();
    let mut lookups = Vec::new();
    let mut all_empty = true;
    let mut taxes = false;
    let mut reminded = false;
    let mut corrected = false;
    let mut pressed = false;
    loop {
        let answer_only = lookups.len() >= MAX_LOOKUPS;
        let step = ask_step(model, &system, &prompt, answer_only)?;
        if step.action == "lookup" && !answer_only {
            let found = lookup(conn, &step, today)
                .map_err(|error| ModelError::BadAnswer(error.to_string()))?;
            let name = format!(
                "{}({}, {}, {})",
                step.tool.clone().unwrap_or_default(),
                step.query.clone().unwrap_or_default(),
                step.year.map(|year| year.to_string()).unwrap_or_default(),
                step.month
                    .map(|month| month.to_string())
                    .unwrap_or_default()
            );
            prompt.push_str(&format!(
                "Ergebnis {name}:\n{}\n",
                if found.lines.is_empty() {
                    "nichts".to_owned()
                } else {
                    found.lines.join("\n")
                }
            ));
            all_empty &= found.empty;
            taxes |= found.taxes;
            amounts.extend(found.amounts);
            for source in found.sources {
                if !sources.contains(&source) {
                    sources.push(source);
                }
            }
            lookups.push(name);
            continue;
        }
        let text = step.text.unwrap_or_default().trim().to_owned();
        // A step without a text, a lookup past the last one included, is asked once more.
        if text.is_empty() {
            if !pressed {
                pressed = true;
                prompt.push_str(
                    "Hinweis: Antworte jetzt mit action answer und einem text.
",
                );
                continue;
            }
            return Ok(Answer {
                text: NO_SURE_NUMBERS.to_owned(),
                sources,
                lookups,
            });
        }
        if lookups.is_empty() {
            // A number without a lookup is refused: once a reminder, then the answer goes without it.
            if !reminded && !unchecked(&text, &amounts).is_empty() {
                reminded = true;
                prompt.push_str("Hinweis: Schlage zuerst nach, bevor du Beträge nennst.\n");
                continue;
            }
            if !unchecked(&text, &amounts).is_empty() {
                return Ok(Answer {
                    text: NO_SURE_NUMBERS.to_owned(),
                    sources,
                    lookups,
                });
            }
            return Ok(Answer {
                text,
                sources,
                lookups,
            });
        }
        if all_empty {
            return Ok(Answer {
                text: NOTHING_FOUND.to_owned(),
                sources: Vec::new(),
                lookups,
            });
        }
        let wrong = unchecked(&text, &amounts);
        if !wrong.is_empty() {
            if !corrected {
                corrected = true;
                prompt.push_str(&format!(
                    "Deine Antwort war: {text}\nDiese Beträge stehen in keinem Ergebnis: {}. Antworte noch einmal und übernimm die Beträge genau.\n",
                    wrong.join(", ")
                ));
                continue;
            }
            return Ok(Answer {
                text: NO_SURE_NUMBERS.to_owned(),
                sources,
                lookups,
            });
        }
        let text = if taxes {
            format!("{text}\n\n{}", crate::taxes::CHECK_NOTICE)
        } else {
            text
        };
        return Ok(Answer {
            text,
            sources,
            lookups,
        });
    }
}

/// The archive opened read-only for the lookups: SQLite refuses every write on it.
pub fn read_only(path: &Path) -> rusqlite::Result<Connection> {
    Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
}

/// A chat of the history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chat {
    pub id: i64,
    pub title: String,
    pub created_at: String,
}

/// One message of a chat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub id: i64,
    /// "owner" or "archive".
    pub role: String,
    pub text: String,
    pub sources: Vec<Source>,
    pub asked_at: String,
}

fn sources_json(sources: &[Source]) -> String {
    let list: Vec<Value> = sources
        .iter()
        .map(|source| match source {
            Source::Document(id) => json!({ "document": id }),
            Source::Booking(id) => json!({ "booking": id }),
        })
        .collect();
    Value::Array(list).to_string()
}

fn sources_of(text: &str) -> Vec<Source> {
    serde_json::from_str::<Vec<Value>>(text)
        .unwrap_or_default()
        .iter()
        .filter_map(|value| {
            value["document"]
                .as_i64()
                .map(Source::Document)
                .or_else(|| value["booking"].as_i64().map(Source::Booking))
        })
        .collect()
}

/// A new chat, titled by its first question.
pub fn new_chat(conn: &Connection, question: &str, now: Timestamp) -> rusqlite::Result<i64> {
    let title: String = question.chars().take(60).collect();
    conn.execute(
        "INSERT INTO chats (title, created_at) VALUES (?1, ?2)",
        params![title, now.to_string()],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn add(
    conn: &Connection,
    chat: i64,
    role: &str,
    text: &str,
    sources: &[Source],
    now: Timestamp,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO chat_messages (chat_id, role, text, sources, asked_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![chat, role, text, sources_json(sources), now.to_string()],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn chats(conn: &Connection) -> rusqlite::Result<Vec<Chat>> {
    conn.prepare("SELECT id, title, created_at FROM chats ORDER BY id DESC")?
        .query_map([], |row| {
            Ok(Chat {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
            })
        })?
        .collect()
}

pub fn messages(conn: &Connection, chat: i64) -> rusqlite::Result<Vec<Message>> {
    conn.prepare("SELECT id, role, text, sources, asked_at FROM chat_messages WHERE chat_id = ?1 ORDER BY id")?
        .query_map([chat], |row| {
            Ok(Message {
                id: row.get(0)?,
                role: row.get(1)?,
                text: row.get(2)?,
                sources: sources_of(&row.get::<_, String>(3)?),
                asked_at: row.get(4)?,
            })
        })?
        .collect()
}

/// The turns of a chat as `answer` takes them: each question with the answer after it.
pub fn history(conn: &Connection, chat: i64) -> rusqlite::Result<Vec<(String, String)>> {
    let messages = messages(conn, chat)?;
    Ok(messages
        .windows(2)
        .filter(|pair| pair[0].role == "owner" && pair[1].role == "archive")
        .map(|pair| (pair[0].text.clone(), pair[1].text.clone()))
        .collect())
}

/// Deletes a chat and its messages.
pub fn delete(conn: &Connection, chat: i64) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM chat_messages WHERE chat_id = ?1", [chat])?;
    conn.execute("DELETE FROM chats WHERE id = ?1", [chat])?;
    Ok(())
}

#[cfg(test)]
mod tests;
