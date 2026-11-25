//! Exact money. An amount is a whole number of cents and never a float (FA-05).

use std::fmt;

/// An amount of money in euro cents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Cents(i64);

/// Why a printed amount could not be read. The message is German, for the owner.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AmountError {
    #[error("Kein Betrag angegeben.")]
    Empty,
    #[error("„{text}“ nutzt einen Punkt vor den Cent. Gemeint ist vermutlich „{suggestion}“.")]
    DotAsDecimal { text: String, suggestion: String },
    #[error("„{text}“ hat mehr als zwei Stellen nach dem Komma.")]
    TooManyDecimals { text: String },
    #[error("„{text}“ ist zu groß für einen Betrag.")]
    TooLarge { text: String },
    #[error("„{text}“ ist kein Betrag. Erwartet wird etwa „1.234,56“.")]
    Invalid { text: String },
}

impl Cents {
    pub const ZERO: Self = Self(0);

    pub const fn new(cents: i64) -> Self {
        Self(cents)
    }

    pub const fn value(self) -> i64 {
        self.0
    }

    /// An amount as a machine writes it, the XML of an e-invoice above all: "1234.56", "-5", "19.5"
    /// or "100.0000". A point before the decimals, no grouping. Decimals past the cent must be zero,
    /// so nothing is rounded. `None` for anything else.
    pub fn parse_decimal(text: &str) -> Option<Self> {
        let text = text.trim();
        let (negative, unsigned) = match text.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, text.strip_prefix('+').unwrap_or(text)),
        };
        let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
        let digits = |part: &str| part.bytes().all(|b| b.is_ascii_digit());
        let dangling_point = unsigned.ends_with('.');
        if whole.is_empty() || dangling_point || !digits(whole) || !digits(fraction) {
            return None;
        }
        let (cent_digits, rest) = fraction.split_at(fraction.len().min(2));
        if rest.bytes().any(|b| b != b'0') {
            return None;
        }
        let cents = match cent_digits.len() {
            0 => 0,
            _ => parse_fraction(cent_digits)?,
        };
        let value = to_cents(whole, cents)?;
        i64::try_from(if negative { -value } else { value })
            .ok()
            .map(Self)
    }

    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }

    pub fn checked_sub(self, other: Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Self)
    }

    /// Reads an amount as German documents print it: "1.234,56", "-12,50", "12,5", "1234", "49,99 €".
    /// A point before one or two final digits ("12.50") is refused, because a German document
    /// never writes cents that way and a guess could be off by a factor of a hundred.
    pub fn parse_de(text: &str) -> Result<Self, AmountError> {
        let owned = || text.to_owned();
        let cleaned = text.replace('\u{a0}', " ");
        let mut rest = cleaned.trim();
        for suffix in ["€", "EUR", "eur"] {
            if let Some(stripped) = rest.strip_suffix(suffix) {
                rest = stripped.trim_end();
            }
        }
        let negative = rest.starts_with('-') || rest.starts_with('\u{2212}');
        if negative || rest.starts_with('+') {
            let sign_len = rest.chars().next().map_or(0, char::len_utf8);
            rest = rest[sign_len..].trim_start();
        }
        if rest.is_empty() {
            return Err(AmountError::Empty);
        }
        if uses_dot_as_decimal(rest) {
            return Err(AmountError::DotAsDecimal {
                text: owned(),
                suggestion: swap_separators(rest),
            });
        }
        let (whole, fraction) = match rest.split_once(',') {
            Some((whole, fraction)) => (whole, Some(fraction)),
            None => (rest, None),
        };
        let euros = parse_whole(whole).ok_or_else(|| AmountError::Invalid { text: owned() })?;
        let cents = match fraction {
            None => 0,
            Some(digits) if digits.len() > 2 && digits.bytes().all(|b| b.is_ascii_digit()) => {
                return Err(AmountError::TooManyDecimals { text: owned() });
            }
            Some(digits) => {
                parse_fraction(digits).ok_or_else(|| AmountError::Invalid { text: owned() })?
            }
        };
        let magnitude =
            to_cents(&euros, cents).ok_or_else(|| AmountError::TooLarge { text: owned() })?;
        let signed = if negative { -magnitude } else { magnitude };
        i64::try_from(signed)
            .map(Self)
            .map_err(|_| AmountError::TooLarge { text: owned() })
    }
}

