//! The command of the balances over time (#36): each account and the sum of all at the end of each
//! month, the bank's balance where it printed one, and a difference named as in "Konten".

use std::sync::{Arc, Mutex};

use serde::Serialize;
use specta::Type;
use tauri::State;

use super::CommandError;
use crate::archive::Opened;
use crate::balances::{self, Day};
use crate::money::Cents;

/// The balance at the end of a month (or of the last day known).
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BalancePoint {
    /// `YYYY-MM-DD`.
    pub day: String,
    pub cents: i32,
    pub text: String,
    /// The bank printed this balance.
    pub by_bank: bool,
    /// The bank's minus the one counted, when they differ.
    pub difference_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AccountBalances {
    pub account: u32,
    pub name: String,
    pub points: Vec<BalancePoint>,
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BalancesRow {
    pub accounts: Vec<AccountBalances>,
    pub total: Vec<BalancePoint>,
}

fn cents32(cents: Cents) -> Result<i32, CommandError> {
    i32::try_from(cents.value()).map_err(|_| CommandError::internal())
}

fn point(day: Day) -> Result<BalancePoint, CommandError> {
    Ok(BalancePoint {
        day: day.day.to_string(),
        cents: cents32(day.shown())?,
        text: day.shown().to_string(),
        by_bank: day.bank.is_some(),
        difference_text: day.difference().map(|cents| cents.to_string()),
    })
}

pub fn balances_row(conn: &rusqlite::Connection) -> Result<BalancesRow, CommandError> {
    let (accounts, total) = balances::all(conn).map_err(|_| CommandError::internal())?;
    let accounts = accounts
        .into_iter()
        .map(|account| {
            let (Some(first), Some(last)) = (account.days.first(), account.days.last()) else {
                return Ok(None);
            };
            let points = balances::month_ends(first.day, last.day)
                .into_iter()
                .filter_map(|end| balances::on(&account.days, end))
                .map(point)
                .collect::<Result<_, _>>()?;
            Ok(Some(AccountBalances {
                account: u32::try_from(account.id).map_err(|_| CommandError::internal())?,
                name: account.name,
                points,
            }))
        })
        .collect::<Result<Vec<_>, CommandError>>()?
        .into_iter()
        .flatten()
        .collect();
    let total = match (total.first(), total.last()) {
        (Some(first), Some(last)) => balances::month_ends(first.0, last.0)
            .into_iter()
            .filter_map(|end| total.iter().rev().find(|(day, _)| *day <= end).copied())
            .map(|(day, cents)| {
                Ok(BalancePoint {
                    day: day.to_string(),
                    cents: cents32(cents)?,
                    text: cents.to_string(),
                    by_bank: false,
                    difference_text: None,
                })
            })
            .collect::<Result<_, CommandError>>()?,
        _ => Vec::new(),
    };
    Ok(BalancesRow { accounts, total })
}

/// Each account and the sum of all at the end of each month.
#[tauri::command]
#[specta::specta]
pub fn balances_by_month(
    archive: State<'_, Arc<Mutex<Opened>>>,
) -> Result<BalancesRow, CommandError> {
    let opened = archive.lock().map_err(|_| CommandError::internal())?;
    let Opened::Ready(ready) = &*opened else {
        return Err(CommandError::archive_not_ready());
    };
    balances_row(ready.connection())
}
