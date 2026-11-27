//! Taxes for the return (#41, FA-44), taken over from 1.0 (`decision-tax.md` there): a tax
//! assignment on a document or a booking, the amount that counts computed from it, the tax year the
//! year paid (§ 11 EStG), the values the law fixes per year from checked sources, sums by form and
//! section. No tax is computed, no refund, and nothing is presented as settled
//! (knowledge/decision-taxes.md).

use std::collections::BTreeMap;

use jiff::Timestamp;
use jiff::civil::Date;
use rusqlite::{Connection, OptionalExtension, params};

/// Said wherever tax content stands.
pub const CHECK_NOTICE: &str = "Keine Steuerberatung, bitte alles selbst prüfen.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Group {
    Werbungskosten,
    Sonderausgaben,
    Belastungen,
    Haushalt,
    Handwerker,
}

impl Group {
    pub fn label(self) -> &'static str {
        match self {
            Group::Werbungskosten => "Werbungskosten",
            Group::Sonderausgaben => "Sonderausgaben",
            Group::Belastungen => "Außergewöhnliche Belastungen",
            Group::Haushalt => "Haushaltsnahe Dienstleistungen (§ 35a)",
            Group::Handwerker => "Handwerkerleistungen (§ 35a)",
        }
    }
}

/// A tax category: its key, name, group, and form and section as ELSTER shows them as headings.
/// No line numbers, they move every year.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Category {
    pub key: &'static str,
    pub label: &'static str,
    pub group: Group,
    pub elster: &'static str,
}

const fn category(
    key: &'static str,
    label: &'static str,
    group: Group,
    elster: &'static str,
) -> Category {
    Category {
        key,
        label,
        group,
        elster,
    }
}

pub const CATEGORIES: [Category; 20] = [
    category(
        "arbeitsmittel",
        "Arbeitsmittel",
        Group::Werbungskosten,
        "Anlage N · Werbungskosten · Arbeitsmittel",
    ),
    category(
        "fortbildung",
        "Fortbildung und Studium",
        Group::Werbungskosten,
        "Anlage N · Werbungskosten · Fortbildungskosten",
    ),
    category(
        "fachliteratur",
        "Fachliteratur",
        Group::Werbungskosten,
        "Anlage N · Werbungskosten · Arbeitsmittel",
    ),
    category(
        "reisekosten",
        "Fahrten und Reisekosten",
        Group::Werbungskosten,
        "Anlage N · Werbungskosten · Reisekosten",
    ),
    category(
        "bewerbung",
        "Bewerbung",
        Group::Werbungskosten,
        "Anlage N · Werbungskosten · Weitere Werbungskosten",
    ),
    category(
        "berufsverband",
        "Gewerkschaft und Berufsverband",
        Group::Werbungskosten,
        "Anlage N · Werbungskosten · Beiträge zu Berufsverbänden",
    ),
    category(
        "arbeitszimmer",
        "Arbeitszimmer",
        Group::Werbungskosten,
        "Anlage N · Werbungskosten · Häusliches Arbeitszimmer",
    ),
    category(
        "umzug",
        "Beruflicher Umzug",
        Group::Werbungskosten,
        "Anlage N · Werbungskosten · Weitere Werbungskosten",
    ),
    category(
        "doppelter_haushalt",
        "Doppelte Haushaltsführung",
        Group::Werbungskosten,
        "Anlage N · Mehraufwendungen für doppelte Haushaltsführung",
    ),
    category(
        "werbungskosten_sonstige",
        "Sonstige Werbungskosten",
        Group::Werbungskosten,
        "Anlage N · Werbungskosten · Weitere Werbungskosten",
    ),
    category(
        "erstausbildung",
        "Erstausbildung oder Erststudium",
        Group::Sonderausgaben,
        "Hauptvordruck · Sonderausgaben · Aufwendungen für die eigene erstmalige Berufsausbildung",
    ),
    category(
        "spende",
        "Spende oder Mitgliedsbeitrag",
        Group::Sonderausgaben,
        "Anlage Sonderausgaben · Zuwendungen",
    ),
    category(
        "kirchensteuer",
        "Kirchensteuer",
        Group::Sonderausgaben,
        "Anlage Sonderausgaben · Kirchensteuer",
    ),
    category(
        "vorsorge",
        "Versicherung und Vorsorge",
        Group::Sonderausgaben,
        "Anlage Vorsorgeaufwand",
    ),
    category(
        "kinderbetreuung",
        "Kinderbetreuung",
        Group::Sonderausgaben,
        "Anlage Kind · Kinderbetreuungskosten",
    ),
    category(
        "krankheit",
        "Krankheitskosten",
        Group::Belastungen,
        "Anlage Außergewöhnliche Belastungen · Krankheitskosten",
    ),
    category(
        "pflege",
        "Pflege und Betreuung",
        Group::Belastungen,
        "Anlage Außergewöhnliche Belastungen · Pflegekosten",
    ),
    category(
        "belastung_sonstige",
        "Sonstige Belastung",
        Group::Belastungen,
        "Anlage Außergewöhnliche Belastungen · Andere außergewöhnliche Belastungen",
    ),
    category(
        "haushaltsnah",
        "Haushaltsnahe Dienstleistung",
        Group::Haushalt,
        "Anlage Haushaltsnahe Aufwendungen · Haushaltsnahe Dienstleistungen",
    ),
    category(
        "handwerker",
        "Handwerkerleistung",
        Group::Handwerker,
        "Anlage Haushaltsnahe Aufwendungen · Handwerkerleistungen",
    ),
];

