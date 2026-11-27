//! The commands of the taxes (#41): the categories, assigning a document or a booking, the tax
//! profile of a year, and what a year collects by form and section. Cents as `i32` and as German
//! text. Every result carries the notice that the owner checks it himself.

use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use super::CommandError;
use crate::archive::Opened;
use crate::money::Cents;
use crate::taxes::{self, Assignment, On, Profile, TaxError};

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TaxCategoryRow {
    pub key: String,
    pub label: String,
    pub group: String,
    /// Form and section as ELSTER shows them, never a line number.
    pub elster: String,
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TaxAmount {
    pub cents: i32,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TaxEntryRow {
    pub document: Option<u32>,
    pub booking: Option<u32>,
    pub day: String,
    pub gross: TaxAmount,
    pub counted: TaxAmount,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TaxSectionRow {
    pub category: TaxCategoryRow,
    pub entries: Vec<TaxEntryRow>,
    pub counted: TaxAmount,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TaxProfileRow {
    pub commute_km: u32,
    pub commute_days: u32,
    pub own_car: bool,
    pub employer_commute_cents: u32,
    pub homeoffice_days: u32,
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TaxYearRow {
    pub year: i32,
    pub notice: String,
    /// Where the values of the year come from; `None` when the app has none for it.
    pub values_source: Option<String>,
    pub profile: TaxProfileRow,
    pub sections: Vec<TaxSectionRow>,
    pub entfernungspauschale: Option<TaxAmount>,
    pub homeoffice: Option<TaxAmount>,
    pub werbungskosten: TaxAmount,
    pub pauschbetrag: Option<TaxAmount>,
    pub above_pauschbetrag: Option<TaxAmount>,
    pub erstausbildung: TaxAmount,
    pub haushalt_reduction: Option<TaxAmount>,
    pub handwerker_reduction: Option<TaxAmount>,
}

#[derive(Debug, Clone, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TaxAssignmentInput {
    pub document: Option<u32>,
    pub booking: Option<u32>,
    pub category: String,
    pub share_percent: u32,
    pub refunded_cents: u32,
    pub labour_cents: Option<u32>,
}

fn amount(cents: i64) -> Result<TaxAmount, CommandError> {
    Ok(TaxAmount {
        cents: i32::try_from(cents).map_err(|_| CommandError::internal())?,
        text: Cents::new(cents).to_string(),
    })
}

fn id32(id: i64) -> Result<u32, CommandError> {
    u32::try_from(id).map_err(|_| CommandError::internal())
}

fn category_row(category: &taxes::Category) -> TaxCategoryRow {
    TaxCategoryRow {
        key: category.key.to_owned(),
        label: category.label.to_owned(),
        group: category.group.label().to_owned(),
        elster: category.elster.to_owned(),
    }
}

fn on(document: Option<u32>, booking: Option<u32>) -> Result<On, CommandError> {
    match (document, booking) {
        (Some(document), None) => Ok(On::Document(i64::from(document))),
        (None, Some(booking)) => Ok(On::Booking(i64::from(booking))),
        _ => Err(CommandError::not_found()),
    }
}

fn tax_error(error: TaxError) -> CommandError {
    match error {
        TaxError::Database(_) => CommandError::internal(),
        other => CommandError::refused(other.to_string()),
    }
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

#[tauri::command]
#[specta::specta]
pub fn tax_categories() -> Vec<TaxCategoryRow> {
    taxes::CATEGORIES.iter().map(category_row).collect()
}

/// The years with an entry, newest first.
#[tauri::command]
#[specta::specta]
pub fn tax_years(archive: State<'_, Arc<Mutex<Opened>>>) -> Result<Vec<i32>, CommandError> {
    with_ready(&archive, |conn| {
        taxes::years(conn).map_err(|_| CommandError::internal())
    })
}

pub fn year_row(conn: &rusqlite::Connection, year: i32) -> Result<TaxYearRow, CommandError> {
    let found = taxes::year(conn, year).map_err(|_| CommandError::internal())?;
    let sections = found
        .sections
        .iter()
        .map(|section| {
            Ok(TaxSectionRow {
                category: category_row(section.category),
                entries: section
                    .entries
                    .iter()
                    .map(|entry| {
                        let (document, booking) = match entry.on {
                            On::Document(id) => (Some(id32(id)?), None),
                            On::Booking(id) => (None, Some(id32(id)?)),
                        };
                        Ok(TaxEntryRow {
                            document,
                            booking,
                            day: entry.day.to_string(),
                            gross: amount(entry.gross)?,
                            counted: amount(entry.counted)?,
                            notes: entry
                                .notes
                                .iter()
                                .map(|note| note.text().to_owned())
                                .collect(),
                        })
                    })
                    .collect::<Result<_, CommandError>>()?,
                counted: amount(section.counted)?,
            })
        })
        .collect::<Result<_, CommandError>>()?;
    let optional = |cents: Option<i64>| cents.map(amount).transpose();
    let profile = found.profile;
    let small = |value: i64| u32::try_from(value).unwrap_or(0);
    Ok(TaxYearRow {
        year,
        notice: taxes::CHECK_NOTICE.to_owned(),
        values_source: found.values.map(|values| values.source.to_owned()),
        profile: TaxProfileRow {
            commute_km: small(profile.commute_km),
            commute_days: small(profile.commute_days),
            own_car: profile.own_car,
            employer_commute_cents: small(profile.employer_commute_cents),
            homeoffice_days: small(profile.homeoffice_days),
        },
        sections,
        entfernungspauschale: optional(found.entfernungspauschale)?,
        homeoffice: optional(found.homeoffice)?,
        werbungskosten: amount(found.werbungskosten)?,
        pauschbetrag: optional(found.values.map(|values| values.arbeitnehmer_pauschbetrag))?,
        above_pauschbetrag: optional(found.above_pauschbetrag)?,
        erstausbildung: amount(found.erstausbildung)?,
        haushalt_reduction: optional(found.haushalt_reduction)?,
        handwerker_reduction: optional(found.handwerker_reduction)?,
    })
}

/// What a tax year collects.
#[tauri::command]
#[specta::specta]
pub fn tax_year(
    archive: State<'_, Arc<Mutex<Opened>>>,
    year: i32,
) -> Result<TaxYearRow, CommandError> {
    with_ready(&archive, |conn| year_row(conn, year))
}

/// Assigns a document or a booking, or changes its assignment.
#[tauri::command]
#[specta::specta]
pub fn assign_tax(
    archive: State<'_, Arc<Mutex<Opened>>>,
    input: TaxAssignmentInput,
) -> Result<(), CommandError> {
    let target = on(input.document, input.booking)?;
    let assignment = Assignment {
        category: input.category,
        share_percent: i64::from(input.share_percent),
        refunded_cents: i64::from(input.refunded_cents),
        labour_cents: input.labour_cents.map(i64::from),
    };
    with_ready(&archive, |conn| {
        taxes::assign(conn, target, &assignment, jiff::Timestamp::now()).map_err(tax_error)
    })
}

#[tauri::command]
#[specta::specta]
pub fn unassign_tax(
    archive: State<'_, Arc<Mutex<Opened>>>,
    document: Option<u32>,
    booking: Option<u32>,
) -> Result<(), CommandError> {
    let target = on(document, booking)?;
    with_ready(&archive, |conn| {
        taxes::unassign(conn, target).map_err(|_| CommandError::internal())
    })
}

#[tauri::command]
#[specta::specta]
pub fn set_tax_profile(
    archive: State<'_, Arc<Mutex<Opened>>>,
    year: i32,
    profile: TaxProfileRow,
) -> Result<(), CommandError> {
    let profile = Profile {
        commute_km: i64::from(profile.commute_km),
        commute_days: i64::from(profile.commute_days),
        own_car: profile.own_car,
        employer_commute_cents: i64::from(profile.employer_commute_cents),
        homeoffice_days: i64::from(profile.homeoffice_days),
    };
    with_ready(&archive, |conn| {
        taxes::set_profile(conn, year, &profile, jiff::Timestamp::now())
            .map_err(|_| CommandError::internal())
    })
}
