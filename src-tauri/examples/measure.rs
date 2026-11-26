//! Measures a reader on a corpus (FA-13, #09): per kind and field, how many values it got right.
//!
//!   cargo run --release --example measure -- --corpus ../fixtures/synthetic
//!
//! With `CORPUS_PRIVATE` set, the owner's documents in that folder are measured too, and their
//! result goes only to `measurements/private/`, which git ignores. A result holds counts, never a value.
//!
//! Readers: `text-layer` (a field counts as right when its printed value stands in a line of the text
//! PDFium reads, the floor for every reader of stage 2), `kind`, `code` and `e-invoice` (the XML an
//! invoice carries). `--archive <db>` counts on a copy of the owner's database, `--files <folder>`
//! counts the e-invoices among the PDFs of a copy of the owner's files. Both write to `private/`.
//! `--reader model --model <name>` asks the local model at `OLLAMA_HOST` for every field, within
//! 14 minutes, and unloads it afterwards. `--reader image --model <name> --width <px>` reads the
//! scans of the corpus as page images. `--reader checks --archive <copy>` runs the stages `read` to
//! `checked` on a writable copy of the archive and counts what is filed and why the rest waits.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use finanzarchiv2_lib::ollama::{self, LocalModel};
use finanzarchiv2_lib::pdf::{self, Pdfium};
use finanzarchiv2_lib::readers::{self, Normalized};
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