impl fmt::Display for Cents {
    /// "1.234,56 €", "-0,05 €", the way the owner reads amounts.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let magnitude = self.0.unsigned_abs();
        let euros = (magnitude / 100).to_string();
        let mut grouped = String::with_capacity(euros.len() + euros.len() / 3);
        for (index, digit) in euros.chars().enumerate() {
            if index > 0 && (euros.len() - index).is_multiple_of(3) {
                grouped.push('.');
            }
            grouped.push(digit);
        }
        let sign = if self.0 < 0 { "-" } else { "" };
        write!(f, "{sign}{grouped},{:02} €", magnitude % 100)
    }
}

/// "12.50" or "1,234.56": the last point stands before one or two digits, or after the comma.
fn uses_dot_as_decimal(text: &str) -> bool {
    if let Some((_, fraction)) = text.split_once(',') {
        return fraction.contains('.');
    }
    text.rsplit_once('.').is_some_and(|(_, tail)| {
        (1..=2).contains(&tail.len()) && tail.bytes().all(|b| b.is_ascii_digit())
    })
}

fn swap_separators(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '.' => ',',
            ',' => '.',
            other => other,
        })
        .collect()
}

/// The digits of whole euros: plain digits, or groups of three after a first group of one to three ("1.234.567").
fn parse_whole(text: &str) -> Option<String> {
    let groups: Vec<&str> = text.split('.').collect();
    let first = groups.first()?;
    let digits_only = |part: &str| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit());
    let well_formed = if groups.len() == 1 {
        digits_only(first)
    } else {
        (1..=3).contains(&first.len())
            && digits_only(first)
            && groups[1..]
                .iter()
                .all(|group| group.len() == 3 && digits_only(group))
    };
    if !well_formed {
        return None;
    }
    Some(groups.concat())
}

/// Euros and cents as one number, `None` when it does not fit.
fn to_cents(euro_digits: &str, cents: i128) -> Option<i128> {
    euro_digits
        .bytes()
        .try_fold(0_i128, |sum, digit| {
            sum.checked_mul(10)?.checked_add(i128::from(digit - b'0'))
        })?
        .checked_mul(100)?
        .checked_add(cents)
}

