//! The commands of the evaluations (#33): what a year brought, by month, category and party.
//! Amounts cross as whole cents (`i32`, a year of a household stays far below its limit) for the
//! charts and as German text for reading, never as a float.

use std::sync::{Arc, Mutex};

use serde::Serialize;
use specta::Type;
use tauri::State;

use super::CommandError;
use crate::archive::Opened;
use crate::evaluations::{self, Part, Side, Sum};

/// A sum as the window takes it.
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SumRow {
    pub cents: i32,
    pub text: String,
    pub count: u32,
}

fn sum_row(sum: Sum) -> Result<SumRow, CommandError> {
    Ok(SumRow {
        cents: i32::try_from(sum.cents).map_err(|_| CommandError::internal())?,
        text: evaluations::text(sum.cents),
        count: sum.count,
    })
}

/// A category or a party with its sum. `id` null is "Ohne Kategorie" or "Ohne Partei".
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PartRow {
    pub id: Option<u32>,
    pub name: Option<String>,
    pub income: bool,
    pub sum: SumRow,
}

fn part_row(part: Part) -> Result<PartRow, CommandError> {
    Ok(PartRow {
        id: part
            .id
            .map(|id| u32::try_from(id).map_err(|_| CommandError::internal()))
            .transpose()?,
        name: part.name,
        income: part.side == Side::Income,
        sum: sum_row(part.sum)?,
    })
}

/// What a year brought, and the year before for comparison.
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct YearRow {
    pub year: i32,
    pub income: Vec<SumRow>,
    pub spending: Vec<SumRow>,
    pub income_total: SumRow,
    pub spending_total: SumRow,
    /// Income minus spending.
    pub balance: SumRow,
    pub categories: Vec<PartRow>,
    pub parties: Vec<PartRow>,
    pub previous_income: SumRow,
    pub previous_spending: SumRow,
}

#[tauri::command]
#[specta::specta]
pub fn evaluation_years(archive: State<'_, Arc<Mutex<Opened>>>) -> Result<Vec<i32>, CommandError> {
    let opened = archive.lock().map_err(|_| CommandError::internal())?;
    let Opened::Ready(ready) = &*opened else {
        return Err(CommandError::archive_not_ready());
    };
    evaluations::years(ready.connection()).map_err(|_| CommandError::internal())
}

pub fn year_row(conn: &rusqlite::Connection, year: i32) -> Result<YearRow, CommandError> {
    let this = evaluations::year(conn, year).map_err(|_| CommandError::internal())?;
    let before = evaluations::year(conn, year - 1).map_err(|_| CommandError::internal())?;
    let income_total = this.total(Side::Income);
    let spending_total = this.total(Side::Spending);
    Ok(YearRow {
        year,
        income: this
            .income
            .iter()
            .map(|sum| sum_row(*sum))
            .collect::<Result<_, _>>()?,
        spending: this
            .spending
            .iter()
            .map(|sum| sum_row(*sum))
            .collect::<Result<_, _>>()?,
        income_total: sum_row(income_total)?,
        spending_total: sum_row(spending_total)?,
        balance: sum_row(Sum {
            cents: income_total.cents - spending_total.cents,
            count: income_total.count + spending_total.count,
        })?,
        categories: this
            .categories
            .into_iter()
            .map(part_row)
            .collect::<Result<_, _>>()?,
        parties: this
            .parties
            .into_iter()
            .map(part_row)
            .collect::<Result<_, _>>()?,
        previous_income: sum_row(before.total(Side::Income))?,
        previous_spending: sum_row(before.total(Side::Spending))?,
    })
}

#[tauri::command]
#[specta::specta]
pub fn evaluation_year(
    archive: State<'_, Arc<Mutex<Opened>>>,
    year: i32,
) -> Result<YearRow, CommandError> {
    let opened = archive.lock().map_err(|_| CommandError::internal())?;
    let Opened::Ready(ready) = &*opened else {
        return Err(CommandError::archive_not_ready());
    };
    year_row(ready.connection(), year)
}
