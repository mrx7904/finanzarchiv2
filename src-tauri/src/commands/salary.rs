//! The command of the salary (#35): the years of the payslips, each month with its amounts or
//! "fehlt", and the Netto against what the booking brought. Cents as `i32` and as German text.

use std::sync::{Arc, Mutex};

use serde::Serialize;
use specta::Type;
use tauri::State;

use super::CommandError;
use crate::archive::Opened;
use crate::money::Cents;
use crate::salary::{self, FIELDS, Month};

/// An amount with its field and its German name.
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SalaryAmount {
    pub field: String,
    pub label: String,
    pub cents: i32,
    pub text: String,
}

/// A month: its amounts, or `missing` ("fehlt") with none.
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SalaryMonthRow {
    /// `YYYY-MM`.
    pub month: String,
    pub missing: bool,
    pub payslips: Vec<u32>,
    pub amounts: Vec<SalaryAmount>,
    pub paid_text: Option<String>,
    /// What the bookings brought minus the Netto, "-50,00 €", when they differ.
    pub difference_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SalaryYearRow {
    pub year: i32,
    pub months: Vec<SalaryMonthRow>,
    pub totals: Vec<SalaryAmount>,
    pub missing: u32,
}

fn amounts(
    by_field: &std::collections::BTreeMap<&'static str, i64>,
) -> Result<Vec<SalaryAmount>, CommandError> {
    FIELDS
        .iter()
        .filter_map(|(field, label)| by_field.get(field).map(|cents| (*field, *label, *cents)))
        .map(|(field, label, cents)| {
            Ok(SalaryAmount {
                field: field.to_owned(),
                label: label.to_owned(),
                cents: i32::try_from(cents).map_err(|_| CommandError::internal())?,
                text: Cents::new(cents).to_string(),
            })
        })
        .collect()
}

fn month_row(month: &Month) -> Result<SalaryMonthRow, CommandError> {
    Ok(SalaryMonthRow {
        month: format!("{:04}-{:02}", month.year, month.month),
        missing: month.missing(),
        payslips: month
            .payslips
            .iter()
            .map(|id| u32::try_from(*id).map_err(|_| CommandError::internal()))
            .collect::<Result<_, _>>()?,
        amounts: amounts(&month.amounts)?,
        paid_text: month.paid.map(|cents| Cents::new(cents).to_string()),
        difference_text: month
            .difference()
            .map(|cents| Cents::new(cents).to_string()),
    })
}

/// The years of the payslips, newest first.
#[tauri::command]
#[specta::specta]
pub fn salary_years(
    archive: State<'_, Arc<Mutex<Opened>>>,
) -> Result<Vec<SalaryYearRow>, CommandError> {
    let opened = archive.lock().map_err(|_| CommandError::internal())?;
    let Opened::Ready(ready) = &*opened else {
        return Err(CommandError::archive_not_ready());
    };
    salary::years(ready.connection())
        .map_err(|_| CommandError::internal())?
        .iter()
        .map(|year| {
            Ok(SalaryYearRow {
                year: year.year,
                months: year
                    .months
                    .iter()
                    .map(month_row)
                    .collect::<Result<_, _>>()?,
                totals: amounts(&year.totals)?,
                missing: u32::try_from(year.missing()).map_err(|_| CommandError::internal())?,
            })
        })
        .collect()
}
