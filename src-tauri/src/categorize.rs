//! Categories by the model (#43, FA-46): every party without a category gets one of the archive's
//! categories, or none, from the local model. A category the owner chose is never changed
//! (`parties.category_by`), and a party is asked once.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use jiff::Timestamp;
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::json;

use crate::archive::Opened;
use crate::money::Cents;
use crate::ollama::{LocalModel, ModelError, Question};

/// What the model learns of a party.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    pub party: i64,
    pub name: String,
    pub outgoing: u32,
    pub incoming: u32,
    /// The most frequent amount and how often it came.
    pub typical: Option<(Cents, u32)>,
    pub rhythm: Option<&'static str>,
    pub kinds: Vec<String>,
    pub purposes: Vec<String>,
    pub senders: Vec<String>,
}

/// A few words on a category, for the model only. The window never shows them.
fn hint(name: &str) -> &'static str {
    match name {
        "Gehalt und Lohn" => "Arbeitgeber, Lohn, Gehalt",
        "Rente und Sozialleistungen" => {
            "Rentenversicherung, Kindergeld, Familienkasse, Arbeitsagentur, Elterngeld"
        }
        "Erstattungen" => "Rückzahlungen, Steuererstattung, Erstattung einer Versicherung",
        "Zinsen und Erträge" => "Zinsen, Dividenden, Ausschüttungen",
        "Sonstige Einnahmen" => "Verkäufe, Geld von Privatpersonen",
        "Miete und Wohnen" => "Vermieter, Hausverwaltung, Nebenkosten, Wohnungsbaugenossenschaft",
        "Strom und Heizung" => "Stadtwerke, Energieversorger, Strom, Gas, Fernwärme",
        "Internet und Telefon" => "Telekom, Vodafone, O2, 1&1, Mobilfunk, Internet",
        "Versicherungen" => {
            "Haftpflicht, Hausrat, Kfz-Versicherung, Zusatzversicherung, Versicherer"
        }
        "Lebensmittel" => "Supermarkt, REWE, EDEKA, ALDI, LIDL, Kaufland, Netto, Penny",
        "Drogerie und Haushalt" => "dm, Rossmann, Müller, IKEA, Baumarkt, Haushaltswaren",
        "Restaurant und Café" => "Restaurant, Café, Imbiss, Lieferando, Bäckerei, Kantine",
        "Auto und Tanken" => "Tankstelle, Werkstatt, Parken, Autowäsche, ADAC",
        "Bus und Bahn" => "Deutsche Bahn, Verkehrsverbund, Deutschlandticket, Nahverkehr",
        "Gesundheit und Apotheke" => "Apotheke, Arzt, Zahnarzt, Optiker, Physiotherapie",
        "Kleidung" => "Mode, Schuhe, Zalando, H&M, C&A",
        "Freizeit und Hobby" => {
            "Kino, Sport, Fitnessstudio, Spiele, Steam, Nintendo, Verein, Konzert"
        }
        "Reisen und Urlaub" => "Hotel, Airbnb, Flug, Reiseveranstalter, Ferienwohnung",
        "Abos und Medien" => {
            "Spotify, Netflix, Disney+, Apple, Google, Microsoft, Zeitung, Streaming, Software"
        }
        "Bildung" => "Hochschule, Studiengebühren, Kurs, Schule, Fachbücher",
        "Kinder" => "Kita, Kindergarten, Spielzeug, Babybedarf",
        "Geschenke und Spenden" => "Spende, Hilfsorganisation, gemeinnützig, Geschenk",
        "Steuern und Abgaben" => {
            "Finanzamt, Rundfunkbeitrag, Kfz-Steuer, Gemeinde, Gebühren der Behörden"
        }
        "Bankgebühren" => "Kontoführung, Kartengebühr, Entgelt der Bank",
        "Kredit und Tilgung" => "Darlehen, Kredit, Rate, Tilgung, Finanzierung",
        "Bargeld" => "Geldautomat, Bargeldauszahlung, Abhebung",
        "Sparen und Anlegen" => "Depot, Broker, Sparplan, Tagesgeld, ETF",
        "Sonstige Ausgaben" => "nur Händler mit gemischtem Sortiment wie Amazon oder Otto",
        _ => "",
    }
}

const SYSTEM: &str = "Du ordnest Zahlungspartner eines deutschen Privathaushalts einer Kategorie zu. \
Wähle eine Kategorie der Liste, in der Richtung der Zahlungen (ein für Geld, das kommt, aus für Geld, das geht), \
nur wenn Name oder Verwendungszweck sie klar zeigen. \
Ist der Partner eine Privatperson (ein Vor- und Nachname) oder ist der Zweck unklar, antworte keine. \
Sonstige Ausgaben ist kein Auffang für Unklares. Erfinde nichts.";

