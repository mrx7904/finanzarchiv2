//! Finding amounts, dates and IBANs in a printed line. Readers keep what is printed and let this
//! code turn it into numbers, never a model (lesson of 1.0: a model wrote 1234 for "1.234,56").

use std::sync::LazyLock;

use jiff::civil::Date;
use regex::Regex;

use crate::money::Cents;

static AMOUNT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[-−]?\d{1,3}(?:\.\d{3})+,\d{2}\b|[-−]?\d+,\d{2}\b")
        .unwrap_or_else(|_| unreachable!("a fixed pattern"))
});
static NUMERIC_DATE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(\d{1,2})\.(\d{1,2})\.(\d{4}|\d{2})\b")
        .unwrap_or_else(|_| unreachable!("a fixed pattern"))
});
static WORD_DATE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(\d{1,2})\.?\s+(januar|februar|märz|april|mai|juni|juli|august|september|oktober|november|dezember)\s+(\d{4})\b")
        .unwrap_or_else(|_| unreachable!("a fixed pattern"))
});
static IBAN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b[A-Z]{2}\d{2}(?: ?[A-Z0-9]{4}){3,7}(?: ?[A-Z0-9]{1,3})?\b")
        .unwrap_or_else(|_| unreachable!("a fixed pattern"))
});
static PERCENT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(\d{1,2})(?:,\d+)?\s?%").unwrap_or_else(|_| unreachable!("a fixed pattern"))
});

const MONTHS: [&str; 12] = [
    "januar",
    "februar",
    "märz",
    "april",
    "mai",
    "juni",
    "juli",
    "august",
    "september",
    "oktober",
    "november",
    "dezember",
];

/// The amounts of a line as printed, left to right.
pub fn amounts(line: &str) -> Vec<(String, Cents)> {
    AMOUNT
        .find_iter(line)
        .filter_map(|found| {
            Cents::parse_de(found.as_str())
                .ok()
                .map(|cents| (found.as_str().to_owned(), cents))
        })
        .collect()
}

/// The last amount of a line: in a row of label and figures, the figure that counts stands at the right.
pub fn last_amount(line: &str) -> Option<(String, Cents)> {
    amounts(line).pop()
}

/// The first date of a line as printed and as a date: "20.11.2026", "1.3.26", "5. März 2026".
pub fn first_date(line: &str) -> Option<(String, Date)> {
    let numeric = NUMERIC_DATE.captures(line).and_then(|found| {
        let day: i8 = found[1].parse().ok()?;
        let month: i8 = found[2].parse().ok()?;
        let year: i16 = match found[3].len() {
            2 => 2000 + found[3].parse::<i16>().ok()?,
            _ => found[3].parse().ok()?,
        };
        Some((
            found.get(0)?.start(),
            found[0].to_owned(),
            Date::new(year, month, day).ok()?,
        ))
    });
    let word = WORD_DATE.captures(line).and_then(|found| {
        let day: i8 = found[1].parse().ok()?;
        let month = MONTHS
            .iter()
            .position(|name| found[2].to_lowercase() == *name)?;
        let year: i16 = found[3].parse().ok()?;
        Some((
            found.get(0)?.start(),
            found[0].to_owned(),
            Date::new(year, i8::try_from(month + 1).ok()?, day).ok()?,
        ))
    });
    [numeric, word]
        .into_iter()
        .flatten()
        .min_by_key(|(at, _, _)| *at)
        .map(|(_, printed, date)| (printed, date))
}

/// The first IBAN of a line whose check sum holds, as printed.
pub fn iban(line: &str) -> Option<String> {
    IBAN.find_iter(line)
        .map(|found| found.as_str().to_owned())
        .find(|candidate| iban_is_valid(candidate))
}

pub fn iban_is_valid(printed: &str) -> bool {
    let compact: String = printed.chars().filter(|c| !c.is_whitespace()).collect();
    if compact.len() < 15 || compact.len() > 34 || !compact.is_ascii() {
        return false;
    }
    let moved = format!("{}{}", &compact[4..], &compact[..4]);
    let mut rest: u32 = 0;
    for c in moved.chars() {
        let digits = match c {
            '0'..='9' => c.to_digit(10).map(|d| vec![d]),
            'A'..='Z' => Some(
                (u32::from(c) - 55)
                    .to_string()
                    .chars()
                    .filter_map(|d| d.to_digit(10))
                    .collect(),
            ),
            _ => None,
        };
        let Some(digits) = digits else { return false };
        for digit in digits {
            rest = (rest * 10 + digit) % 97;
        }
    }
    rest == 1
}

