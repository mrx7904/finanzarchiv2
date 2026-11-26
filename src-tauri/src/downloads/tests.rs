use super::*;
use crate::archive::{self, Opened};
use crate::readers::model::tests::Scripted;
use crate::testing;

fn file(dir: &Path, name: &str, bytes: &[u8]) -> std::path::PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

#[test]
fn only_what_looks_like_a_document_of_money_is_taken() {
    let dir = tempfile::tempdir().unwrap();
    let invoice = file(
        dir.path(),
        "r.pdf",
        &testing::pdf(&[
            "Rechnungsnummer R-1",
            "Rechnungsbetrag 11,90",
            "Umsatzsteuer 19 % 1,90",
        ]),
    );
    let verdict = select(Some(testing::pdfium()), None, &invoice);
    assert!(verdict.take, "{verdict:?}");
    assert_eq!(
        verdict.reason,
        "Aus den Downloads übernommen (sieht aus wie eine Rechnung)."
    );

    let ticket = file(
        dir.path(),
        "ticket.pdf",
        &testing::pdf(&["Ihre Eintrittskarte", "Block A, Reihe 3"]),
    );
    assert!(
        !select(Some(testing::pdfium()), None, &ticket).take,
        "no signal and no model"
    );
    let says_invoice = Scripted::new(vec![Ok(r#"{"kind":"invoice"}"#.into())]);
    let by_model = select(Some(testing::pdfium()), Some(&says_invoice), &ticket);
    assert!(by_model.take);
    assert!(by_model.reason.contains("laut Modell"));
    let says_other = Scripted::new(vec![Ok(r#"{"kind":"other"}"#.into())]);
    assert!(!select(Some(testing::pdfium()), Some(&says_other), &ticket).take);

    let scan = file(dir.path(), "scan.pdf", &testing::pdf_without_text());
    assert!(!select(Some(testing::pdfium()), None, &scan).take);
    let photo = file(dir.path(), "foto.png", testing::PNG);
    assert!(!select(Some(testing::pdfium()), None, &photo).take);
    let notes = file(dir.path(), "notizen.txt", b"Einkaufsliste");
    assert!(!select(Some(testing::pdfium()), None, &notes).take);
    let csv = file(
        dir.path(),
        "umsatz.csv",
        b"Umsatzanzeige;x\nIBAN;DE89 3704 0044 0532 0130 00\nBuchung;Valuta;Saldo;Betrag\n",
    );
    assert!(select(Some(testing::pdfium()), None, &csv).take);
}

#[test]
fn switching_on_leaves_what_lay_there_until_the_owner_asks() {
    let dir = tempfile::tempdir().unwrap();
    let opened = archive::open(
        &dir.path().join("archiv").join(archive::FILE_NAME),
        testing::t0(),
    )
    .unwrap();
    let Opened::Ready(ready) = opened else {
        panic!("not ready")
    };
    let conn = ready.connection();
    let downloads = dir.path().join("Downloads");
    std::fs::create_dir_all(&downloads).unwrap();
    file(
        &downloads,
        "alte-rechnung.pdf",
        &testing::pdf(&["Rechnungsbetrag 5,00"]),
    );
    file(&downloads, "urlaub.png", testing::PNG);
    assert!(!is_on(conn).unwrap());
    assert_eq!(switch_on(conn, &downloads, testing::t0()).unwrap(), 2);
    assert!(is_on(conn).unwrap());
    assert_eq!(
        switch_on(conn, &downloads, testing::t0()).unwrap(),
        0,
        "never twice"
    );
    let folder = crate::folders::list(conn)
        .unwrap()
        .into_iter()
        .find(|folder| folder.is_downloads)
        .unwrap();
    assert!(!folder.recursive, "only the top level");
    assert!(
        crate::folders::new_files(conn, &folder).unwrap().is_empty(),
        "what lay there is left"
    );
    assert_eq!(counts(conn).unwrap(), (0, 0, 2));
    assert_eq!(recheck(conn).unwrap(), 2);
    assert_eq!(
        crate::folders::new_files(conn, &folder).unwrap().len(),
        2,
        "Vorhandene prüfen"
    );
    switch_off(conn).unwrap();
    assert!(!is_on(conn).unwrap());
}

#[test]
fn a_taken_download_shows_why_with_its_document() {
    use std::sync::{Arc, Mutex};
    let dir = tempfile::tempdir().unwrap();
    let archive_dir = dir.path().join("archiv");
    let archive = Arc::new(Mutex::new(
        archive::open(&archive_dir.join(archive::FILE_NAME), testing::t0()).unwrap(),
    ));
    let downloads = dir.path().join("Downloads");
    std::fs::create_dir_all(&downloads).unwrap();
    let with = |run: &dyn Fn(&Connection)| {
        let guard = archive.lock().unwrap();
        let Opened::Ready(ready) = &*guard else {
            panic!("not ready")
        };
        run(ready.connection());
    };
    with(&|conn| {
        switch_on(conn, &downloads, testing::t0()).unwrap();
    });
    let bytes = testing::pdf(&["Rechnungsnummer R-7", "Rechnungsbetrag 11,90"]);
    file(&downloads, "r.pdf", &bytes);
    file(&downloads, "urlaub.png", testing::PNG);
    let intake = crate::intake::Intake::new(Arc::clone(&archive), Box::new(|| {}));
    let watcher = crate::folders::Watcher::new(
        Arc::clone(&archive),
        Arc::clone(&intake),
        Some(testing::pdfium()),
        None,
    );
    watcher.refresh();
    assert_eq!(watcher.tick(), 0);
    std::thread::sleep(crate::folders::SETTLE + std::time::Duration::from_millis(300));
    assert_eq!(watcher.tick(), 1, "the invoice, not the photo");
    with(&|conn| {
        assert_eq!(counts(conn).unwrap(), (1, 1, 0));
        let crate::documents::Outcome::Added { id } = crate::documents::import_bytes(
            conn,
            &archive_dir.join("Archiv"),
            Some(testing::pdfium()),
            &bytes,
            "r.pdf",
            crate::documents::Source::Folder,
            testing::t0(),
        ) else {
            panic!("not added")
        };
        conn.execute(
            "UPDATE intake_jobs SET document_id = ?1 WHERE name = 'r.pdf'",
            [id],
        )
        .unwrap();
        let listed = crate::documents::list(conn).unwrap();
        assert_eq!(
            listed[0].arrival.as_deref(),
            Some("Aus den Downloads übernommen (sieht aus wie eine Rechnung).")
        );
    });
}