fn parse_fraction(text: &str) -> Option<i128> {
    match text.len() {
        1 | 2 if text.bytes().all(|b| b.is_ascii_digit()) => {
            let value: i128 = text.parse().ok()?;
            Some(if text.len() == 1 { value * 10 } else { value })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn cents(text: &str) -> i64 {
        Cents::parse_de(text)
            .map(Cents::value)
            .unwrap_or_else(|e| panic!("{text}: {e}"))
    }

    #[test]
    fn reads_german_amounts() {
        assert_eq!(cents("1.234,56"), 123_456);
        assert_eq!(cents("-12,50"), -1_250);
        assert_eq!(cents("12,5"), 1_250);
        assert_eq!(cents("1234"), 123_400);
        assert_eq!(cents("1.234"), 123_400);
        assert_eq!(cents("49,99 €"), 4_999);
        assert_eq!(cents("49,99\u{a0}EUR"), 4_999);
        assert_eq!(cents("\u{2212}3,00"), -300);
        assert_eq!(cents("+0,05"), 5);
        assert_eq!(cents("1.234.567,89"), 123_456_789);
        assert_eq!(cents("0,00"), 0);
    }

    #[test]
    fn refuses_a_point_before_the_cents_and_says_what_was_meant() {
        assert_eq!(
            Cents::parse_de("12.50"),
            Err(AmountError::DotAsDecimal {
                text: "12.50".into(),
                suggestion: "12,50".into()
            })
        );
        assert_eq!(
            Cents::parse_de("1,234.56"),
            Err(AmountError::DotAsDecimal {
                text: "1,234.56".into(),
                suggestion: "1.234,56".into()
            })
        );
        let message = Cents::parse_de("12.50")
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default();
        assert!(message.contains("12,50"), "{message}");
    }

    #[test]
    fn refuses_what_is_no_amount() {
        assert_eq!(Cents::parse_de("  "), Err(AmountError::Empty));
        assert_eq!(Cents::parse_de("€"), Err(AmountError::Empty));
        assert_eq!(
            Cents::parse_de("1,234"),
            Err(AmountError::TooManyDecimals {
                text: "1,234".into()
            })
        );
        for text in [
            "abc",
            "12,",
            ",50",
            "1.23.456",
            "12.3456,00",
            "1,2,3",
            "--5",
            "1 234,00",
            "12,5x",
            "12,+5",
            "12,-5",
            "12,+",
        ] {
            assert!(
                matches!(
                    Cents::parse_de(text),
                    Err(AmountError::Invalid { .. } | AmountError::DotAsDecimal { .. })
                ),
                "{text} was read"
            );
        }
    }

    #[test]
    fn refuses_amounts_beyond_the_range() {
        assert!(matches!(
            Cents::parse_de("99999999999999999999,00"),
            Err(AmountError::TooLarge { .. })
        ));
        assert!(matches!(
            Cents::parse_de("100000000000000000000000000000000000000"),
            Err(AmountError::TooLarge { .. })
        ));
    }

    #[test]
    fn prints_the_way_the_owner_reads() {
        assert_eq!(Cents::new(123_456).to_string(), "1.234,56 €");
        assert_eq!(Cents::new(-5).to_string(), "-0,05 €");
        assert_eq!(Cents::new(0).to_string(), "0,00 €");
        assert_eq!(Cents::new(100_000_000).to_string(), "1.000.000,00 €");
        assert_eq!(Cents::new(99_999).to_string(), "999,99 €");
    }

    #[test]
    fn adds_and_subtracts_without_overflow() {
        assert_eq!(
            Cents::new(150).checked_add(Cents::new(-50)),
            Some(Cents::new(100))
        );
        assert_eq!(Cents::new(i64::MAX).checked_add(Cents::new(1)), None);
        assert_eq!(Cents::new(i64::MIN).checked_sub(Cents::new(1)), None);
        assert_eq!(Cents::ZERO.checked_sub(Cents::new(7)), Some(Cents::new(-7)));
    }

    #[test]
    fn decimals_are_read_as_machines_write_them() {
        let read = |text: &str| Cents::parse_decimal(text).map(Cents::value);
        assert_eq!(read("1234.56"), Some(123_456));
        assert_eq!(read("19.5"), Some(1_950));
        assert_eq!(read("100"), Some(10_000));
        assert_eq!(read("100.0000"), Some(10_000));
        assert_eq!(read("-5.00"), Some(-500));
        assert_eq!(read("+5.00"), Some(500));
        assert_eq!(read(" 0.07 "), Some(7));
        assert_eq!(
            read("1.005"),
            None,
            "a decimal past the cent is never rounded"
        );
        for refused in [
            "",
            "-",
            ".5",
            "5.",
            "1,50",
            "1.234.56",
            "1e3",
            "12a",
            "--1",
            "99999999999999999999",
        ] {
            assert_eq!(read(refused), None, "{refused:?}");
        }
    }

    proptest! {
        #[test]
        fn writing_as_a_machine_then_reading_gives_the_same_amount(value in -(i64::MAX / 100)..(i64::MAX / 100)) {
            let sign = if value < 0 { "-" } else { "" };
            let written = format!("{sign}{}.{:02}", (value / 100).unsigned_abs(), (value % 100).unsigned_abs());
            prop_assert_eq!(Cents::parse_decimal(&written), Some(Cents::new(value)), "written as {}", written);
        }

        #[test]
        fn reading_a_decimal_never_panics(text in "\\PC{0,24}") {
            let _ = Cents::parse_decimal(&text);
        }

        #[test]
        fn printing_then_reading_gives_the_same_amount(value in any::<i64>()) {
            let printed = Cents::new(value).to_string();
            prop_assert_eq!(Cents::parse_de(&printed), Ok(Cents::new(value)), "printed as {}", printed);
        }

        #[test]
        fn reading_never_panics(text in "\\PC{0,24}") {
            let _ = Cents::parse_de(&text);
        }
    }
}
