//! Income and spending from the bookings (#33, FA-37), the only source of money. A booking counts
//! once: one replaced by its CSV twin does not, a move between own accounts does not. A booking
//! counts by its direction, except that money in with a category of spending is a refund and lowers
//! that category, and money out with a category of income lowers that income, in the year the
//! money moved (knowledge/decision-evaluations.md).

use std::collections::BTreeMap;

use rusqlite::{Connection, params};

use crate::money::Cents;

/// The bookings that count, with their effective category: their own, else their party's.
const COUNTED: &str = "
    SELECT b.id, b.day, b.amount_cents, coalesce(b.category_id, p.category_id) AS category, c.direction, c.name, p.id, p.name
    FROM bookings b
    LEFT JOIN parties p ON p.id = b.party_id
    LEFT JOIN categories c ON c.id = coalesce(b.category_id, p.category_id)
    WHERE b.replaced_by IS NULL
      AND NOT EXISTS (SELECT 1 FROM booking_pairs m WHERE (m.out_booking = b.id OR m.in_booking = b.id) AND m.undone_at IS NULL)
      AND substr(b.day, 1, 4) = ?1";

/// Whether a booking is income or spending, and by how much it moves that side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Income,
    Spending,
}

/// The side a booking counts on and the amount it adds there (negative for a refund).
pub fn side_of(cents: i64, direction: Option<&str>) -> (Side, i64) {
    match (cents >= 0, direction) {
        // Money in with a category of spending: a refund, it lowers the spending.
        (true, Some("out")) => (Side::Spending, -cents),
        // Money out with a category of income: it lowers the income.
        (false, Some("in")) => (Side::Income, cents),
        (true, _) => (Side::Income, cents),
        (false, _) => (Side::Spending, -cents),
    }
}

/// A sum with its count of bookings.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Sum {
    pub cents: i64,
    pub count: u32,
}

impl Sum {
    fn add(&mut self, cents: i64) {
        self.cents += cents;
        self.count += 1;
    }
}

/// A category or a party with its sum on one side. `None` is "Ohne Kategorie" or "Ohne Partei".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub side: Side,
    pub sum: Sum,
}

/// What one year brought.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Year {
    pub year: i32,
    /// Index 0 is January.
    pub income: [Sum; 12],
    pub spending: [Sum; 12],
    pub categories: Vec<Part>,
    pub parties: Vec<Part>,
}

impl Year {
    pub fn total(&self, side: Side) -> Sum {
        let months = match side {
            Side::Income => &self.income,
            Side::Spending => &self.spending,
        };
        months.iter().fold(Sum::default(), |total, month| Sum {
            cents: total.cents + month.cents,
            count: total.count + month.count,
        })
    }
}

type Key = (Option<i64>, Option<String>, bool);

fn sorted(map: BTreeMap<Key, Sum>) -> Vec<Part> {
    let mut parts: Vec<Part> = map
        .into_iter()
        .map(|((id, name, income), sum)| Part {
            id,
            name,
            side: if income { Side::Income } else { Side::Spending },
            sum,
        })
        .collect();
    // The larger first, the one without a name last.
    parts.sort_by(|a, b| {
        a.id.is_none()
            .cmp(&b.id.is_none())
            .then(b.sum.cents.cmp(&a.sum.cents))
            .then(a.name.cmp(&b.name))
    });
    parts
}

/// What a year brought, by month, category and party.
pub fn year(conn: &Connection, year: i32) -> rusqlite::Result<Year> {
    let mut income = [Sum::default(); 12];
    let mut spending = [Sum::default(); 12];
    let mut categories: BTreeMap<Key, Sum> = BTreeMap::new();
    let mut parties: BTreeMap<Key, Sum> = BTreeMap::new();
    let mut statement = conn.prepare(COUNTED)?;
    let rows = statement.query_map(params![format!("{year:04}")], |row| {
        Ok((
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, Option<i64>>(3)?,
            row.get::<_, Option<String>>(4)?,
            row.get::<_, Option<String>>(5)?,
            row.get::<_, Option<i64>>(6)?,
            row.get::<_, Option<String>>(7)?,
        ))
    })?;
    for row in rows {
        let (day, cents, category, direction, category_name, party, party_name) = row?;
        let month: usize = day
            .get(5..7)
            .and_then(|month| month.parse().ok())
            .unwrap_or(1);
        let index = month.clamp(1, 12) - 1;
        let (side, amount) = side_of(cents, direction.as_deref());
        let income_side = side == Side::Income;
        match side {
            Side::Income => income[index].add(amount),
            Side::Spending => spending[index].add(amount),
        }
        categories
            .entry((category, category_name, income_side))
            .or_default()
            .add(amount);
        parties
            .entry((party, party_name, income_side))
            .or_default()
            .add(amount);
    }
    Ok(Year {
        year,
        income,
        spending,
        categories: sorted(categories),
        parties: sorted(parties),
    })
}

/// The years that have bookings, newest first.
pub fn years(conn: &Connection) -> rusqlite::Result<Vec<i32>> {
    conn.prepare("SELECT DISTINCT CAST(substr(day, 1, 4) AS INTEGER) FROM bookings WHERE replaced_by IS NULL ORDER BY 1 DESC")?
        .query_map([], |row| row.get(0))?
        .collect()
}

/// A sum as the window shows it.
pub fn text(cents: i64) -> String {
    Cents::new(cents).to_string()
}

#[cfg(test)]
mod tests;