pub fn category_of(key: &str) -> Option<&'static Category> {
    CATEGORIES.iter().find(|category| category.key == key)
}

/// The values the law fixes for a year, with where they come from and when they were checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct YearValues {
    pub arbeitnehmer_pauschbetrag: i64,
    /// Cents per kilometre of the one-way distance, for kilometres 1 to 20 and from 21.
    pub entfernung_first_20: i64,
    pub entfernung_from_21: i64,
    /// Highest Entfernungspauschale without an own car.
    pub entfernung_max_ohne_auto: i64,
    pub homeoffice_per_day: i64,
    pub homeoffice_max: i64,
    pub erstausbildung_max: i64,
    /// A donation up to this needs no Zuwendungsbestätigung.
    pub spende_vereinfacht: i64,
    /// Percent of the counted costs, and the highest reduction.
    pub haushalt_percent: i64,
    pub haushalt_max: i64,
    pub handwerker_percent: i64,
    pub handwerker_max: i64,
    pub source: &'static str,
    /// The values of another year taken over without a check of their own.
    pub taken_from: Option<i32>,
}

const COMMON: YearValues = YearValues {
    arbeitnehmer_pauschbetrag: 123_000,
    entfernung_first_20: 30,
    entfernung_from_21: 38,
    entfernung_max_ohne_auto: 450_000,
    homeoffice_per_day: 600,
    homeoffice_max: 126_000,
    erstausbildung_max: 600_000,
    spende_vereinfacht: 30_000,
    haushalt_percent: 20,
    haushalt_max: 400_000,
    handwerker_percent: 20,
    handwerker_max: 120_000,
    source: "geprüft am 31.10.2026",
    taken_from: None,
};

/// The values of a year, `None` for a year the app has none for. 2025 and 2026 were checked on
/// 2026-10-31 (1.0, `decision-tax.md`), 2023 and 2024 carry the values of 2025 and say so.
pub fn year_values(year: i32) -> Option<YearValues> {
    match year {
        2023 | 2024 => Some(YearValues {
            source: "Werte wie 2025, nicht eigens geprüft",
            taken_from: Some(2025),
            ..COMMON
        }),
        2025 => Some(COMMON),
        2026 => Some(YearValues {
            entfernung_first_20: 38,
            source: "Steueränderungsgesetz 2025, Stand 31.10.2026",
            ..COMMON
        }),
        _ => None,
    }
}

/// The tax assignment of a document or a booking as the owner enters it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    pub category: String,
    /// The share used for work or the household, 1 to 100.
    pub share_percent: i64,
    /// What an employer, an insurance or someone else paid back.
    pub refunded_cents: i64,
    /// For craftsmen: the labour share with travel, the only part § 35a counts.
    pub labour_cents: Option<i64>,
}

/// What counts: the amount less what was paid back, times the share, for craftsmen the labour
/// share only. Never below zero, rounded to the cent.
pub fn counted(gross: i64, assignment: &Assignment) -> i64 {
    let group = category_of(&assignment.category).map(|category| category.group);
    let base = if group == Some(Group::Handwerker) {
        assignment.labour_cents.unwrap_or(0).min(gross)
    } else {
        gross
    };
    let after = (base - assignment.refunded_cents).max(0);
    (after * assignment.share_percent + 50) / 100
}

