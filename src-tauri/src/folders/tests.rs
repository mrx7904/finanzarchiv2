use super::*;
use crate::archive::{self, Opened};
use crate::testing;

fn opened(dir: &Path) -> Arc<Mutex<Opened>> {
    Arc::new(Mutex::new(
        archive::open(&dir.join(archive::FILE_NAME), testing::t0()).unwrap(),
    ))
}

fn with<T>(archive: &Mutex<Opened>, run: impl FnOnce(&Connection) -> T) -> T {
    let guard = archive.lock().unwrap();
    let Opened::Ready(ready) = &*guard else {
        panic!("not ready")
    };
    run(ready.connection())
}

fn jobs(archive: &Mutex<Opened>) -> Vec<String> {
    with(archive, |conn| {
        conn.prepare("SELECT name FROM intake_jobs WHERE source = 'folder' ORDER BY id")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    })
}

#[test]
fn the_inbox_is_made_once_and_can_be_moved_not_removed() {
    let dir = tempfile::tempdir().unwrap();
    let archive = opened(&dir.path().join("archiv"));
    let documents = dir.path().join("Dokumente");
    let inbox = with(&archive, |conn| {
        ensure_inbox(conn, &documents, testing::t0()).unwrap()
    });
    assert_eq!(inbox, documents.join(INBOX_NAME));
    assert!(inbox.is_dir());
    std::fs::remove_dir(&inbox).unwrap();
    assert_eq!(
        with(&archive, |conn| ensure_inbox(
            conn,
            &documents,
            testing::t0()
        )
        .unwrap()),
        inbox,
        "made again, never twice"
    );
    assert!(inbox.is_dir());
    let folders = with(&archive, |conn| list(conn).unwrap());
    assert_eq!(folders.len(), 1);
    assert_eq!(
        with(&archive, |conn| remove(conn, folders[0].id)),
        Err(FolderError::Inbox)
    );
    let elsewhere = dir.path().join("Anderswo");
    with(&archive, |conn| {
        move_inbox(conn, &dir.path().join("archiv"), &elsewhere).unwrap()
    });
    let moved = with(&archive, |conn| list(conn).unwrap());
    assert_eq!(
        (moved.len(), moved[0].path.clone(), moved[0].is_inbox),
        (1, elsewhere, true)
    );
}

#[test]
fn a_folder_inside_the_archive_or_twice_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let archive_dir = dir.path().join("archiv");
    let archive = opened(&archive_dir);
    let folder = dir.path().join("Rechnungen");
    with(&archive, |conn| {
        add(conn, &archive_dir, &folder, testing::t0()).unwrap()
    });
    assert_eq!(
        with(&archive, |conn| add(
            conn,
            &archive_dir,
            &folder,
            testing::t0()
        )),
        Err(FolderError::Known)
    );
    assert_eq!(
        with(&archive, |conn| add(
            conn,
            &archive_dir,
            &archive_dir.join("Archiv"),
            testing::t0()
        )),
        Err(FolderError::InsideArchive)
    );
}

#[test]
fn downloads_still_running_and_system_files_are_not_taken() {
    for name in [
        "rechnung.pdf.crdownload",
        "a.part",
        "~$brief.docx",
        ".DS_Store",
        "Thumbs.db",
        "x.tmp",
    ] {
        assert!(!wanted(Path::new(name)), "{name}");
    }
    assert!(wanted(Path::new("Rechnung 2026.pdf")));
}

#[test]
fn a_new_file_is_taken_once_its_size_stood_still_and_never_again() {
    let dir = tempfile::tempdir().unwrap();
    let archive_dir = dir.path().join("archiv");
    let archive = opened(&archive_dir);
    let folder = dir.path().join("Eingang");
    std::fs::create_dir_all(&folder).unwrap();
    // A file that lay there while the app was closed.
    std::fs::write(folder.join("alt.pdf"), testing::pdf(&["Rechnung R-1"])).unwrap();
    with(&archive, |conn| {
        add(conn, &archive_dir, &folder, testing::t0()).unwrap()
    });
    let intake = Intake::new(Arc::clone(&archive), Box::new(|| {}));
    let watcher = Watcher::new(Arc::clone(&archive), Arc::clone(&intake), None, None);
    watcher.refresh();
    assert_eq!(watcher.tick(), 0, "the first look only notes its size");
    std::thread::sleep(SETTLE + Duration::from_millis(300));
    assert_eq!(watcher.tick(), 1, "settled");
    assert_eq!(jobs(&archive), ["alt.pdf"]);

    // The app starts again: the folder is read again, the file is known.
    let again = Watcher::new(Arc::clone(&archive), Arc::clone(&intake), None, None);
    again.refresh();
    assert_eq!(
        again.tick(),
        0,
        "a file handed in before is not handed again"
    );
    std::thread::sleep(SETTLE + Duration::from_millis(300));
    assert_eq!(again.tick(), 0);
    assert_eq!(jobs(&archive).len(), 1);
    let documents_in_folder = std::fs::read_dir(&folder).unwrap().count();
    assert_eq!(
        documents_in_folder, 1,
        "nothing in the folder is moved or deleted"
    );
}

#[test]
fn a_document_the_owner_deleted_is_not_taken_again_from_a_folder() {
    let dir = tempfile::tempdir().unwrap();
    let archive = opened(dir.path());
    let root = dir.path().join("Archiv");
    let bytes = testing::pdf(&["Rechnung R-2"]);
    with(&archive, |conn| {
        let crate::documents::Outcome::Added { id } = crate::documents::import_bytes(
            conn,
            &root,
            Some(testing::pdfium()),
            &bytes,
            "a.pdf",
            Source::Dialog,
            testing::t0(),
        ) else {
            panic!("not added")
        };
        crate::documents::delete(conn, &root, id).unwrap();
        let again = crate::documents::import_bytes(
            conn,
            &root,
            Some(testing::pdfium()),
            &bytes,
            "a.pdf",
            Source::Folder,
            testing::t0(),
        );
        assert_eq!(
            again,
            crate::documents::Outcome::Refused(crate::documents::Refusal::DeletedBefore)
        );
        let by_hand = crate::documents::import_bytes(
            conn,
            &root,
            Some(testing::pdfium()),
            &bytes,
            "a.pdf",
            Source::Dialog,
            testing::t0(),
        );
        assert!(
            matches!(by_hand, crate::documents::Outcome::Added { .. }),
            "the owner may take it in by hand"
        );
    });
}
