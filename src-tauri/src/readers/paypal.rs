//! PayPal's payment mails (#32, FA-36), read by code. Only the text of a mail whose sender is one
//! of PayPal's own domains counts, never a document that only names PayPal. Read are the payee,
//! the amount, the day and PayPal's transaction code, and the seller's order number where PayPal
//! prints it. Measured layout on the owner's mail (2026-11-26, masked): "Sie haben 9,99 € EUR an
//! … gezahlt", the lines "Händler …", "Transaktionsdatum …", "Bestellnummer …", "Gesamtbetrag …" and
//! "Transaktionscode: …", the code sometimes on the next line.

use std::sync::LazyLock;

use regex::Regex;

use super::parse::{after_label, first_date, last_amount};
use super::{Input, Line, Normalized, Reader, Value};

static DOMAIN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^([a-z0-9-]+\.)*paypal\.(de|com|at|ch|fr|it|es|nl|be|pl|co\.uk)$")
        .unwrap_or_else(|_| unreachable!("a fixed pattern"))
});

static SENTENCE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(?:sie haben|du hast)\s+(.+?)\s+an\s+(.+?)\s+(?:gezahlt|gesendet|bezahlt)\b")
        .unwrap_or_else(|_| unreachable!("a fixed pattern"))
});

static CODE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b([0-9A-Z]{12,20})\b").unwrap_or_else(|_| unreachable!("a fixed pattern"))
});

/// The address of a "Von:" line, `info@example.com` of `Von: Beispiel <info@example.com>`.
fn address_of(from: &str) -> Option<&str> {
    let from = from.trim();
    match (from.rfind('<'), from.rfind('>')) {
        (Some(open), Some(close)) if open < close => Some(from[open + 1..close].trim()),
        _ => from.split_whitespace().find(|word| word.contains('@')),
    }
}

/// Whether a domain is one of PayPal's own, with any subdomain.
pub fn own_domain(domain: &str) -> bool {
    DOMAIN.is_match(domain)
}

/// Whether the "Von:" line of the text of a mail names one of PayPal's own domains.
pub fn from_paypal(lines: &[Line]) -> bool {
    lines
        .iter()
        .take(6)
        .find_map(|line| line.text.strip_prefix("Von:"))
        .and_then(address_of)
        .and_then(|address| address.rsplit_once('@'))
        .is_some_and(|(_, domain)| own_domain(domain))
}

/// A name as PayPal prints it, the dots of a cut name dropped.
fn tidy_name(name: &str) -> Option<String> {
    let name = name
        .trim()
        .trim_end_matches(['…', '.'])
        .trim()
        .trim_matches(['"', '„', '“', '\''])
        .trim();
    (!name.is_empty() && name.chars().any(char::is_alphabetic)).then(|| name.to_owned())
}

/// The payment a PayPal mail tells, as far as code could read it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Payment {
    pub payee: Option<Value>,
    pub amount: Option<Value>,
    pub day: Option<Value>,
    pub code: Option<Value>,
    pub order: Option<Value>,
}

impl Payment {
    /// Payee, amount and day are what make it a payment; the code may be missing in older mails.
    pub fn complete(&self) -> bool {
        self.payee.is_some() && self.amount.is_some() && self.day.is_some()
    }

    pub fn code_text(&self) -> Option<String> {
        self.code.as_ref().map(|value| value.value.stored())
    }
}

fn starts_with_label(text: &str, label: &str) -> bool {
    let lower = text.to_lowercase();
    lower.starts_with(label)
        && lower[label.len()..]
            .chars()
            .next()
            .is_none_or(|c| !c.is_alphanumeric())
}