#[derive(Debug, thiserror::Error)]
pub enum TaxError {
    #[error("Diese Steuerkategorie gibt es nicht.")]
    UnknownCategory,
    #[error("Der Anteil liegt zwischen 1 und 100 Prozent.")]
    Share,
    #[error("Die Zahlung ist schon über ihren Beleg zugeordnet.")]
    ThroughDocument,
    #[error("Der Beleg ist schon über seine Zahlung zugeordnet.")]
    ThroughBooking,
    #[error(transparent)]
    Database(#[from] rusqlite::Error),
}

/// What an assignment is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum On {
    Document(i64),
    Booking(i64),
}

fn linked_bookings(conn: &Connection, document: i64) -> rusqlite::Result<Vec<i64>> {
    conn.prepare("SELECT booking_id FROM links WHERE document_id = ?1 AND undone_at IS NULL")?
        .query_map([document], |row| row.get(0))?
        .collect()
}

fn linked_documents(conn: &Connection, booking: i64) -> rusqlite::Result<Vec<i64>> {
    conn.prepare("SELECT document_id FROM links WHERE booking_id = ?1 AND undone_at IS NULL")?
        .query_map([booking], |row| row.get(0))?
        .collect()
}

fn assigned(conn: &Connection, column: &str, id: i64) -> rusqlite::Result<bool> {
    conn.query_row(
        &format!("SELECT EXISTS (SELECT 1 FROM tax_assignments WHERE {column} = ?1)"),
        [id],
        |row| row.get(0),
    )
}

/// Assigns, or changes the assignment. A payment is assigned once: on its document when it has one
/// assigned, never on both.
pub fn assign(
    conn: &Connection,
    on: On,
    assignment: &Assignment,
    now: Timestamp,
) -> Result<(), TaxError> {
    if category_of(&assignment.category).is_none() {
        return Err(TaxError::UnknownCategory);
    }
    if !(1..=100).contains(&assignment.share_percent) {
        return Err(TaxError::Share);
    }
    let (column, id) = match on {
        On::Document(document) => {
            for booking in linked_bookings(conn, document)? {
                if assigned(conn, "booking_id", booking)? {
                    return Err(TaxError::ThroughBooking);
                }
            }
            ("document_id", document)
        }
        On::Booking(booking) => {
            for document in linked_documents(conn, booking)? {
                if assigned(conn, "document_id", document)? {
                    return Err(TaxError::ThroughDocument);
                }
            }
            ("booking_id", booking)
        }
    };
    conn.execute(
        &format!(
            "INSERT INTO tax_assignments ({column}, category, share_percent, refunded_cents, labour_cents, made_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT ({column}) DO UPDATE SET category = excluded.category, share_percent = excluded.share_percent,
               refunded_cents = excluded.refunded_cents, labour_cents = excluded.labour_cents, made_at = excluded.made_at"
        ),
        params![
            id,
            assignment.category,
            assignment.share_percent,
            assignment.refunded_cents.max(0),
            assignment.labour_cents.map(|cents| cents.max(0)),
            now.to_string()
        ],
    )?;
    Ok(())
}

pub fn unassign(conn: &Connection, on: On) -> rusqlite::Result<()> {
    match on {
        On::Document(id) => {
            conn.execute("DELETE FROM tax_assignments WHERE document_id = ?1", [id])?
        }
        On::Booking(id) => {
            conn.execute("DELETE FROM tax_assignments WHERE booking_id = ?1", [id])?
        }
    };
    Ok(())
}

/// What the owner says of a year that no document shows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Profile {
    pub commute_km: i64,
    pub commute_days: i64,
    pub own_car: bool,
    /// A tax-free Jobticket or a flat-taxed Fahrtkostenzuschuss, which lowers the Entfernungspauschale.
    pub employer_commute_cents: i64,
    pub homeoffice_days: i64,
}

pub fn profile(conn: &Connection, year: i32) -> rusqlite::Result<Profile> {
    Ok(conn
        .query_row(
            "SELECT commute_km, commute_days, own_car, employer_commute_cents, homeoffice_days FROM tax_profiles WHERE year = ?1",
            [year],
            |row| {
                Ok(Profile {
                    commute_km: row.get(0)?,
                    commute_days: row.get(1)?,
                    own_car: row.get::<_, i64>(2)? != 0,
                    employer_commute_cents: row.get(3)?,
                    homeoffice_days: row.get(4)?,
                })
            },
        )
        .optional()?
        .unwrap_or(Profile {
            own_car: true,
            ..Profile::default()
        }))
}