/// The lines of a PDF as the readers get them, with page and box.
fn reader_lines(pdfium: &Pdfium, pdf: &Path) -> Vec<readers::Line> {
    fs::read(pdf)
        .ok()
        .and_then(|bytes| pages::read(pdfium, &bytes).ok())
        .map(|read| {
            read.iter()
                .flat_map(|page| {
                    page.lines.iter().map(|line| readers::Line {
                        page: page.number,
                        text: line.text.clone(),
                        left: f64::from(line.left),
                        bottom: f64::from(line.bottom),
                        right: f64::from(line.right),
                        top: f64::from(line.top),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// What a reader found for a field, in the form the truth file prints it.
fn as_truth(value: &readers::Value) -> String {
    match &value.value {
        Normalized::Percent(rate) => rate.to_string(),
        _ => fold(&value.printed),
    }
}

/// The reader `code`: every field of the documents with text, right when it equals the truth as printed.
fn measure_code(pdfium: &Pdfium, corpus: &Path) -> BTreeMap<String, Tally> {
    let mut tallies: BTreeMap<String, Tally> = BTreeMap::new();
    for (pdf, truth) in documents(corpus) {
        if !truth["text_layer"].as_bool().unwrap_or(true) {
            continue;
        }
        let kind = kinds::Kind::from_code(truth["kind"].as_str().unwrap_or("unknown"));
        let input = readers::Input {
            kind,
            lines: reader_lines(pdfium, &pdf),
            e_invoice: None,
        };
        let found: Vec<readers::Value> = readers::for_kind(kind)
            .iter()
            .flat_map(|reader| reader.read(&input))
            .collect();
        let Some(fields) = truth["fields"].as_object() else {
            continue;
        };
        for (field, expected) in fields {
            let Some(expected) = expected.as_str() else {
                continue;
            };
            let tally = tallies
                .entry(format!("{}.{field}", kind.code()))
                .or_default();
            tally.total += 1;
            if found
                .iter()
                .any(|value| value.field == field && as_truth(value) == fold(expected))
            {
                tally.right += 1;
            }
        }
    }
    tallies
}

/// How many documents of each kind in an archive copy the code readers give each field. Counts only.
fn archive_code(database: &Path) -> BTreeMap<String, Tally> {
    let mut tallies: BTreeMap<String, Tally> = BTreeMap::new();
    let Ok(conn) =
        rusqlite::Connection::open_with_flags(database, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
    else {
        return tallies;
    };
    let documents: Vec<(i64, String)> = conn
        .prepare("SELECT id, kind FROM documents ORDER BY id")
        .and_then(|mut statement| {
            statement
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
                .collect()
        })
        .unwrap_or_default();
    for (id, _) in documents {
        let lines = documents::lines_of(&conn, id).unwrap_or_default();
        let kind = kinds::classify(lines.iter().map(String::as_str)).kind;
        let readers = readers::for_kind(kind);
        if readers.is_empty() {
            continue;
        }
        let input = readers::Input {
            kind,
            lines: readers::lines_of(&conn, id).unwrap_or_default(),
            e_invoice: None,
        };
        let found: Vec<readers::Value> = readers
            .iter()
            .flat_map(|reader| reader.read(&input))
            .collect();
        let documents_of_kind = tallies
            .entry(format!("{}.documents", kind.code()))
            .or_default();
        documents_of_kind.total += 1;
        documents_of_kind.right += 1;
        let fields: &[&str] = if kind == kinds::Kind::Payslip {
            &[
                "employer",
                "period",
                "brutto",
                "lohnsteuer",
                "solidaritaetszuschlag",
                "kirchensteuer",
                "krankenversicherung",
                "rentenversicherung",
                "arbeitslosenversicherung",
                "pflegeversicherung",
                "netto",
            ]
        } else {
            &[
                "sender",
                "invoice_number",
                "invoice_date",
                "net",
                "vat_rate",
                "vat",
                "gross",
                "due_date",
                "iban",
            ]
        };
        for &field in fields {
            let tally = tallies
                .entry(format!("{}.{field} found", kind.code()))
                .or_default();
            tally.total += 1;
            if found.iter().any(|value| value.field == field) {
                tally.right += 1;
            }
        }
    }
    tallies
}

/// A value of the e-invoice in the form the truth file prints it.
fn e_invoice_as_truth(value: &readers::Value) -> String {
    match &value.value {
        Normalized::Cents(cents) => cents.to_string().trim_end_matches(" €").to_owned(),
        Normalized::Date(date) => date.strftime("%d.%m.%Y").to_string(),
        Normalized::Percent(rate) => rate.to_string(),
        Normalized::Text(text) => fold(text),
    }
}

fn compact(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// The reader `e-invoice`: every field of the invoices that carry an XML, right when it equals the
/// truth. The other invoices are not counted, they have nothing for this reader.
fn measure_e_invoice(pdfium: &Pdfium, corpus: &Path) -> BTreeMap<String, Tally> {
    let mut tallies: BTreeMap<String, Tally> = BTreeMap::new();
    for (pdf, truth) in documents(corpus) {
        if !truth["e_invoice"].as_bool().unwrap_or(false) {
            continue;
        }
        let found = fs::read(&pdf)
            .ok()
            .and_then(|bytes| readers::e_invoice::attachment(pdfium, &bytes))
            .map(|(_, xml)| readers::e_invoice::read_xml(&xml).values)
            .unwrap_or_default();
        let Some(fields) = truth["fields"].as_object() else {
            continue;
        };
        for (field, expected) in fields {
            let Some(expected) = expected.as_str() else {
                continue;
            };
            let tally = tallies.entry(format!("invoice.{field}")).or_default();
            tally.total += 1;
            if found.iter().any(|value| {
                value.field == field
                    && compact(&e_invoice_as_truth(value)) == compact(&fold(expected))
            }) {
                tally.right += 1;
            }
        }
    }
    tallies
}

/// How many PDFs below a folder carry an XML, how many of those are a readable e-invoice, and how
/// often each field was found. Counts only, for a copy of the owner's files.
fn files_e_invoice(pdfium: &Pdfium, dir: &Path) -> BTreeMap<String, Tally> {
    fn pdfs(dir: &Path, found: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                pdfs(&path, found);
            } else if path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"))
            {
                found.push(path);
            }
        }
    }
    let mut paths = Vec::new();
    pdfs(dir, &mut paths);
    let mut tallies: BTreeMap<String, Tally> = BTreeMap::new();
    for path in paths {
        let attachment = fs::read(&path)
            .ok()
            .and_then(|bytes| readers::e_invoice::attachment(pdfium, &bytes));
        let with_xml = tallies.entry("pdf.with xml".into()).or_default();
        with_xml.total += 1;
        let Some((_, xml)) = attachment else { continue };
        with_xml.right += 1;
        let read = readers::e_invoice::read_xml(&xml);
        let readable = tallies.entry("xml.readable".into()).or_default();
        readable.total += 1;
        if read.problem.is_none() {
            readable.right += 1;
        }
        for field in [
            "sender",
            "invoice_number",
            "invoice_date",
            "net",
            "vat_rate",
            "vat",
            "gross",
            "due_date",
            "iban",
        ] {
            let tally = tallies.entry(format!("xml.{field} found")).or_default();
            tally.total += 1;
            if read.values.iter().any(|value| value.field == field) {
                tally.right += 1;
            }
        }
    }
    tallies
}

/// One measurement of a model must end within the GPU budget the owner gave (15 minutes).
const MODEL_BUDGET: std::time::Duration = std::time::Duration::from_secs(14 * 60);

/// What a run of the model gave beside the tallies.
#[derive(Default)]
struct ModelRun {
    tallies: BTreeMap<String, Tally>,
    documents: u32,
    seconds: f64,
    findings: u32,
    stopped: Option<String>,
}

impl ModelRun {
    /// Asks the model for every field of the kind. `None` ends the run: the budget is spent or
    /// Ollama does not answer.
    fn ask(
        &mut self,
        model: &ollama::Ollama,
        started: std::time::Instant,
        kind: kinds::Kind,
        lines: &[readers::Line],
    ) -> Option<Vec<readers::Value>> {
        if started.elapsed() > MODEL_BUDGET {
            self.stopped = Some("Zeitbudget erreicht".into());
            return None;
        }
        let asked = std::time::Instant::now();
        let read = readers::model::ask(model, kind, readers::model::fields_of(kind), lines);
        self.seconds += asked.elapsed().as_secs_f64();
        self.documents += 1;
        match read {
            Ok(read) => {
                self.findings += u32::try_from(read.findings.len()).unwrap_or(u32::MAX);
                Some(read.values)
            }
            Err(error) if error.should_wait() => {
                self.stopped = Some(error.to_string());
                None
            }
            Err(_) => {
                self.findings += 1;
                Some(Vec::new())
            }
        }
    }

    fn result(&self, name: &str, model: &ollama::Ollama, gpu: Option<u64>) -> Value {
        let mut result = report(name, &self.tallies);
        let per_document = if self.documents == 0 {
            0.0
        } else {
            self.seconds / f64::from(self.documents)
        };
        let memory = gpu.map_or_else(
            || "?".to_owned(),
            |bytes| format!("{:.1} GB", bytes as f64 / 1e9),
        );
        println!(
            "  {} Dokumente, {per_document:.1} s je Dokument, {} Befunde, GPU {memory}",
            self.documents, self.findings
        );
        result["reader"] = json!("model");
        result["model"] = json!(model.name());
        result["documents"] = json!(self.documents);
        result["seconds_per_document"] = json!((per_document * 10.0).round() / 10.0);
        result["findings"] = json!(self.findings);
        result["gpu_bytes"] = json!(gpu);
        result["stopped"] = json!(self.stopped);
        result
    }
}

/// The reader `model` on the generated corpus: every field of each document with text, right when
/// it equals the truth as printed.
fn measure_model(
    pdfium: &Pdfium,
    model: &ollama::Ollama,
    corpus: &Path,
) -> (ModelRun, Option<u64>) {
    let started = std::time::Instant::now();
    let mut run = ModelRun::default();
    let mut gpu = None;
    for (pdf, truth) in documents(corpus) {
        let kind = kinds::Kind::from_code(truth["kind"].as_str().unwrap_or("unknown"));
        if !truth["text_layer"].as_bool().unwrap_or(true)
            || readers::model::fields_of(kind).is_empty()
        {
            continue;
        }
        let Some(found) = run.ask(model, started, kind, &reader_lines(pdfium, &pdf)) else {
            break;
        };
        gpu = gpu.or_else(|| model.loaded_bytes());
        let Some(fields) = truth["fields"].as_object() else {
            continue;
        };
        for (field, expected) in fields {
            let Some(expected) = expected.as_str() else {
                continue;
            };
            let tally = run
                .tallies
                .entry(format!("{}.{field}", kind.code()))
                .or_default();
            tally.total += 1;
            if found
                .iter()
                .any(|value| value.field == field && as_truth(value) == fold(expected))
            {
                tally.right += 1;
            }
        }
    }
    (run, gpu)
}

/// The reader `model` on a copy of the owner's database: how often it finds each field, and how
/// often it agrees with code where both found one. Counts only.
fn archive_model(model: &ollama::Ollama, database: &Path) -> (ModelRun, Option<u64>) {
    let started = std::time::Instant::now();
    let mut run = ModelRun::default();
    let mut gpu = None;
    let Ok(conn) =
        rusqlite::Connection::open_with_flags(database, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
    else {
        return (run, gpu);
    };
    let ids: Vec<i64> = conn
        .prepare("SELECT id FROM documents ORDER BY id")
        .and_then(|mut statement| statement.query_map([], |row| row.get(0))?.collect())
        .unwrap_or_default();
    for id in ids {
        let texts = documents::lines_of(&conn, id).unwrap_or_default();
        let kind = kinds::classify(texts.iter().map(String::as_str)).kind;
        if readers::model::fields_of(kind).is_empty() {
            continue;
        }
        let input = readers::Input {
            kind,
            lines: readers::lines_of(&conn, id).unwrap_or_default(),
            e_invoice: None,
        };
        let by_code: Vec<readers::Value> = readers::for_kind(kind)
            .iter()
            .flat_map(|reader| reader.read(&input))
            .collect();
        let Some(by_model) = run.ask(model, started, kind, &input.lines) else {
            break;
        };
        gpu = gpu.or_else(|| model.loaded_bytes());
        for (field, _, _) in readers::model::fields_of(kind) {
            let mine = by_model.iter().find(|value| value.field == *field);
            let code = by_code.iter().find(|value| value.field == *field);
            let found = run
                .tallies
                .entry(format!("{}.{field} found", kind.code()))
                .or_default();
            found.total += 1;
            found.right += u32::from(mine.is_some());
            if let (Some(mine), Some(code)) = (mine, code) {
                let agrees = run
                    .tallies
                    .entry(format!("{}.{field} agrees with code", kind.code()))
                    .or_default();
                agrees.total += 1;
                agrees.right += u32::from(mine.value == code.value);
            }
        }
    }
    (run, gpu)
}

/// A model name as part of a file name.
fn file_part(model: &str) -> String {
    model
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

/// Runs the model on the corpus or the archive copy, writes the result and unloads the model.
fn run_model(pdfium: &Pdfium, model_name: &str, corpus: &Path, archive: Option<&str>, out: &Path) {
    let model = match ollama::Ollama::from_env(model_name) {
        Ok(model) => model,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    let (name, path, (run, gpu)) = match archive {
        Some(database) => (
            "archive",
            out.join("private")
                .join(format!("archive-model-{}.json", file_part(model_name))),
            archive_model(&model, Path::new(database)),
        ),
        None => (
            "synthetic",
            out.join(format!("synthetic-model-{}.json", file_part(model_name))),
            measure_model(pdfium, &model, corpus),
        ),
    };
    if model.unload().is_err() {
        eprintln!("Das Modell ließ sich nicht entladen, bitte ollama stop {model_name}");
    }
    let result = run.result(name, &model, gpu);
    if let Some(reason) = &run.stopped {
        eprintln!("Abgebrochen: {reason}");
    }
    write(&path, &result);
}

/// The reader `image` on the scans of the corpus (#15): the vision model names the kind and reads
/// every field of it from the page images at the given width. Within the GPU budget, unloaded after.
fn measure_image(pdfium: &Pdfium, model_name: &str, width: u32, corpus: &Path, out: &Path) {
    let model = match ollama::Ollama::from_env(model_name) {
        Ok(model) => model,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    let started = std::time::Instant::now();
    let mut run = ModelRun::default();
    let mut gpu = None;
    let mut pages = 0_u32;
    for (pdf, truth) in documents(corpus) {
        if truth["text_layer"].as_bool().unwrap_or(true) {
            continue;
        }
        if started.elapsed() > MODEL_BUDGET {
            run.stopped = Some("Zeitbudget erreicht".into());
            break;
        }
        let bytes = fs::read(&pdf).unwrap_or_default();
        let pngs = readers::image::pages_of(pdfium, &bytes, documents::FileType::Pdf, width);
        pages += u32::try_from(pngs.len()).unwrap_or(0);
        let asked = std::time::Instant::now();
        let read = readers::image::ask(&model, None, &[], &pngs);
        run.seconds += asked.elapsed().as_secs_f64();
        run.documents += 1;
        let read = match read {
            Ok(read) => read,
            Err(error) if error.should_wait() => {
                run.stopped = Some(error.to_string());
                break;
            }
            Err(_) => {
                run.findings += 1;
                readers::image::Read::default()
            }
        };
        gpu = gpu.or_else(|| model.loaded_bytes());
        run.findings += u32::try_from(read.findings.len()).unwrap_or(0);
        let expected_kind = truth["kind"].as_str().unwrap_or("unknown");
        let kind = run
            .tallies
            .entry(format!("{expected_kind}.kind"))
            .or_default();
        kind.total += 1;
        kind.right += u32::from(read.kind.map(kinds::Kind::code) == Some(expected_kind));
        let Some(fields) = truth["fields"].as_object() else {
            continue;
        };
        for (field, expected) in fields {
            let Some(expected) = expected.as_str() else {
                continue;
            };
            let tally = run
                .tallies
                .entry(format!("{expected_kind}.{field}"))
                .or_default();
            tally.total += 1;
            if read
                .values
                .iter()
                .any(|value| value.field == field && as_truth(value) == fold(expected))
            {
                tally.right += 1;
            }
        }
    }
    if model.unload().is_err() {
        eprintln!("Das Modell ließ sich nicht entladen, bitte ollama stop {model_name}");
    }
    let mut result = run.result("synthetic", &model, gpu);
    result["reader"] = json!("image");
    result["width"] = json!(width);
    result["pages"] = json!(pages);
    if let Some(reason) = &run.stopped {
        eprintln!("Abgebrochen: {reason}");
    }
    write(
        &out.join(format!(
            "synthetic-image-{}-{width}.json",
            file_part(model_name)
        )),
        &result,
    );
}

/// The checks of #16 on a copy of the owner's archive (the database with its folder `Archiv`
/// beside it): the copy is opened and migrated, every document goes through the stages `read` to
/// `checked` with the model, and how many are filed and why the others wait is counted.
fn measure_checks(pdfium: &Pdfium, model_name: &str, database: &Path, out: &Path) {
    let model = match ollama::Ollama::from_env(model_name) {
        Ok(model) => model,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    let opened = match finanzarchiv2_lib::archive::open(database, jiff::Timestamp::now()) {
        Ok(finanzarchiv2_lib::archive::Opened::Ready(ready)) => ready,
        _ => {
            eprintln!("Die Kopie des Archivs ließ sich nicht öffnen");
            std::process::exit(1);
        }
    };
    let root = documents::archive_root(database);
    let now = jiff::Timestamp::now();
    let conn = opened.connection();
    let _ = documents::classify_missing(conn);
    let _ = readers::e_invoice::extract_missing(conn, &root, pdfium);
    let _ = readers::read_outdated(conn, now);
    let _ = finanzarchiv2_lib::checks::supersede_statement_rule(conn, now);
    let _ = finanzarchiv2_lib::checks::requeue_unchecked(conn, &root, now);
    let archive = std::sync::Mutex::new(finanzarchiv2_lib::archive::Opened::Ready(opened));
    let started = std::time::Instant::now();
    let paused = std::sync::atomic::AtomicBool::new(false);
    let mut stopped = None;
    loop {
        if started.elapsed() > MODEL_BUDGET {
            stopped = Some("Zeitbudget erreicht".to_owned());
            break;
        }
        let job = archive.lock().ok().and_then(|guard| match &*guard {
            finanzarchiv2_lib::archive::Opened::Ready(ready) => {
                finanzarchiv2_lib::intake::jobs::open_jobs(ready.connection())
                    .ok()?
                    .into_iter()
                    .next()
            }
            _ => None,
        });
        let Some(job) = job else { break };
        match finanzarchiv2_lib::intake::process(
            &archive,
            Some(pdfium),
            Some(&model),
            &job,
            &paused,
            &mut |_| false,
        ) {
            Ok(finanzarchiv2_lib::intake::Stage::Model) => {
                stopped = Some("Das Modell antwortet nicht".to_owned());
                break;
            }
            Ok(_) => {}
            Err(_) => {
                stopped = Some("Die Kopie ließ sich nicht schreiben".to_owned());
                break;
            }
        }
    }
    if model.unload().is_err() {
        eprintln!("Das Modell ließ sich nicht entladen, bitte ollama stop {model_name}");
    }
    let mut tallies: BTreeMap<String, Tally> = BTreeMap::new();
    if let Ok(guard) = archive.lock()
        && let finanzarchiv2_lib::archive::Opened::Ready(ready) = &*guard
    {
        let conn = ready.connection();
        let states: Vec<(String, String)> = conn
            .prepare("SELECT kind, state FROM documents")
            .and_then(|mut statement| {
                statement
                    .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
                    .collect()
            })
            .unwrap_or_default();
        for (kind, state) in states {
            let tally = tallies.entry(format!("{kind}.filed")).or_default();
            tally.total += 1;
            tally.right += u32::from(state == "filed");
        }
        let reasons: Vec<(String, String, Option<String>, i64)> = conn
            .prepare(
                "SELECT d.kind, c.check_id, c.field, count(*) FROM field_checks c JOIN documents d ON d.id = c.document_id
                 WHERE c.passed = 0 AND c.check_id LIKE 'check:%' GROUP BY 1, 2, 3",
            )
            .and_then(|mut statement| {
                statement
                    .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))?
                    .collect()
            })
            .unwrap_or_default();
        for (kind, check, field, count) in reasons {
            let key = format!(
                "{kind}.waits for {check}{}",
                field.map(|field| format!(" {field}")).unwrap_or_default()
            );
            let tally = tallies.entry(key).or_default();
            tally.right = u32::try_from(count).unwrap_or(0);
            tally.total = tally.right;
        }
    }
    let mut result = report("archive", &tallies);
    result["reader"] = json!("checks");
    result["model"] = json!(model_name);
    result["seconds"] = json!(started.elapsed().as_secs());
    result["stopped"] = json!(stopped);
    println!("  {} s", started.elapsed().as_secs());
    if let Some(reason) = &stopped {
        eprintln!("Abgebrochen: {reason}");
    }
    write(&out.join("private").join("archive-checks.json"), &result);
}

/// The reader `statements` (#20): on the corpus, every statement of a known layout read with its
/// balances and every booking right, and one of an unknown layout read as none. On a copy of the
/// owner's database, how many statements are read, how many add up, and how many bookings.
fn measure_statements(pdfium: &Pdfium, corpus: &Path, archive: Option<&str>, out: &Path) {
    let mut tallies: BTreeMap<String, Tally> = BTreeMap::new();
    let mut count = |key: &str, right: bool| {
        let tally = tallies.entry(key.to_owned()).or_default();
        tally.total += 1;
        tally.right += u32::from(right);
    };
    let name = if let Some(database) = archive {
        let Ok(conn) = rusqlite::Connection::open_with_flags(
            database,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        ) else {
            eprintln!("Die Kopie ließ sich nicht öffnen");
            std::process::exit(1);
        };
        let ids: Vec<i64> = conn
            .prepare("SELECT id FROM documents ORDER BY id")
            .and_then(|mut statement| statement.query_map([], |row| row.get(0))?.collect())
            .unwrap_or_default();
        let mut bookings = 0_u32;
        for id in ids {
            let texts = documents::lines_of(&conn, id).unwrap_or_default();
            if kinds::classify(texts.iter().map(String::as_str)).kind != kinds::Kind::Statement {
                continue;
            }
            let read = finanzarchiv2_lib::statements::read(
                &readers::lines_of(&conn, id).unwrap_or_default(),
            );
            count("statement.read", read.is_some());
            if let Some(statement) = read {
                count(
                    "statement.adds up",
                    statement.difference() == Some(finanzarchiv2_lib::money::Cents::ZERO),
                );
                count("statement.has bookings", !statement.bookings.is_empty());
                bookings += u32::try_from(statement.bookings.len()).unwrap_or(0);
                for booking in &statement.bookings {
                    count("booking.with other side", booking.other_side.is_some());
                }
            }
        }
        println!("  {bookings} Buchungen");
        "archive"
    } else {
        for (pdf, truth) in documents(corpus) {
            if truth["kind"] != "statement" {
                continue;
            }
            let read = finanzarchiv2_lib::statements::read(&reader_lines(pdfium, &pdf));
            let known = truth["layout"].is_string();
            count("statement.layout known", read.is_some() == known);
            let Some(statement) = read else { continue };
            count(
                "statement.opening",
                statement.opening.value() == truth["cents"]["opening"].as_i64().unwrap_or(i64::MIN),
            );
            count(
                "statement.closing",
                statement.closing.value() == truth["cents"]["closing"].as_i64().unwrap_or(i64::MIN),
            );
            count(
                "statement.iban",
                statement.iban
                    == truth["fields"]["iban"]
                        .as_str()
                        .unwrap_or("")
                        .replace(' ', ""),
            );
            count(
                "statement.adds up",
                statement.difference() == Some(finanzarchiv2_lib::money::Cents::ZERO),
            );
            let expected = truth["bookings"].as_array().cloned().unwrap_or_default();
            count(
                "statement.booking count",
                expected.len() == statement.bookings.len(),
            );
            for (index, want) in expected.iter().enumerate() {
                let got = statement.bookings.get(index);
                count(
                    "booking.day",
                    got.is_some_and(|got| {
                        got.day.strftime("%d.%m.%Y").to_string()
                            == want["date"].as_str().unwrap_or("")
                    }),
                );
                count(
                    "booking.amount",
                    got.is_some_and(|got| {
                        got.amount.value() == want["cents"].as_i64().unwrap_or(i64::MIN)
                    }),
                );
                count(
                    "booking.other side",
                    got.is_some_and(|got| got.other_side.as_deref() == want["party"].as_str()),
                );
                count(
                    "booking.kind",
                    got.is_some_and(|got| Some(got.kind.as_str()) == want["kind"].as_str()),
                );
            }
        }
        "synthetic"
    };
    let mut result = report(name, &tallies);
    result["reader"] = json!("statements");
    let path = if archive.is_some() {
        out.join("private").join("archive-statements.json")
    } else {
        out.join("synthetic-statements.json")
    };
    write(&path, &result);
}

/// The reader of layout B on copies of the owner's CSV files (#21): which read as a statement, which
/// add up, how many bookings. Counts only.
fn measure_csv(folder: &Path, out: &Path) {
    let mut tallies: BTreeMap<String, Tally> = BTreeMap::new();
    let mut bookings = 0_usize;
    for entry in fs::read_dir(folder).into_iter().flatten().flatten() {
        let path = entry.path();
        if !path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("csv"))
        {
            continue;
        }
        let bytes = fs::read(&path).unwrap_or_default();
        let detected = documents::FileType::detect(&bytes) == Some(documents::FileType::Csv);
        let text = finanzarchiv2_lib::statements::layout_b::decode(&bytes);
        let lines: Vec<readers::Line> = finanzarchiv2_lib::statements::layout_b::lines_of(&text)
            .into_iter()
            .map(|line| readers::Line {
                page: 1,
                text: line.text,
                left: f64::from(line.left),
                bottom: f64::from(line.bottom),
                right: f64::from(line.right),
                top: f64::from(line.top),
            })
            .collect();
        let read = finanzarchiv2_lib::statements::read(&lines);
        for (key, right) in [
            ("csv.taken at the door", detected),
            ("csv.read", read.is_some()),
            (
                "csv.adds up",
                read.as_ref().and_then(|statement| statement.difference())
                    == Some(finanzarchiv2_lib::money::Cents::ZERO),
            ),
        ] {
            let tally = tallies.entry(key.to_owned()).or_default();
            tally.total += 1;
            tally.right += u32::from(right);
        }
        bookings += read.map_or(0, |statement| statement.bookings.len());
    }
    println!("  {bookings} Buchungen");
    let mut result = report("files", &tallies);
    result["reader"] = json!("csv");
    result["bookings"] = json!(bookings);
    write(&out.join("private").join("files-csv.json"), &result);
}

/// The balance of each account against the bank on a writable copy of the owner's archive (#22):
/// the copy is migrated and read again, then each account's findings are counted. Counts only.
fn measure_accounts(database: &Path, out: &Path) {
    let Ok(finanzarchiv2_lib::archive::Opened::Ready(ready)) =
        finanzarchiv2_lib::archive::open(database, jiff::Timestamp::now())
    else {
        eprintln!("Die Kopie des Archivs ließ sich nicht öffnen");
        std::process::exit(1);
    };
    let conn = ready.connection();
    let _ = documents::classify_missing(conn);
    let _ = readers::read_outdated(conn, jiff::Timestamp::now());
    // The balance after every booking read, whatever day it is now.
    let today = jiff::civil::date(2100, 1, 1);
    let listed = finanzarchiv2_lib::accounts::list(conn, today).unwrap_or_default();
    let mut tallies: BTreeMap<String, Tally> = BTreeMap::new();
    for account in &listed {
        let mut count = |key: &str, right: bool| {
            let tally = tallies.entry(key.to_owned()).or_default();
            tally.total += 1;
            tally.right += u32::from(right);
        };
        count("account.without finding", account.findings.is_empty());
        count(
            "account.balance equals the bank's last",
            account.balance.is_some()
                && account.bank_balance.map(|(_, cents)| cents) == account.balance,
        );
        for finding in &account.findings {
            let kind = match finding {
                finanzarchiv2_lib::accounts::Finding::Differs { .. } => "finding.differs",
                finanzarchiv2_lib::accounts::Finding::Gap { .. } => "finding.gap",
                finanzarchiv2_lib::accounts::Finding::NoOpening => "finding.no opening",
            };
            count(kind, true);
        }
    }
    let mut result = report("archive", &tallies);
    result["reader"] = json!("accounts");
    write(&out.join("private").join("archive-accounts.json"), &result);
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
    if reader == "accounts" {
        let Some(database) = value("--archive") else {
            eprintln!("--reader accounts braucht --archive <Kopie der Datenbank>");
            std::process::exit(1);
        };
        measure_accounts(Path::new(&database), &out);
        return;
    }
    if reader == "csv" {
        let Some(folder) = value("--files") else {
            eprintln!("--reader csv braucht --files <Ordner mit Kopien der CSV-Dateien>");
            std::process::exit(1);
        };
        measure_csv(Path::new(&folder), &out);
        return;
    }
    if reader == "statements" {
        measure_statements(pdfium, &corpus, value("--archive").as_deref(), &out);
        return;
    }
    if reader == "checks" {
        let model = value("--model").unwrap_or_else(|| readers::model::DEFAULT_MODEL.into());
        let Some(database) = value("--archive") else {
            eprintln!("--reader checks braucht --archive <Kopie der Datenbank>");
            std::process::exit(1);
        };
        measure_checks(pdfium, &model, Path::new(&database), &out);
        return;
    }
    if reader == "image" {
        let model = value("--model").unwrap_or_else(|| readers::model::DEFAULT_MODEL.into());
        let width = value("--width")
            .and_then(|width| width.parse().ok())
            .unwrap_or(readers::image::WIDTH);
        measure_image(pdfium, &model, width, &corpus, &out);
        return;
    }
    if reader == "model" {
        let model = value("--model").unwrap_or_else(|| readers::model::DEFAULT_MODEL.into());
        run_model(pdfium, &model, &corpus, value("--archive").as_deref(), &out);
        return;
    }
    if let Some(files) = value("--files") {
        let mut result = report("files", &files_e_invoice(pdfium, Path::new(&files)));
        result["reader"] = json!("e-invoice");
        write(&out.join("private").join("files-e-invoice.json"), &result);
        return;
    }
    if let Some(database) = value("--archive") {
        let tallies = if reader == "code" {
            archive_code(Path::new(&database))
        } else {
            archive_kinds(Path::new(&database))
        };
        let mut result = report("archive", &tallies);
        result["reader"] = json!(reader);
        write(
            &out.join("private").join(format!("archive-{reader}.json")),
            &result,
        );
        return;
    }
    let run = |corpus: &Path| match reader.as_str() {
        "kind" => measure_kind(pdfium, corpus),
        "code" => measure_code(pdfium, corpus),
        "e-invoice" => measure_e_invoice(pdfium, corpus),
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
