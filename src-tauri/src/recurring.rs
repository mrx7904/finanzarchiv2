//! Recurring payments (#34, FA-38): subscriptions, contracts, insurances, rent and salary, found
//! from the bookings that count (#33) by their party, their rhythm and a near amount, at least
//! three times. Each comes with its last and next day, its amount in the year, a changed amount
//! and whether it ended. The owner can say one is none, and that stays
//! (knowledge/decision-recurring.md).

use std::collections::BTreeMap;

use jiff::civil::Date;
use jiff::{Span, Timestamp};
use rusqlite::{Connection, params};

use crate::money::Cents;

/// The bookings of a party that count: not replaced by a CSV twin, no move between own accounts.
const BOOKINGS: &str = "
    SELECT b.party_id, p.name, b.day, b.amount_cents
    FROM bookings b
    JOIN parties p ON p.id = b.party_id
    WHERE b.replaced_by IS NULL
      AND NOT EXISTS (SELECT 1 FROM booking_pairs m WHERE (m.out_booking = b.id OR m.in_booking = b.id) AND m.undone_at IS NULL)
    ORDER BY b.party_id, b.day, b.id";

/// The days and amounts of one party in one direction.
type Payments = Vec<(Date, i64)>;

/// How often a payment comes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rhythm {
    Monthly,
    Quarterly,
    Yearly,
}

impl Rhythm {
    pub const ALL: [Rhythm; 3] = [Rhythm::Monthly, Rhythm::Quarterly, Rhythm::Yearly];

    pub fn months(self) -> i64 {
        match self {
            Rhythm::Monthly => 1,
            Rhythm::Quarterly => 3,
            Rhythm::Yearly => 12,
        }
    }

    /// The days between two payments that keep the rhythm, a few days late or early.
    fn fits(self, days: i32) -> bool {
        match self {
            Rhythm::Monthly => (25..=36).contains(&days),
            Rhythm::Quarterly => (80..=100).contains(&days),
            Rhythm::Yearly => (350..=380).contains(&days),
        }
    }

    /// The days of two rhythms, when one payment was missed.
    fn fits_missed(self, days: i32) -> bool {
        match self {
            Rhythm::Monthly => (52..=67).contains(&days),
            Rhythm::Quarterly => (170..=195).contains(&days),
            Rhythm::Yearly => (715..=745).contains(&days),
        }
    }

    /// The word the window shows.
    pub fn name(self) -> &'static str {
        match self {
            Rhythm::Monthly => "monatlich",
            Rhythm::Quarterly => "vierteljährlich",
            Rhythm::Yearly => "jährlich",
        }
    }

    pub fn from_name(name: &str) -> Option<Rhythm> {
        Rhythm::ALL.into_iter().find(|rhythm| rhythm.name() == name)
    }

    fn after(self, day: Date, times: i64) -> Date {
        day.checked_add(Span::new().months(self.months() * times))
            .unwrap_or(day)
    }
}

/// A changed amount: since which payment, and by how much more (negative: less) is paid or got.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Change {
    pub day: Date,
    pub more_cents: i64,
}

/// One recurring payment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recurring {
    pub party: i64,
    pub name: String,
    pub incoming: bool,
    pub rhythm: Rhythm,
    /// The first payment of this run, which with party, direction and rhythm names it.
    pub since: Date,
    pub last: Date,
    /// `None` once it ended.
    pub next: Option<Date>,
    pub count: u32,
    /// The last amount, as booked (money out negative).
    pub cents: i64,
    /// The last amount times the payments of a year.
    pub per_year: i64,
    pub change: Option<Change>,
    /// Twice its rhythm without a payment.
    pub ended: bool,
}

impl Recurring {
    /// "seit März 2026 2,00 € mehr", the change as the window shows it.
    pub fn change_text(&self) -> Option<String> {
        let change = self.change?;
        let word = if change.more_cents >= 0 {
            "mehr"
        } else {
            "weniger"
        };
        Some(format!(
            "seit {} {} {} {word}",
            MONTHS[usize::from(change.day.month().unsigned_abs()) - 1],
            change.day.year(),
            Cents::new(change.more_cents.abs())
        ))
    }
}

const MONTHS: [&str; 12] = [
    "Januar",
    "Februar",
    "März",
    "April",
    "Mai",
    "Juni",
    "Juli",
    "August",
    "September",
    "Oktober",
    "November",
    "Dezember",
];

/// Whether two amounts are near enough to be one payment: a fifth, at least a euro.
fn near(a: i64, b: i64) -> bool {
    (a - b).abs() <= (a.abs() / 5).max(100)
}