pub fn set_profile(
    conn: &Connection,
    year: i32,
    profile: &Profile,
    now: Timestamp,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO tax_profiles (year, commute_km, commute_days, own_car, employer_commute_cents, homeoffice_days, changed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT (year) DO UPDATE SET commute_km = excluded.commute_km, commute_days = excluded.commute_days,
           own_car = excluded.own_car, employer_commute_cents = excluded.employer_commute_cents,
           homeoffice_days = excluded.homeoffice_days, changed_at = excluded.changed_at",
        params![
            year,
            profile.commute_km.max(0),
            profile.commute_days.clamp(0, 366),
            i64::from(profile.own_car),
            profile.employer_commute_cents.max(0),
            profile.homeoffice_days.clamp(0, 366),
            now.to_string()
        ],
    )?;
    Ok(())
}

/// The Entfernungspauschale: one way, per day, by the year's rate, capped without an own car, less
/// what the employer paid towards the way, never below zero.
pub fn entfernungspauschale(profile: &Profile, values: &YearValues) -> i64 {
    let first = profile.commute_km.min(20) * values.entfernung_first_20;
    let rest = (profile.commute_km - 20).max(0) * values.entfernung_from_21;
    let full = (first + rest) * profile.commute_days;
    let capped = if profile.own_car {
        full
    } else {
        full.min(values.entfernung_max_ohne_auto)
    };
    (capped - profile.employer_commute_cents).max(0)
}

pub fn homeoffice(profile: &Profile, values: &YearValues) -> i64 {
    (profile.homeoffice_days * values.homeoffice_per_day).min(values.homeoffice_max)
}

/// A hint at an entry, which never blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Note {
    /// No booking paid it: the year is that of the invoice date.
    YearFromInvoice,
    /// § 35a without a labour share: nothing counts.
    NoLabour,
    /// A donation above the simplified proof needs a Zuwendungsbestätigung.
    NeedsReceipt,
}

impl Note {
    pub fn text(self) -> &'static str {
        match self {
            Note::YearFromInvoice => "Jahr nach Rechnungsdatum, keine Zahlung gefunden",
            Note::NoLabour => "Lohnanteil fehlt",
            Note::NeedsReceipt => "Zuwendungsbestätigung nötig",
        }
    }
}

/// One entry of a year: what is assigned, its amount and what counts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub on: On,
    pub category: &'static Category,
    pub day: Date,
    /// What was paid, positive.
    pub gross: i64,
    pub counted: i64,
    pub notes: Vec<Note>,
}

/// A section of a year: the entries of one category and their sum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub category: &'static Category,
    pub entries: Vec<Entry>,
    pub counted: i64,
}

/// What a tax year collects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Year {
    pub year: i32,
    pub values: Option<YearValues>,
    pub profile: Profile,
    pub sections: Vec<Section>,
    pub entfernungspauschale: Option<i64>,
    pub homeoffice: Option<i64>,
    /// The documents and the lump sums of Werbungskosten together.
    pub werbungskosten: i64,
    /// What they are above the Arbeitnehmer-Pauschbetrag, zero when below.
    pub above_pauschbetrag: Option<i64>,
    /// Erstausbildung capped at its maximum.
    pub erstausbildung: i64,
    /// The § 35a reductions, by the year's rate, capped.
    pub haushalt_reduction: Option<i64>,
    pub handwerker_reduction: Option<i64>,
}

fn day_of(text: &str) -> Option<Date> {
    text.get(..10)?.parse().ok()
}

/// A row of `tax_assignments`: document, booking, category, share, refunded, labour.
type Stored = (Option<i64>, Option<i64>, String, i64, i64, Option<i64>);