/// Payment services: their payee tells the category, never their own name.
const PAYMENT_SERVICES: &[&str] = &[
    "paypal",
    "klarna",
    "amazon pay",
    "sofort",
    "giropay",
    "apple pay",
    "google pay",
    "stripe",
    "adyen",
    "mollie",
];

/// Whether a party is a payment service, which gets no category of its own.
pub fn payment_service(name: &str) -> bool {
    let name = name.to_lowercase();
    PAYMENT_SERVICES
        .iter()
        .any(|service| name.contains(service))
}

/// The question for one party.
pub fn prompt(context: &Context, categories: &[(i64, String, String)]) -> String {
    let mut text = String::from("Kategorien:\n");
    for (_, name, direction) in categories {
        let hint = hint(name);
        let direction = if direction == "in" { "ein" } else { "aus" };
        if hint.is_empty() {
            text.push_str(&format!("- {name} ({direction})\n"));
        } else {
            text.push_str(&format!("- {name} ({direction}): {hint}\n"));
        }
    }
    text.push_str(&format!(
        "\nPartei: {}\nBuchungen: {} Ausgänge, {} Eingänge",
        context.name, context.outgoing, context.incoming
    ));
    if let Some((cents, times)) = context.typical {
        text.push_str(&format!(", meist {cents} ({times}-mal)"));
    }
    if let Some(rhythm) = context.rhythm {
        text.push_str(&format!(", {rhythm}"));
    }
    if !context.kinds.is_empty() {
        text.push_str(&format!("\nArten: {}", context.kinds.join(", ")));
    }
    if !context.purposes.is_empty() {
        text.push_str("\nVerwendungszwecke:");
        for purpose in &context.purposes {
            text.push_str(&format!("\n- {purpose}"));
        }
    }
    if !context.senders.is_empty() {
        text.push_str(&format!("\nBelege von: {}", context.senders.join(", ")));
    }
    text
}

/// Asks the model for the category of a party: `Ok(None)` when none fits.
pub fn ask(
    model: &dyn LocalModel,
    context: &Context,
    categories: &[(i64, String, String)],
) -> Result<Option<i64>, ModelError> {
    let mut names: Vec<&str> = categories
        .iter()
        .map(|(_, name, _)| name.as_str())
        .collect();
    names.push("keine");
    let schema = json!({
        "type": "object",
        "properties": { "category": { "type": "string", "enum": names } },
        "required": ["category"],
    });
    let answer = model.ask(&Question {
        system: SYSTEM,
        prompt: &prompt(context, categories),
        schema: &schema,
        images: &[],
    })?;
    let answer: serde_json::Value = serde_json::from_str(&answer).unwrap_or_default();
    let chosen = answer["category"].as_str().unwrap_or("keine");
    // Code checks what it can: the direction of the category is the one of most bookings.
    let wanted = if context.incoming > context.outgoing {
        "in"
    } else {
        "out"
    };
    Ok(categories
        .iter()
        .find(|(_, name, direction)| name == chosen && direction == wanted)
        .map(|(id, _, _)| *id))
}

fn trimmed(text: &str, at_most: usize) -> String {
    let text: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    text.chars().take(at_most).collect()
}

/// The rhythm of days between bookings, when they keep one.
fn rhythm(days: &[jiff::civil::Date]) -> Option<&'static str> {
    if days.len() < 3 {
        return None;
    }
    let mut gaps: Vec<i32> = days
        .windows(2)
        .map(|pair| (pair[1] - pair[0]).get_days())
        .collect();
    gaps.sort_unstable();
    let middle = gaps[gaps.len() / 2];
    match middle {
        26..=35 => Some("monatlich"),
        85..=95 => Some("vierteljährlich"),
        355..=375 => Some("jährlich"),
        5..=9 => Some("wöchentlich"),
        _ => None,
    }
}