/// The chains of one party and direction: from each payment the next one in the rhythm, a missed
/// one allowed, with an amount near the last or a new amount that comes again (a changed price).
/// Monthly first, then quarterly, then yearly, each booking in one chain at most. A chain needs
/// three payments, and no more near amounts between its payments than half its own, so the
/// purchases at a shop are no recurring payment.
fn chains(bookings: &[(Date, i64)]) -> Vec<(Rhythm, Vec<(Date, i64)>)> {
    let gap = |a: usize, b: usize| (bookings[b].0 - bookings[a].0).get_days();
    let mut used = vec![false; bookings.len()];
    let mut chains = Vec::new();
    for rhythm in Rhythm::ALL {
        for start in 0..bookings.len() {
            if used[start] {
                continue;
            }
            let mut chain = vec![start];
            let mut cursor = start;
            loop {
                let last = bookings[cursor].1;
                // The same amount again in the rhythm: a new price, not a one-off.
                let comes_again = |j: usize| {
                    (j + 1..bookings.len()).any(|k| {
                        !used[k] && rhythm.fits(gap(j, k)) && bookings[k].1 == bookings[j].1
                    })
                };
                let next = [false, true].into_iter().find_map(|missed| {
                    (cursor + 1..bookings.len())
                        .filter(|j| !used[*j])
                        .filter(|j| {
                            let days = gap(cursor, *j);
                            if missed {
                                rhythm.fits_missed(days)
                            } else {
                                rhythm.fits(days)
                            }
                        })
                        .filter(|j| near(last, bookings[*j].1) || comes_again(*j))
                        .min_by_key(|j| (bookings[*j].1 - last).abs())
                });
                let Some(next) = next else {
                    break;
                };
                chain.push(next);
                cursor = next;
            }
            if chain.len() < 3 {
                continue;
            }
            let (first, end) = (chain[0], chain[chain.len() - 1]);
            let typical = bookings[end].1;
            let others = (first..=end)
                .filter(|j| !chain.contains(j) && near(typical, bookings[*j].1))
                .count();
            if others * 2 > chain.len() {
                continue;
            }
            for j in &chain {
                used[*j] = true;
            }
            chains.push((rhythm, chain.into_iter().map(|j| bookings[j]).collect()));
        }
    }
    chains
}

/// The change of the amount: the last amount differs from one that was paid at least twice before.
fn change_of(run: &[(Date, i64)]) -> Option<Change> {
    let (_, last) = *run.last()?;
    let start = run.iter().rposition(|(_, cents)| *cents != last)? + 1;
    let before = run[start - 1].1;
    let paid_twice = start >= 2 && run[start - 2].1 == before;
    paid_twice.then(|| Change {
        day: run[start].0,
        more_cents: last.abs() - before.abs(),
    })
}

/// What the owner said is none: party, direction, rhythm and the first day name a payment.
fn rejected(conn: &Connection) -> rusqlite::Result<Vec<(i64, bool, String, String)>> {
    conn.prepare("SELECT party_id, incoming, rhythm, since FROM recurring_none")?
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get::<_, i64>(1)? != 0,
                row.get(2)?,
                row.get(3)?,
            ))
        })?
        .collect()
}

/// The recurring payments of the archive as of `today`: the running first, the larger first,
/// those that ended last. None the owner said is none.
pub fn found(conn: &Connection, today: Date) -> rusqlite::Result<Vec<Recurring>> {
    let mut by_party: BTreeMap<(i64, bool), (String, Payments)> = BTreeMap::new();
    let rows = conn
        .prepare(BOOKINGS)?
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    for (party, name, day, cents) in rows {
        let Ok(day) = day.parse::<Date>() else {
            continue;
        };
        by_party
            .entry((party, cents >= 0))
            .or_insert_with(|| (name, Vec::new()))
            .1
            .push((day, cents));
    }
    let rejected = rejected(conn)?;
    // A statement comes weeks after its month: ended is judged by the last day the statements read
    // reach, or the last booking.
    let known: Option<String> = conn.query_row(
        "SELECT max(day) FROM (SELECT closing_day AS day FROM statements UNION ALL SELECT max(day) FROM bookings WHERE replaced_by IS NULL)",
        [],
        |row| row.get(0),
    )?;
    let until = known
        .and_then(|day| day.parse::<Date>().ok())
        .map_or(today, |day| day.min(today));
    let mut found = Vec::new();
    for ((party, incoming), (name, bookings)) in by_party {
        for (rhythm, run) in chains(&bookings) {
            {
                let (since, _) = run[0];
                let Some(&(last, cents)) = run.last() else {
                    continue;
                };
                let is_rejected = rejected.iter().any(|(p, i, r, s)| {
                    *p == party && *i == incoming && r == rhythm.name() && *s == since.to_string()
                });
                if is_rejected {
                    continue;
                }
                let ended = until > rhythm.after(last, 2);
                found.push(Recurring {
                    party,
                    name: name.clone(),
                    incoming,
                    rhythm,
                    since,
                    last,
                    next: (!ended).then(|| rhythm.after(last, 1)),
                    count: u32::try_from(run.len()).unwrap_or(u32::MAX),
                    cents,
                    per_year: cents * (12 / rhythm.months()),
                    change: change_of(&run),
                    ended,
                });
            }
        }
    }
    found.sort_by(|a, b| {
        a.ended
            .cmp(&b.ended)
            .then(b.per_year.abs().cmp(&a.per_year.abs()))
            .then(a.name.cmp(&b.name))
    });
    Ok(found)
}

/// The owner says a found payment is none. It is not found again.
pub fn reject(
    conn: &Connection,
    party: i64,
    incoming: bool,
    rhythm: Rhythm,
    since: Date,
    now: Timestamp,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO recurring_none (party_id, incoming, rhythm, since, decided_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![party, i64::from(incoming), rhythm.name(), since.to_string(), now.to_string()],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests;
