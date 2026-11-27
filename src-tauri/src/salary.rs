//! Salary from the payslips (#35, FA-39): Brutto, Netto and the deductions of every month and year
//! from the filed payslips, each value the one that counts (`checks::chosen`). A month within the
//! range of the payslips without one is missing, never zero, and the Netto stands against the
//! booking that paid it (#24) (knowledge/decision-salary.md).

use std::collections::BTreeMap;

use jiff::civil::Date;
use rusqlite::{Connection, params};

use crate::checks;

/// The amounts of a payslip, in the order the window shows them, with their German names.
pub const FIELDS: [(&str, &str); 9] = [
    ("brutto", "Brutto"),
    ("netto", "Netto"),
    ("lohnsteuer", "Lohnsteuer"),
    ("solidaritaetszuschlag", "Solidaritätszuschlag"),
    ("kirchensteuer", "Kirchensteuer"),
    ("krankenversicherung", "Krankenversicherung"),
    ("rentenversicherung", "Rentenversicherung"),
    ("arbeitslosenversicherung", "Arbeitslosenversicherung"),
    ("pflegeversicherung", "Pflegeversicherung"),
];

/// One month of the range of the payslips.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Month {
    pub year: i32,
    /// 1 is January.
    pub month: i8,
    /// The payslips of the month, two when there were two employers.
    pub payslips: Vec<i64>,
    /// The amounts by field, summed over the payslips; a field no payslip has is absent.
    pub amounts: BTreeMap<&'static str, i64>,
    /// What the bookings linked to the payslips brought.
    pub paid: Option<i64>,
}

impl Month {
    /// No payslip: "fehlt".
    pub fn missing(&self) -> bool {
        self.payslips.is_empty()
    }

    /// What the bookings brought minus the Netto, when both are known and they differ.
    pub fn difference(&self) -> Option<i64> {
        let netto = *self.amounts.get("netto")?;
        let paid = self.paid?;
        (paid != netto).then_some(paid - netto)
    }
}

/// A year of the range: its months from the first payslip's to the last's, and the sums.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Year {
    pub year: i32,
    pub months: Vec<Month>,
    pub totals: BTreeMap<&'static str, i64>,
}

impl Year {
    pub fn missing(&self) -> usize {
        self.months.iter().filter(|month| month.missing()).count()
    }
}

/// The month of a payslip, `YYYY-MM` as the reader stores it.
fn month_of(period: &str) -> Option<(i32, i8)> {
    let day: Date = format!("{period}-01").parse().ok()?;
    Some((day.year().into(), day.month()))
}

/// Every month from the first payslip's to the last's, the missing ones included.
pub fn months(conn: &Connection) -> rusqlite::Result<Vec<Month>> {
    let payslips: Vec<i64> = conn
        .prepare("SELECT id FROM documents WHERE state = 'filed' AND kind = 'payslip' ORDER BY id")?
        .query_map([], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let mut by_month: BTreeMap<(i32, i8), Month> = BTreeMap::new();
    for payslip in payslips {
        let chosen = checks::chosen(&checks::values(conn, payslip)?);
        let Some((year, month)) = chosen.get("period").and_then(|seen| month_of(&seen.value))
        else {
            continue;
        };
        let entry = by_month.entry((year, month)).or_insert_with(|| Month {
            year,
            month,
            ..Month::default()
        });
        entry.payslips.push(payslip);
        for (field, _) in FIELDS {
            if let Some(cents) = chosen
                .get(field)
                .and_then(|seen| seen.value.parse::<i64>().ok())
            {
                *entry.amounts.entry(field).or_default() += cents;
            }
        }
        let paid: Option<i64> = conn.query_row(
            "SELECT sum(cents) FROM links WHERE document_id = ?1 AND undone_at IS NULL",
            params![payslip],
            |row| row.get(0),
        )?;
        if let Some(paid) = paid {
            *entry.paid.get_or_insert(0) += paid;
        }
    }
    let (Some(&first), Some(&last)) = (by_month.keys().next(), by_month.keys().next_back()) else {
        return Ok(Vec::new());
    };
    let mut all = Vec::new();
    let (mut year, mut month) = first;
    while (year, month) <= last {
        all.push(by_month.remove(&(year, month)).unwrap_or(Month {
            year,
            month,
            ..Month::default()
        }));
        (year, month) = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };
    }
    Ok(all)
}

/// The years of the range, newest first, each with its months and sums.
pub fn years(conn: &Connection) -> rusqlite::Result<Vec<Year>> {
    let mut years: BTreeMap<i32, Vec<Month>> = BTreeMap::new();
    for month in months(conn)? {
        years.entry(month.year).or_default().push(month);
    }
    Ok(years
        .into_iter()
        .rev()
        .map(|(year, months)| {
            let mut totals: BTreeMap<&'static str, i64> = BTreeMap::new();
            for month in &months {
                for (field, cents) in &month.amounts {
                    *totals.entry(field).or_default() += cents;
                }
            }
            Year {
                year,
                months,
                totals,
            }
        })
        .collect())
}

#[cfg(test)]
mod tests;