/// Reads the payment of a PayPal mail, `None` when the mail is not from PayPal.
pub fn read_payment(lines: &[Line]) -> Option<Payment> {
    if !from_paypal(lines) {
        return None;
    }
    let mut payment = Payment::default();
    let sentence = lines.iter().find_map(|line| {
        SENTENCE
            .captures(line.text.trim())
            .map(|found| (line, found[1].to_owned(), found[2].to_owned()))
    });
    // The payee: the seller's line, then PayPal's sentence, then the subject.
    payment.payee = lines
        .iter()
        .filter(|line| starts_with_label(&line.text, "händler"))
        .find_map(|line| {
            let name = tidy_name(after_label(&line.text, "händler")?)?;
            Some(Value::on(
                "sender",
                name.clone(),
                Normalized::Text(name),
                line,
            ))
        })
        .or_else(|| {
            let (line, _, name) = sentence.as_ref()?;
            let name = tidy_name(name)?;
            Some(Value::on(
                "sender",
                name.clone(),
                Normalized::Text(name),
                line,
            ))
        })
        .or_else(|| {
            lines.iter().take(6).find_map(|line| {
                let subject = line.text.strip_prefix("Betreff:")?;
                let at = subject.to_lowercase().rfind("zahlung an ")?;
                let name = tidy_name(&subject[at + "zahlung an ".len()..])?;
                Some(Value::on(
                    "sender",
                    name.clone(),
                    Normalized::Text(name),
                    line,
                ))
            })
        });
    // The amount paid: PayPal's sentence, else the total.
    payment.amount = sentence
        .as_ref()
        .and_then(|(line, amount, _)| {
            let (printed, cents) = last_amount(amount)?;
            Some(Value::on("gross", printed, Normalized::Cents(cents), line))
        })
        .or_else(|| {
            lines
                .iter()
                .filter(|line| starts_with_label(&line.text, "gesamtbetrag"))
                .find_map(|line| {
                    let (printed, cents) = last_amount(&line.text)?;
                    Some(Value::on("gross", printed, Normalized::Cents(cents), line))
                })
        });
    // The day: the day of the transaction, else the day of the mail.
    payment.day = lines
        .iter()
        .filter(|line| starts_with_label(&line.text, "transaktionsdatum"))
        .chain(
            lines
                .iter()
                .take(6)
                .filter(|line| line.text.starts_with("Datum:")),
        )
        .find_map(|line| {
            let (printed, date) = first_date(&line.text)?;
            Some(Value::on(
                "invoice_date",
                printed,
                Normalized::Date(date),
                line,
            ))
        });
    // PayPal's transaction code, on the line of its label or the next.
    payment.code = lines.iter().enumerate().find_map(|(at, line)| {
        let lower = line.text.to_lowercase();
        let label = ["transaktionscode", "transaktions-id"]
            .into_iter()
            .find(|label| lower.contains(label))?;
        let rest = after_label(&line.text, label).unwrap_or("");
        let (text, line) = if CODE.is_match(rest) {
            (rest.to_owned(), line)
        } else {
            let next = lines.get(at + 1)?;
            (next.text.clone(), next)
        };
        let code = CODE
            .captures(&text)
            .map(|found| found[1].to_owned())
            .filter(|code| {
                code.chars().any(|c| c.is_ascii_digit())
                    && code.chars().any(|c| c.is_ascii_uppercase())
            })?;
        Some(Value::on(
            "invoice_number",
            code.clone(),
            Normalized::Text(code),
            line,
        ))
    });
    payment.order = lines
        .iter()
        .filter(|line| starts_with_label(&line.text, "bestellnummer"))
        .find_map(|line| {
            let token = after_label(&line.text, "bestellnummer")?
                .split_whitespace()
                .next()?
                .to_owned();
            token
                .chars()
                .any(|c| c.is_ascii_digit())
                .then(|| Value::on("order_number", token.clone(), Normalized::Text(token), line))
        });
    Some(payment)
}

/// The field that marks a document as a payment over PayPal.
pub const VIA: &str = "paid_via";

/// The reader of PayPal's mails. It stands in for the reader of invoices on such a mail.
pub struct PayPalCode;

impl Reader for PayPalCode {
    fn name(&self) -> String {
        format!("code:{}", super::READERS_VERSION)
    }

