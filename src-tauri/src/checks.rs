//! The checks that decide what is filed by itself (FA-21, #16). Each check has a stable id, its
//! result is a row of `field_checks`, and a document that passes all of them is filed with a row of
//! `decisions`: the rule, the values it saw and how to undo it (knowledge/decision-checks.md).

use std::collections::BTreeMap;

use jiff::Timestamp;
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::json;

use crate::kinds::Kind;
use crate::money::Cents;

/// The rule that files a document whose checks all passed.
pub const RULE_CHECKED: &str = "file.checks-passed:1";
/// The rule that filed a statement without checks in stage 2, the owner's choice of 2026-11-25.
/// Since #20 a statement passes `check:balance` instead, and these decisions are superseded.
pub const RULE_STATEMENT: &str = "file.statement:1";

/// The rule that files a document after the owner answered for what a check could not see (#17).
pub const RULE_OWNER: &str = "file.owner-answered:1";

/// The field under which the owner accepts a check that failed, `accept:check:sum` for example.
pub const ACCEPT: &str = "accept:";

/// Whether the owner accepted the failed check (a value of the owner under `accept:<check>`).
fn accepted(values: &BTreeMap<String, Vec<Seen>>, check: &str) -> bool {
    values.get(&format!("{ACCEPT}{check}")).is_some_and(|seen| {
        seen.iter()
            .any(|seen| seen.family == "owner" && seen.value == "1")
    })
}

/// The ids of the checks. Every result of #16 is stored under one of them, the field beside it.
pub const KIND: &str = "check:kind";
pub const PRESENT: &str = "check:present";
pub const SOURCE: &str = "check:source";
pub const AGREE: &str = "check:agree";
pub const SUM: &str = "check:sum";
pub const DUPLICATE: &str = "check:duplicate";
pub const UNDONE: &str = "check:undone";
pub const BALANCE: &str = "check:balance";

/// Readers in the order their value counts: the owner, the sender's own data, code, the text
/// model, the image.
const FAMILIES: [&str; 5] = ["owner", "e-invoice", "code", "model", "image"];

/// One value a reader stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seen {
    pub family: &'static str,
    pub printed: String,
    pub value: String,
    pub on_a_line: bool,
}

/// One result of a check, with the German sentence the owner reads under "Warum?".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub check: &'static str,
    pub field: Option<&'static str>,
    pub passed: bool,
    pub detail: String,
}

impl Outcome {
    fn of(check: &'static str, field: Option<&'static str>, passed: bool, detail: String) -> Self {
        Self {
            check,
            field,
            passed,
            detail,
        }
    }
}

fn family(reader: &str) -> Option<&'static str> {
    let name = reader.split_once(':').map_or(reader, |(family, _)| family);
    FAMILIES.into_iter().find(|family| *family == name)
}

fn label(field: &str) -> &'static str {
    match field {
        "gross" => "Bruttobetrag",
        "net" => "Nettobetrag",
        "vat" => "Umsatzsteuer",
        "vat_rate" => "Steuersatz",
        "brutto" => "Brutto",
        "netto" => "Netto",
        "sender" => "Absender",
        "invoice_number" => "Rechnungsnummer",
        "employer" => "Arbeitgeber",
        "period" => "Monat",
        _ => "Wert",
    }
}

fn reader_label(family: &str) -> &'static str {
    match family {
        "owner" => "du",
        "e-invoice" => "die E-Rechnung",
        "code" => "der Code",
        "model" => "das Modell",
        _ => "das Bild",
    }
}