fn entries(conn: &Connection, values: Option<&YearValues>) -> rusqlite::Result<Vec<Entry>> {
    let rows: Vec<Stored> = conn
        .prepare("SELECT document_id, booking_id, category, share_percent, refunded_cents, labour_cents FROM tax_assignments")?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let mut entries = Vec::new();
    for (document, booking, key, share_percent, refunded_cents, labour_cents) in rows {
        let Some(category) = category_of(&key) else {
            continue;
        };
        let assignment = Assignment {
            category: key,
            share_percent,
            refunded_cents,
            labour_cents,
        };
        let mut notes = Vec::new();
        let (on, day, gross) = match (document, booking) {
            (Some(document), _) => {
                let chosen = crate::checks::chosen(&crate::checks::values(conn, document)?);
                let gross = chosen
                    .get("gross")
                    .and_then(|seen| seen.value.parse::<i64>().ok());
                // The year paid: the first booking that paid it, else the invoice date with a note.
                let paid: Option<String> = conn
                    .query_row(
                        "SELECT min(b.day) FROM links l JOIN bookings b ON b.id = l.booking_id WHERE l.document_id = ?1 AND l.undone_at IS NULL",
                        [document],
                        |row| row.get(0),
                    )
                    .optional()?
                    .flatten();
                let day = match paid.as_deref().and_then(day_of) {
                    Some(day) => Some(day),
                    None => {
                        notes.push(Note::YearFromInvoice);
                        chosen
                            .get("invoice_date")
                            .and_then(|seen| day_of(&seen.value))
                    }
                };
                let (Some(day), Some(gross)) = (day, gross) else {
                    continue;
                };
                (On::Document(document), day, gross.abs())
            }
            (None, Some(booking)) => {
                let (day, cents): (String, i64) = conn.query_row(
                    "SELECT day, amount_cents FROM bookings WHERE id = ?1",
                    [booking],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )?;
                let Some(day) = day_of(&day) else {
                    continue;
                };
                (On::Booking(booking), day, cents.abs())
            }
            (None, None) => continue,
        };
        if category.group == Group::Handwerker && assignment.labour_cents.is_none() {
            notes.push(Note::NoLabour);
        }
        if category.key == "spende"
            && values.is_some_and(|values| gross > values.spende_vereinfacht)
        {
            notes.push(Note::NeedsReceipt);
        }
        entries.push(Entry {
            on,
            category,
            day,
            gross,
            counted: counted(gross, &assignment),
            notes,
        });
    }
    Ok(entries)
}

/// The years that have an entry, newest first.
pub fn years(conn: &Connection) -> rusqlite::Result<Vec<i32>> {
    let mut years: Vec<i32> = entries(conn, None)?
        .iter()
        .map(|entry| i32::from(entry.day.year()))
        .collect();
    years.sort_unstable_by(|a, b| b.cmp(a));
    years.dedup();
    Ok(years)
}

/// What a tax year collects, by the day paid.
pub fn year(conn: &Connection, year: i32) -> rusqlite::Result<Year> {
    let values = year_values(year);
    let profile = profile(conn, year)?;
    let mut by_category: BTreeMap<&'static str, Vec<Entry>> = BTreeMap::new();
    for entry in entries(conn, values.as_ref())? {
        if i32::from(entry.day.year()) == year {
            by_category
                .entry(entry.category.key)
                .or_default()
                .push(entry);
        }
    }
    let sections: Vec<Section> = CATEGORIES
        .iter()
        .filter_map(|category| {
            let mut entries = by_category.remove(category.key)?;
            entries.sort_by_key(|entry| entry.day);
            let counted = entries.iter().map(|entry| entry.counted).sum();
            Some(Section {
                category,
                entries,
                counted,
            })
        })
        .collect();
    let sum = |group: Group| -> i64 {
        sections
            .iter()
            .filter(|section| section.category.group == group)
            .map(|section| section.counted)
            .sum()
    };
    let entfernungspauschale = values
        .as_ref()
        .map(|values| entfernungspauschale(&profile, values));
    let homeoffice = values.as_ref().map(|values| homeoffice(&profile, values));
    let werbungskosten =
        sum(Group::Werbungskosten) + entfernungspauschale.unwrap_or(0) + homeoffice.unwrap_or(0);
    let erstausbildung_sum: i64 = sections
        .iter()
        .filter(|section| section.category.key == "erstausbildung")
        .map(|section| section.counted)
        .sum();
    let reduction = |cents: i64, percent: i64, max: i64| ((cents * percent + 50) / 100).min(max);
    Ok(Year {
        year,
        values,
        profile,
        entfernungspauschale,
        homeoffice,
        werbungskosten,
        above_pauschbetrag: values
            .as_ref()
            .map(|values| (werbungskosten - values.arbeitnehmer_pauschbetrag).max(0)),
        erstausbildung: values.as_ref().map_or(erstausbildung_sum, |values| {
            erstausbildung_sum.min(values.erstausbildung_max)
        }),
        haushalt_reduction: values.as_ref().map(|values| {
            reduction(
                sum(Group::Haushalt),
                values.haushalt_percent,
                values.haushalt_max,
            )
        }),
        handwerker_reduction: values.as_ref().map(|values| {
            reduction(
                sum(Group::Handwerker),
                values.handwerker_percent,
                values.handwerker_max,
            )
        }),
        sections,
    })
}

#[cfg(test)]
mod tests;