/// The first percentage of a line: "19 %" gives 19.
pub fn percent(line: &str) -> Option<(String, u32)> {
    PERCENT
        .captures(line)
        .and_then(|found| Some((found.get(0)?.as_str().to_owned(), found[1].parse().ok()?)))
}

/// Whether a lowered line holds the label at the start of a word, so "datum" is not found in
/// "geburtsdatum".
pub fn has_label(lower: &str, label: &str) -> bool {
    lower.match_indices(label).any(|(at, _)| {
        lower[..at]
            .chars()
            .next_back()
            .is_none_or(|before| !before.is_alphanumeric())
    })
}

/// The line with its label removed and trimmed of separators, for values such as an invoice number.
pub fn after_label<'a>(line: &'a str, label_lower: &str) -> Option<&'a str> {
    let lower = line.to_lowercase();
    let at = lower.find(label_lower)?;
    // Lowering can change byte lengths only outside ASCII, labels are compared on the same text.
    let rest = line.get(at + label_lower.len()..)?;
    Some(rest.trim_start_matches([':', '.', ' ', '#']).trim())
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date;

    #[test]
    fn finds_the_amounts_of_a_line() {
        let found = amounts("Umsatzsteuer 19 % von 1.234,56 = 234,57 EUR");
        assert_eq!(
            found.iter().map(|(p, _)| p.as_str()).collect::<Vec<_>>(),
            ["1.234,56", "234,57"]
        );
        assert_eq!(
            last_amount("Rechnungsbetrag 1.469,13").map(|(_, c)| c.value()),
            Some(146_913)
        );
        assert_eq!(
            last_amount("Gutschrift -12,50").map(|(_, c)| c.value()),
            Some(-1_250)
        );
        assert_eq!(last_amount("Kundennummer 12345"), None);
        assert_eq!(last_amount("Seite 1 von 2, 2026"), None);
    }

    #[test]
    fn finds_dates_in_both_ways_of_writing() {
        assert_eq!(
            first_date("Rechnungsdatum 20.11.2026"),
            Some(("20.11.2026".into(), date(2026, 11, 20)))
        );
        assert_eq!(
            first_date("vom 1.3.26"),
            Some(("1.3.26".into(), date(2026, 3, 1)))
        );
        assert_eq!(
            first_date("Dortmund, 5. März 2026"),
            Some(("5. März 2026".into(), date(2026, 3, 5)))
        );
        assert_eq!(first_date("am 31.02.2026"), None, "no such day");
        assert_eq!(first_date("Version 1.2.3"), None);
    }

    #[test]
    fn an_iban_counts_only_with_a_valid_check_sum() {
        assert_eq!(
            iban("IBAN DE89 3704 0044 0532 0130 00 BIC X"),
            Some("DE89 3704 0044 0532 0130 00".into())
        );
        assert_eq!(iban("IBAN DE88 3704 0044 0532 0130 00"), None);
        assert!(iban_is_valid("GB82WEST12345698765432"));
        assert!(!iban_is_valid("DE89"));
    }

    #[test]
    fn a_label_counts_only_at_the_start_of_a_word() {
        assert!(has_label("rechnung vom 03.03.2025", "rechnung vom"));
        assert!(!has_label("geburtsdatum 01.01.1990", "datum"));
        assert!(has_label("datum: 01.01.2026", "datum"));
    }

    #[test]
    fn finds_percent_and_the_rest_after_a_label() {
        assert_eq!(
            percent("Umsatzsteuer 19 % 234,57"),
            Some(("19 %".into(), 19))
        );
        assert_eq!(percent("MwSt. 7%"), Some(("7%".into(), 7)));
        assert_eq!(
            after_label("Rechnungsnummer: R-2026-0042", "rechnungsnummer"),
            Some("R-2026-0042")
        );
    }
}