/// Every value of a document by field, the strongest reader first.
pub fn values(conn: &Connection, document: i64) -> rusqlite::Result<BTreeMap<String, Vec<Seen>>> {
    let mut statement = conn.prepare(
        "SELECT field, reader, printed, value, page IS NOT NULL AND top IS NOT NULL OR element IS NOT NULL
         FROM field_values WHERE document_id = ?1 ORDER BY id",
    )?;
    let rows = statement.query_map([document], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, bool>(4)?,
        ))
    })?;
    let mut by_field: BTreeMap<String, Vec<Seen>> = BTreeMap::new();
    for row in rows {
        let (field, reader, printed, value, on_a_line) = row?;
        let Some(family) = family(&reader) else {
            continue;
        };
        by_field.entry(field).or_default().push(Seen {
            family,
            printed,
            value,
            on_a_line: on_a_line || family == "owner",
        });
    }
    for seen in by_field.values_mut() {
        seen.sort_by_key(|seen| FAMILIES.iter().position(|family| *family == seen.family));
    }
    Ok(by_field)
}

/// The value that counts for each field: the one of the strongest reader.
pub fn chosen(values: &BTreeMap<String, Vec<Seen>>) -> BTreeMap<String, Seen> {
    values
        .iter()
        .filter_map(|(field, seen)| Some((field.clone(), seen.first()?.clone())))
        .collect()
}

fn cents(chosen: &BTreeMap<String, Seen>, field: &str) -> Option<Cents> {
    chosen
        .get(field)
        .and_then(|seen| seen.value.parse().ok())
        .map(Cents::new)
}

/// The values a decision rests on: the gross of an invoice, the Brutto and Netto of a payslip.
pub fn decisive(kind: Kind) -> &'static [&'static str] {
    crate::readers::image::decisive(kind)
}

/// The value is there, stands in the text or the XML, and two independent readers agree on it.
fn decisive_checks(field: &'static str, seen: Option<&Vec<Seen>>) -> Vec<Outcome> {
    let name = label(field);
    let Some(first) = seen.and_then(|seen| seen.first()) else {
        return vec![Outcome::of(
            PRESENT,
            Some(field),
            false,
            format!("Der {name} fehlt, kein Leser hat ihn gefunden."),
        )];
    };
    let seen = seen.map_or(&[][..], Vec::as_slice);
    let mut outcomes = vec![Outcome::of(
        PRESENT,
        Some(field),
        true,
        format!("Der {name} ist {}.", first.printed),
    )];
    outcomes.push(if first.on_a_line {
        Outcome::of(
            SOURCE,
            Some(field),
            true,
            format!("Der {name} steht so im Dokument."),
        )
    } else {
        Outcome::of(
            SOURCE,
            Some(field),
            false,
            format!("Der {name} wurde nur aus dem Bild der Seite gelesen."),
        )
    });
    let differing = seen.iter().find(|other| other.value != first.value);
    let agreeing: Vec<&str> = {
        let mut families: Vec<&str> = seen
            .iter()
            .filter(|other| other.value == first.value)
            .map(|other| other.family)
            .collect();
        families.dedup();
        families
    };
    // The owner's value stands above every reading, the readings beside it no longer count.
    let differing = differing.filter(|_| first.family != "owner");
    outcomes.push(match differing {
        Some(other) => Outcome::of(
            AGREE,
            Some(field),
            false,
            format!(
                "Beim {name} liest {} {}, {} aber {}.",
                reader_label(first.family),
                first.printed,
                reader_label(other.family),
                other.printed
            ),
        ),
        None if first.family == "owner" => Outcome::of(
            AGREE,
            Some(field),
            true,
            format!("Den {name} hast du bestätigt."),
        ),
        None if agreeing.len() >= 2 => Outcome::of(
            AGREE,
            Some(field),
            true,
            format!(
                "Den {name} lesen {} und {} gleich.",
                reader_label(agreeing[0]),
                reader_label(agreeing[1])
            ),
        ),
        None => Outcome::of(
            AGREE,
            Some(field),
            false,
            format!(
                "Den {name} hat nur {} gelesen, ein zweiter Leser fehlt.",
                reader_label(first.family)
            ),
        ),
    });
    outcomes
}