/// What the model learns of a party, from its bookings and documents.
pub fn context_of(conn: &Connection, party: i64) -> rusqlite::Result<Context> {
    let name: String =
        conn.query_row("SELECT name FROM parties WHERE id = ?1", [party], |row| {
            row.get(0)
        })?;
    let bookings: Vec<(String, i64, String, String)> = conn
        .prepare("SELECT day, amount_cents, kind, purpose FROM bookings WHERE party_id = ?1 AND replaced_by IS NULL ORDER BY day")?
        .query_map([party], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let outgoing = u32::try_from(bookings.iter().filter(|b| b.1 < 0).count()).unwrap_or(u32::MAX);
    let incoming = u32::try_from(bookings.iter().filter(|b| b.1 >= 0).count()).unwrap_or(u32::MAX);
    let mut amounts: std::collections::BTreeMap<i64, u32> = std::collections::BTreeMap::new();
    for booking in &bookings {
        *amounts.entry(booking.1).or_default() += 1;
    }
    let typical = amounts
        .iter()
        .max_by_key(|(cents, times)| (**times, -cents.abs()))
        .map(|(cents, times)| (Cents::new(*cents), *times));
    let days: Vec<jiff::civil::Date> = bookings.iter().filter_map(|b| b.0.parse().ok()).collect();
    let mut kinds: Vec<String> = Vec::new();
    let mut purposes: Vec<String> = Vec::new();
    for (_, _, kind, purpose) in bookings.iter().rev() {
        if !kinds.contains(kind) && kinds.len() < 3 {
            kinds.push(kind.clone());
        }
        let purpose = trimmed(purpose, 90);
        if !purpose.is_empty() && !purposes.contains(&purpose) && purposes.len() < 6 {
            purposes.push(purpose);
        }
    }
    let senders: Vec<String> = conn
        .prepare(
            "SELECT DISTINCT v.value FROM field_values v JOIN documents d ON d.id = v.document_id
             WHERE d.party_id = ?1 AND v.field IN ('sender', 'employer') LIMIT 3",
        )?
        .query_map([party], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(Context {
        party,
        name,
        outgoing,
        incoming,
        typical,
        rhythm: rhythm(&days),
        kinds,
        purposes,
        senders,
    })
}

/// The categories as the model gets them.
pub fn categories(conn: &Connection) -> rusqlite::Result<Vec<(i64, String, String)>> {
    conn.prepare("SELECT id, name, direction FROM categories ORDER BY direction, position, id")?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .collect()
}

/// The parties to ask: no category, never asked, with at least one booking.
pub fn waiting(conn: &Connection, limit: u32) -> rusqlite::Result<Vec<i64>> {
    conn.prepare(
        "SELECT p.id FROM parties p WHERE p.category_id IS NULL AND p.category_asked_at IS NULL AND p.category_by IS NULL
         AND EXISTS (SELECT 1 FROM bookings b WHERE b.party_id = p.id AND b.replaced_by IS NULL)
         ORDER BY (SELECT count(*) FROM bookings b WHERE b.party_id = p.id) DESC, p.id LIMIT ?1",
    )?
    .query_map([limit], |row| row.get(0))?
    .collect()
}

/// Keeps what the model chose, unless the owner chose meanwhile.
pub fn keep(
    conn: &Connection,
    party: i64,
    category: Option<i64>,
    now: Timestamp,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE parties SET category_id = coalesce(category_id, ?2),
            category_by = CASE WHEN category_id IS NULL AND ?2 IS NOT NULL THEN 'model' ELSE category_by END,
            category_asked_at = ?3
         WHERE id = ?1 AND (category_by IS NULL OR category_by <> 'owner')",
        params![party, category, now.to_string()],
    )?;
    Ok(())
}

fn with<T>(
    archive: &Mutex<Opened>,
    run: impl FnOnce(&Connection) -> rusqlite::Result<T>,
) -> Option<T> {
    let guard = archive.lock().ok()?;
    let Opened::Ready(ready) = &*guard else {
        return None;
    };
    run(ready.connection()).ok()
}

/// Asks the model for up to `limit` parties, the archive locked only to read and to keep.
/// Returns how many were asked, `None` when the model did not answer (they wait for the next pass).
pub fn pass(archive: &Mutex<Opened>, model: &dyn LocalModel, limit: u32) -> Option<usize> {
    let (categories, contexts) = with(archive, |conn| {
        let categories = categories(conn)?;
        let contexts = waiting(conn, limit)?
            .into_iter()
            .map(|party| context_of(conn, party))
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok((categories, contexts))
    })?;
    for context in &contexts {
        if payment_service(&context.name) {
            with(archive, |conn| {
                keep(conn, context.party, None, Timestamp::now())
            });
            continue;
        }
        let chosen = match ask(model, context, &categories) {
            Ok(chosen) => chosen,
            Err(error) if error.should_wait() => return None,
            Err(_) => None,
        };
        with(archive, |conn| {
            keep(conn, context.party, chosen, Timestamp::now())
        });
    }
    Some(contexts.len())
}

/// A pass a minute and a half after start, then every ten minutes, 40 parties at a time until
/// none waits.
pub fn start(archive: Arc<Mutex<Opened>>, model: Arc<dyn LocalModel>) {
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(90));
        loop {
            while let Some(asked) = pass(&archive, model.as_ref(), 40) {
                if asked == 0 {
                    break;
                }
            }
            std::thread::sleep(Duration::from_secs(600));
        }
    });
}

/// Whether a party's category was given by the model.
pub fn by_model(conn: &Connection, party: i64) -> rusqlite::Result<bool> {
    Ok(conn
        .query_row(
            "SELECT category_by FROM parties WHERE id = ?1",
            [party],
            |row| row.get::<_, Option<String>>(0),
        )
        .optional()?
        .flatten()
        .as_deref()
        == Some("model"))
}

#[cfg(test)]
mod tests;
