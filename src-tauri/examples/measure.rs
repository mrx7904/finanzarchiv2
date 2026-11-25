//! Measures a reader on a corpus (FA-13, #09): per kind and field, how many values it got right.
//!
//!   cargo run --release --example measure -- --corpus ../fixtures/synthetic
//!
//! With `CORPUS_PRIVATE` set, the owner's documents in that folder are measured too, and their
//! result goes only to `measurements/private/`, which git ignores. A result holds counts, never a value.
//!
//! The one reader so far is `text-layer`: a field counts as right when its printed value stands in
//! a line of the text PDFium reads. It is the floor for every reader of stage 2: a value that is not
//! in the text layer must come from the image.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use finanzarchiv2_lib::pdf::{self, Pdfium};
use finanzarchiv2_lib::{documents, kinds, pages};
use serde_json::{Value, json};

#[derive(Default)]
struct Tally {
    right: u32,
    total: u32,
}

/// Whitespace folded, so a value broken over two runs still counts.
fn fold(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn lines_of(pdfium: &Pdfium, pdf: &Path) -> Vec<String> {
    fs::read(pdf)
        .ok()
        .and_then(|bytes| pages::read(pdfium, &bytes).ok())
        .map(|read| {
            read.iter()
                .flat_map(|page| page.lines.iter().map(|line| line.text.clone()))
                .collect()
        })
        .unwrap_or_default()
}

fn text_of(pdfium: &Pdfium, pdf: &Path) -> String {
    fold(&lines_of(pdfium, pdf).join(
        "
",
    ))
}

/// Every truth file below `dir` with its PDF.
fn documents(dir: &Path) -> Vec<(PathBuf, Value)> {
    let mut found = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return found;
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            found.extend(documents(&path));
        } else if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            let truth = fs::read(&path)
                .ok()
                .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
            if let Some(truth) = truth {
                found.push((path.with_extension("pdf"), truth));
            }
        }
    }
    found
}

fn measure(pdfium: &Pdfium, corpus: &Path) -> BTreeMap<String, Tally> {
    let mut tallies: BTreeMap<String, Tally> = BTreeMap::new();
    for (pdf, truth) in documents(corpus) {
        let text = text_of(pdfium, &pdf);
        let kind = truth["kind"].as_str().unwrap_or("unknown");
        let layer = if truth["text_layer"].as_bool().unwrap_or(true) {
            ""
        } else {
            " (scan)"
        };
        let mut values: Vec<(String, String)> = truth["fields"]
            .as_object()
            .map(|fields| {
                fields
                    .iter()
                    .filter_map(|(name, value)| Some((name.clone(), value.as_str()?.to_owned())))
                    .collect()
            })
            .unwrap_or_default();
        if let Some(bookings) = truth["bookings"].as_array() {
            values.extend(bookings.iter().filter_map(|booking| {
                Some((
                    "booking".to_owned(),
                    booking["printed"].as_str()?.to_owned(),
                ))
            }));
        }
        for (field, value) in values {
            let tally = tallies.entry(format!("{kind}{layer}.{field}")).or_default();
            tally.total += 1;
            if text.contains(&fold(&value)) {
                tally.right += 1;
            }
        }
    }
    tallies
}

/// The reader `kind`: does code name the kind of each document right? A scan has no text, so
/// "unknown" is right for it until #15 reads it as an image.
fn measure_kind(pdfium: &Pdfium, corpus: &Path) -> BTreeMap<String, Tally> {
    let mut tallies: BTreeMap<String, Tally> = BTreeMap::new();
    for (pdf, truth) in documents(corpus) {
        let scan = !truth["text_layer"].as_bool().unwrap_or(true);
        let expected = if scan {
            "unknown"
        } else {
            truth["kind"].as_str().unwrap_or("unknown")
        };
        let lines = lines_of(pdfium, &pdf);
        let named = kinds::classify(lines.iter().map(String::as_str))
            .kind
            .code();
        let tally = tallies
            .entry(format!(
                "kind.{expected}{}",
                if scan { " (scan)" } else { "" }
            ))
            .or_default();
        tally.total += 1;
        if named == expected {
            tally.right += 1;
        }
    }
    tallies
}

/// How many documents of an archive (a copy of its database) code gives each kind. Counts only.
fn archive_kinds(database: &Path) -> BTreeMap<String, Tally> {
    let mut tallies: BTreeMap<String, Tally> = BTreeMap::new();
    let Ok(conn) =
        rusqlite::Connection::open_with_flags(database, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
    else {
        eprintln!("{} ließ sich nicht öffnen", database.display());
        return tallies;
    };
    let ids: Vec<i64> = conn
        .prepare("SELECT id FROM documents ORDER BY id")
        .and_then(|mut statement| statement.query_map([], |row| row.get(0))?.collect())
        .unwrap_or_default();
    for id in ids {
        let lines = documents::lines_of(&conn, id).unwrap_or_default();
        let kind = kinds::classify(lines.iter().map(String::as_str))
            .kind
            .code();
        let tally = tallies.entry(format!("kind.{kind}")).or_default();
        tally.total += 1;
        tally.right += 1;
    }
    tallies
}

fn report(name: &str, tallies: &BTreeMap<String, Tally>) -> Value {
    println!("{name}");
    let mut fields = serde_json::Map::new();
    for (field, tally) in tallies {
        println!("  {field:<45} {:>4} von {:>4}", tally.right, tally.total);
        fields.insert(
            field.clone(),
            json!({ "right": tally.right, "total": tally.total }),
        );
    }
    json!({ "corpus": name, "fields": fields })
}

fn write(path: &Path, result: &Value) {
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let text = serde_json::to_string_pretty(result).unwrap_or_default();
    if fs::write(path, format!("{text}\n")).is_err() {
        eprintln!("{} konnte nicht geschrieben werden", path.display());
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let value = |flag: &str| {
        args.iter()
            .position(|arg| arg == flag)
            .and_then(|index| args.get(index + 1))
            .cloned()
    };
    let corpus = PathBuf::from(value("--corpus").unwrap_or_else(|| "fixtures/synthetic".into()));
    let out = PathBuf::from(value("--out").unwrap_or_else(|| "measurements".into()));
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("pdfium");
    let Ok(pdfium) = pdf::init(&[pdf::platform_folder(&root)]) else {
        eprintln!("PDFium fehlt, bitte `node scripts/fetch-pdfium.mjs` ausführen");
        std::process::exit(1);
    };
    let reader = value("--reader").unwrap_or_else(|| "text-layer".into());
    if let Some(database) = value("--archive") {
        let mut result = report("archive", &archive_kinds(Path::new(&database)));
        result["reader"] = json!("kind");
        write(&out.join("private").join("archive-kind.json"), &result);
        return;
    }
    let run = |corpus: &Path| match reader.as_str() {
        "kind" => measure_kind(pdfium, corpus),
        _ => measure(pdfium, corpus),
    };
    let mut result = report("synthetic", &run(&corpus));
    result["reader"] = json!(reader);
    write(&out.join(format!("synthetic-{reader}.json")), &result);
    if let Some(private) = std::env::var_os("CORPUS_PRIVATE").filter(|value| !value.is_empty()) {
        let mut result = report("private", &run(Path::new(&private)));
        result["reader"] = json!(reader);
        write(
            &out.join("private").join(format!("private-{reader}.json")),
            &result,
        );
    }
}