/// Net plus VAT is the gross to the cent, and the VAT fits the rate within a cent.
pub fn invoice_sum(
    net: Option<Cents>,
    vat: Option<Cents>,
    gross: Cents,
    rate: Option<u32>,
) -> Outcome {
    let outcome = |passed, detail| Outcome::of(SUM, Some("gross"), passed, detail);
    match (net, vat) {
        (None, None) => outcome(
            true,
            "Netto und Steuer sind nicht ausgewiesen, es zählt der Bruttobetrag.".into(),
        ),
        (Some(net), Some(vat)) => {
            if net.checked_add(vat) != Some(gross) {
                return outcome(
                    false,
                    format!("Netto {net} plus Steuer {vat} ergibt nicht den Bruttobetrag {gross}."),
                );
            }
            if let Some(rate) = rate {
                let expected = (i128::from(net.value()) * i128::from(rate) + 50).div_euclid(100);
                if (expected - i128::from(vat.value())).abs() > 1 {
                    return outcome(
                        false,
                        format!("Die Steuer {vat} passt nicht zu {rate} % von {net}."),
                    );
                }
            }
            outcome(
                true,
                format!("Netto {net} plus Steuer {vat} ergibt {gross}."),
            )
        }
        _ => outcome(
            false,
            "Netto oder Steuer fehlt, die Summe lässt sich nicht prüfen.".into(),
        ),
    }
}

/// The deductions of a payslip that come off the Brutto.
pub const DEDUCTIONS: &[&str] = &[
    "lohnsteuer",
    "solidaritaetszuschlag",
    "kirchensteuer",
    "krankenversicherung",
    "rentenversicherung",
    "arbeitslosenversicherung",
    "pflegeversicherung",
];

/// The Brutto minus every deduction is the Netto to the cent.
pub fn payslip_sum(brutto: Cents, deductions: &[Cents], netto: Cents) -> Outcome {
    let total = deductions
        .iter()
        .try_fold(Cents::ZERO, |sum, deduction| sum.checked_add(*deduction));
    let rest = total.and_then(|total| brutto.checked_sub(total));
    match (total, rest) {
        (Some(total), Some(rest)) if rest == netto => Outcome::of(
            SUM,
            Some("netto"),
            true,
            format!("Brutto {brutto} minus Abzüge {total} ergibt das Netto {netto}."),
        ),
        (Some(total), Some(rest)) => Outcome::of(
            SUM,
            Some("netto"),
            false,
            format!("Brutto {brutto} minus Abzüge {total} ergibt {rest}, gedruckt ist {netto}."),
        ),
        _ => Outcome::of(
            SUM,
            Some("netto"),
            false,
            "Die Abzüge sind zu groß zum Rechnen.".into(),
        ),
    }
}