    fn read(&self, input: &Input) -> Vec<Value> {
        let Some(payment) = read_payment(&input.lines) else {
            return Vec::new();
        };
        let mut values: Vec<Value> = [
            payment.payee.clone(),
            payment.amount.clone(),
            payment.day.clone(),
            payment.code.clone(),
            payment.order.clone(),
        ]
        .into_iter()
        .flatten()
        .collect();
        if let Some(first) = input.lines.first() {
            values.push(Value::on(
                VIA,
                "PayPal",
                Normalized::Text("PayPal".into()),
                first,
            ));
        }
        values
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kinds::Kind;

    pub fn lines(texts: &[&str]) -> Vec<Line> {
        texts
            .iter()
            .enumerate()
            .map(|(at, text)| Line {
                page: 1,
                text: (*text).to_owned(),
                left: 0.0,
                bottom: -(at as f64) - 1.0,
                right: 0.0,
                top: -(at as f64),
            })
            .collect()
    }

    fn mail(from: &str) -> Vec<Line> {
        lines(&[
            &format!("Von: PayPal <{from}>"),
            "An: test@example.com",
            "Datum: 26.11.2026 10:15",
            "Betreff: Beleg für Ihre Zahlung an Beispiel Musik AB",
            "",
            "Hallo Max Muster!",
            "Sie haben 9,99 € EUR an Beispiel Musik AB gezahlt",
            "Händler Beispiel Musik AB",
            "Transaktionsdatum 25.11.2026",
            "Bestellnummer B1X2Y3",
            "Zwischensumme 9,99 €",
            "Gesamtbetrag 9,99 € EUR",
            "Transaktionscode: 1AB23456CD7890123",
        ])
    }

    fn stored(values: &[Value], field: &str) -> Option<String> {
        values
            .iter()
            .find(|value| value.field == field)
            .map(|value| value.value.stored())
    }

    #[test]
    fn a_payment_mail_of_paypal_is_read_by_code() {
        let values = PayPalCode.read(&Input {
            kind: Kind::Receipt,
            lines: mail("service@paypal.de"),
            e_invoice: None,
        });
        assert_eq!(
            stored(&values, "sender").as_deref(),
            Some("Beispiel Musik AB")
        );
        assert_eq!(stored(&values, "gross").as_deref(), Some("999"));
        assert_eq!(
            stored(&values, "invoice_date").as_deref(),
            Some("2026-11-25")
        );
        assert_eq!(
            stored(&values, "invoice_number").as_deref(),
            Some("1AB23456CD7890123")
        );
        assert_eq!(stored(&values, "order_number").as_deref(), Some("B1X2Y3"));
        assert_eq!(stored(&values, VIA).as_deref(), Some("PayPal"));
    }

    #[test]
    fn only_paypal_own_domains_count() {
        for own in [
            "service@paypal.de",
            "service@paypal.com",
            "noreply@mail.paypal.de",
            "x@e.paypal.co.uk",
        ] {
            assert!(from_paypal(&mail(own)), "{own}");
        }
        // A display name or a local part that says PayPal is not PayPal.
        assert!(!from_paypal(&mail("paypal@example.com")));
        assert!(read_payment(&mail("paypal@example.com")).is_none());
        for other in [
            "paypal.de.example.com",
            "notpaypal.de",
            "paypal-sicherheit.de",
            "example.com",
        ] {
            assert!(!own_domain(other), "{other}");
        }
    }

    #[test]
    fn the_payee_comes_from_the_sentence_or_the_subject_and_the_code_from_the_next_line() {
        let older = lines(&[
            "Von: PayPal <service@paypal.de>",
            "Datum: 02.03.2026 08:00",
            "Betreff: Beleg für Ihre Zahlung an Beispiel Kino…",
            "",
            "Sie haben 12,50 € EUR an Beispiel Kino… gezahlt",
            "Transaktionscode",
            "9ZY87654XW3210987",
        ]);
        let payment = read_payment(&older).unwrap();
        assert_eq!(payment.payee.unwrap().value.stored(), "Beispiel Kino");
        assert_eq!(payment.amount.unwrap().value.stored(), "1250");
        assert_eq!(
            payment.day.unwrap().value.stored(),
            "2026-03-02",
            "the day of the mail"
        );
        assert_eq!(payment.code.unwrap().value.stored(), "9ZY87654XW3210987");

        let subject_only = lines(&[
            "Von: PayPal <service@paypal.de>",
            "Datum: 02.03.2026 08:00",
            "Betreff: Beleg für Ihre Zahlung an Beispiel Verein",
            "",
            "Sie haben 5,00 € EUR an gezahlt",
            "Händler",
        ]);
        let payment = read_payment(&subject_only).unwrap();
        assert_eq!(payment.payee.unwrap().value.stored(), "Beispiel Verein");
        assert!(payment.code.is_none(), "an older layout prints none");
    }
}