fn folded(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// The keys under which two documents are the same: sender, number and gross of an invoice (the
/// date instead of a missing number), employer and month of a payslip.
fn identity(kind: Kind, chosen: &BTreeMap<String, Seen>) -> Option<Vec<String>> {
    let get = |field: &str| chosen.get(field).map(|seen| folded(&seen.value));
    match kind {
        Kind::Invoice | Kind::Receipt => {
            let second = get("invoice_number").or_else(|| get("invoice_date"))?;
            Some(vec![get("sender")?, second, get("gross")?])
        }
        Kind::Payslip => Some(vec![get("employer")?, get("period")?]),
        _ => None,
    }
}

/// Another document of the same kind with the same identity, if there is one.
fn duplicate_of(
    conn: &Connection,
    document: i64,
    kind: Kind,
    key: &[String],
) -> rusqlite::Result<Option<i64>> {
    let others: Vec<i64> = conn
        .prepare("SELECT id FROM documents WHERE id <> ?1 AND kind = ?2 ORDER BY id")?
        .query_map(params![document, kind.code()], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    for other in others {
        if identity(kind, &chosen(&values(conn, other)?)).as_deref() == Some(key) {
            return Ok(Some(other));
        }
    }
    Ok(None)
}

/// Every check of a document, and whether it is filed.
pub struct Verdict {
    pub rule: Option<&'static str>,
    pub outcomes: Vec<Outcome>,
    pub chosen: BTreeMap<String, Seen>,
}

impl Verdict {
    pub fn filed(&self) -> bool {
        self.rule.is_some()
    }
}

/// Runs the checks of a document without writing anything.
pub fn judge(conn: &Connection, document: i64) -> rusqlite::Result<Verdict> {
    let kind: String = conn.query_row(
        "SELECT kind FROM documents WHERE id = ?1",
        [document],
        |row| row.get(0),
    )?;
    let kind = Kind::from_code(&kind);
    let values = values(conn, document)?;
    let chosen = chosen(&values);
    let undone: bool = conn
        .query_row(
            "SELECT 1 FROM decisions WHERE document_id = ?1 AND undone_at IS NOT NULL AND rule <> ?2 LIMIT 1",
            params![document, RULE_STATEMENT],
            |_| Ok(true),
        )
        .optional()?
        .unwrap_or(false);
    let mut outcomes = Vec::new();
    if undone {
        outcomes.push(Outcome::of(
            UNDONE,
            None,
            false,
            "Du hast das automatische Ablegen rückgängig gemacht, das Dokument wartet auf dich."
                .into(),
        ));
    }
    match kind {
        Kind::Statement => {
            outcomes.push(Outcome::of(
                KIND,
                None,
                true,
                "Das Dokument ist ein Kontoauszug.".into(),
            ));
            // #20: a statement is filed when its bookings lead from the old to the new balance.
            outcomes.push(match crate::statements::difference_of(conn, document)? {
                None => Outcome::of(
                    BALANCE,
                    None,
                    false,
                    "Den Aufbau dieses Kontoauszugs kennt das Archiv nicht, seine Buchungen ließen sich nicht lesen.".into(),
                ),
                Some((opening, sum, closing, count)) => {
                    let reached = opening.checked_add(sum);
                    if reached == Some(closing) {
                        Outcome::of(
                            BALANCE,
                            None,
                            true,
                            format!("Alter Kontostand {opening} und {count} Buchungen ergeben den neuen Kontostand {closing}."),
                        )
                    } else {
                        let reached = reached.map_or_else(|| "zu viel".to_owned(), |reached| reached.to_string());
                        Outcome::of(
                            BALANCE,
                            None,
                            false,
                            format!("Alter Kontostand {opening} und {count} Buchungen ergeben {reached}, gedruckt ist {closing}."),
                        )
                    }
                }
            });
        }
        Kind::Invoice | Kind::Receipt | Kind::Payslip => {
            outcomes.push(Outcome::of(
                KIND,
                None,
                true,
                format!(
                    "Das Dokument ist {}.",
                    match kind {
                        Kind::Payslip => "eine Gehaltsabrechnung",
                        Kind::Receipt => "ein Zahlungsbeleg",
                        _ => "eine Rechnung",
                    }
                ),
            ));
            for field in decisive(kind) {
                outcomes.extend(decisive_checks(field, values.get(*field)));
            }
            match kind {
                Kind::Payslip => {
                    if let (Some(brutto), Some(netto)) =
                        (cents(&chosen, "brutto"), cents(&chosen, "netto"))
                    {
                        let deductions: Vec<Cents> = DEDUCTIONS
                            .iter()
                            .filter_map(|field| cents(&chosen, field))
                            .collect();
                        outcomes.push(payslip_sum(brutto, &deductions, netto));
                    }
                }
                _ => {
                    if let Some(gross) = cents(&chosen, "gross") {
                        let rate = chosen
                            .get("vat_rate")
                            .and_then(|seen| seen.value.parse().ok());
                        outcomes.push(invoice_sum(
                            cents(&chosen, "net"),
                            cents(&chosen, "vat"),
                            gross,
                            rate,
                        ));
                    }
                }
            }
            outcomes.push(match identity(kind, &chosen) {
                Some(key) => match duplicate_of(conn, document, kind, &key)? {
                    Some(_) => Outcome::of(DUPLICATE, None, false, "Ein anderes Dokument hat denselben Absender, dieselbe Nummer und denselben Betrag.".into()),
                    None => Outcome::of(DUPLICATE, None, true, "Kein anderes Dokument gleicht ihm.".into()),
                },
                None => Outcome::of(DUPLICATE, None, false, "Ohne Absender, Nummer oder Datum lässt sich ein Doppel nicht ausschließen.".into()),
            });
        }
        Kind::Other | Kind::Unknown => {
            outcomes.push(Outcome::of(
                KIND,
                None,
                false,
                "Die Art des Dokuments ist unklar.".into(),
            ));
        }
    }
    // What the owner answered in "Prüfen" (#17) stands above a check that cannot see it.
    for outcome in outcomes.iter_mut().filter(|outcome| !outcome.passed) {
        if accepted(&values, outcome.check) {
            outcome.passed = true;
            outcome.detail = format!("Von dir bestätigt: {}", outcome.detail);
        }
    }
    let passed = outcomes.iter().all(|outcome| outcome.passed);
    let rule = passed.then_some(if values.keys().any(|field| field.starts_with(ACCEPT)) {
        RULE_OWNER
    } else {
        RULE_CHECKED
    });
    Ok(Verdict {
        rule,
        outcomes,
        chosen,
    })
}

/// Runs the checks, stores their results, and files the document with a decision or lets it wait.
pub fn decide(conn: &Connection, document: i64, now: Timestamp) -> rusqlite::Result<Verdict> {
    let verdict = judge(conn, document)?;
    conn.execute(
        "DELETE FROM field_checks WHERE document_id = ?1 AND check_id LIKE 'check:%'",
        [document],
    )?;
    for outcome in &verdict.outcomes {
        conn.execute(
            "INSERT INTO field_checks (document_id, field, check_id, passed, detail, checked_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![document, outcome.field, outcome.check, outcome.passed, outcome.detail, now.to_string()],
        )?;
    }
    let state = if verdict.filed() { "filed" } else { "waiting" };
    conn.execute(
        "UPDATE documents SET state = ?2 WHERE id = ?1",
        params![document, state],
    )?;
    if let Some(rule) = verdict.rule {
        let inputs: serde_json::Map<String, serde_json::Value> = verdict
            .chosen
            .iter()
            .map(|(field, seen)| {
                (
                    field.clone(),
                    json!({ "value": seen.value, "printed": seen.printed, "reader": seen.family }),
                )
            })
            .collect();
        conn.execute(
            "INSERT INTO decisions (document_id, rule, inputs, change, undo, decided_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                document,
                rule,
                serde_json::Value::Object(inputs).to_string(),
                json!({ "state": ["open", "filed"] }).to_string(),
                json!({ "state": "waiting" }).to_string(),
                now.to_string()
            ],
        )?;
    }
    if verdict.filed() {
        crate::naming::name_document(conn, document)?;
    }
    Ok(verdict)
}

/// Undoes the automatic filing of a document: it waits for the owner and is not filed by itself
/// again. Returns whether there was a decision to undo.
pub fn undo(conn: &Connection, document: i64, now: Timestamp) -> rusqlite::Result<bool> {
    let tx = conn.unchecked_transaction()?;
    let changed = tx.execute(
        "UPDATE decisions SET undone_at = ?2 WHERE id = (SELECT max(id) FROM decisions WHERE document_id = ?1 AND undone_at IS NULL)",
        params![document, now.to_string()],
    )?;
    if changed > 0 {
        tx.execute(
            "UPDATE documents SET state = 'waiting' WHERE id = ?1",
            [document],
        )?;
        tx.execute(
            "INSERT INTO field_checks (document_id, field, check_id, passed, detail, checked_at) VALUES (?1, NULL, ?2, 0, ?3, ?4)",
            params![document, UNDONE, "Du hast das automatische Ablegen rückgängig gemacht, das Dokument wartet auf dich.", now.to_string()],
        )?;
    }
    tx.commit()?;
    Ok(changed > 0)
}

/// The statements stage 2 filed without checks (`file.statement:1`) are checked again once their
/// bookings are read (#20): the old decision is marked undone, which the check `check:undone` does
/// not count, and the document waits for a job at the stage `read`. Returns how many.
pub fn supersede_statement_rule(conn: &Connection, now: Timestamp) -> rusqlite::Result<usize> {
    let tx = conn.unchecked_transaction()?;
    let documents: Vec<i64> = tx
        .prepare(
            "SELECT DISTINCT document_id FROM decisions WHERE rule = ?1 AND undone_at IS NULL",
        )?
        .query_map([RULE_STATEMENT], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    tx.execute(
        "UPDATE decisions SET undone_at = ?2 WHERE rule = ?1 AND undone_at IS NULL",
        params![RULE_STATEMENT, now.to_string()],
    )?;
    for document in &documents {
        tx.execute(
            "UPDATE documents SET state = 'open' WHERE id = ?1",
            [document],
        )?;
    }
    tx.commit()?;
    Ok(documents.len())
}

/// The documents of an older archive that were never checked get a job at the stage `read`, so the
/// model reads what code left and the checks decide (#16). Returns how many.
pub fn requeue_unchecked(
    conn: &Connection,
    root: &std::path::Path,
    now: Timestamp,
) -> rusqlite::Result<usize> {
    let documents: Vec<(i64, String)> = conn
        .prepare(
            "SELECT id, original_name FROM documents d WHERE state = 'open' AND text_state IS NOT NULL
             AND NOT EXISTS (SELECT 1 FROM intake_jobs j WHERE j.document_id = d.id AND j.stage NOT IN ('filed', 'waiting', 'failed'))",
        )?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let tx = conn.unchecked_transaction()?;
    for (id, name) in &documents {
        let path = crate::documents::stored_of(&tx, root, *id)?
            .map(|(path, _)| path.to_string_lossy().into_owned())
            .unwrap_or_default();
        tx.execute(
            "INSERT INTO intake_jobs (source, path, name, stage, document_id, created_at, updated_at) VALUES ('dialog', ?1, ?2, 'read', ?3, ?4, ?4)",
            params![path, name, id, now.to_string()],
        )?;
    }
    tx.commit()?;
    Ok(documents.len())
}

/// What the window shows of a document's checks: its state, the decision that filed it, every check.
pub struct Shown {
    pub state: String,
    pub rule: Option<String>,
    pub decided_at: Option<String>,
    pub outcomes: Vec<(String, Option<String>, bool, Option<String>)>,
}

pub fn shown(conn: &Connection, document: i64) -> rusqlite::Result<Shown> {
    let state: String = conn.query_row(
        "SELECT state FROM documents WHERE id = ?1",
        [document],
        |row| row.get(0),
    )?;
    let decision: Option<(String, String)> = conn
        .query_row(
            "SELECT rule, decided_at FROM decisions WHERE document_id = ?1 AND undone_at IS NULL ORDER BY id DESC LIMIT 1",
            [document],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let outcomes = conn
        .prepare("SELECT check_id, field, passed, detail FROM field_checks WHERE document_id = ?1 ORDER BY passed, id")?
        .query_map([document], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(Shown {
        state,
        rule: decision.as_ref().map(|(rule, _)| rule.clone()),
        decided_at: decision.map(|(_, at)| at),
        outcomes,
    })
}

#[cfg(test)]
mod tests;
