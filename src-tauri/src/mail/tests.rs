use std::sync::{Arc, Mutex};

use async_imap::error::Error;
use futures_util::TryStreamExt;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

use super::*;
use crate::archive;
use crate::testing;
use secrets::InMemory;

/// Every command line the fake server received, in order.
type Seen = Arc<Mutex<Vec<String>>>;

/// A fake IMAP server on a local port that records every command and answers like Gmail would.
/// It accepts writing commands too, so only the client can keep them away.
async fn fake_server() -> (std::net::SocketAddr, Seen) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let at = listener.local_addr().unwrap();
    let seen: Seen = Arc::default();
    let record = Arc::clone(&seen);
    tokio::spawn(async move {
        while let Ok((socket, _)) = listener.accept().await {
            tokio::spawn(serve(socket, Arc::clone(&record)));
        }
    });
    (at, seen)
}

async fn serve(socket: TcpStream, seen: Seen) {
    let (read, mut write) = socket.into_split();
    let mut lines = BufReader::new(read);
    let _ = write
        .write_all(b"* OK [CAPABILITY IMAP4rev1] fake ready\r\n")
        .await;
    loop {
        let mut line = String::new();
        if lines.read_line(&mut line).await.unwrap_or(0) == 0 {
            return;
        }
        let line = line.trim_end().to_owned();
        seen.lock().unwrap().push(line.clone());
        let mut words = line.split(' ');
        let tag = words.next().unwrap_or("*").to_owned();
        let mut command = words.next().unwrap_or("").to_ascii_uppercase();
        if command == "UID" {
            command = format!("UID {}", words.next().unwrap_or("").to_ascii_uppercase());
        }
        let reply = match command.as_str() {
            "LOGIN" if line.ends_with("\"app-passwort\"") => {
                format!("{tag} OK LOGIN completed\r\n")
            }
            "LOGIN" if line.ends_with("\"konto-passwort\"") => {
                format!("{tag} NO [ALERT] Application-specific password required (Failure)\r\n")
            }
            "LOGIN" => {
                format!("{tag} NO [AUTHENTICATIONFAILED] Invalid credentials (Failure)\r\n")
            }
            "EXAMINE" => format!(
                "* FLAGS (\\Seen)\r\n* 3 EXISTS\r\n* 0 RECENT\r\n* OK [UIDVALIDITY 7] UIDs valid\r\n{tag} OK [READ-ONLY] INBOX selected\r\n"
            ),
            "UID SEARCH" => format!("* SEARCH 1 2 3\r\n{tag} OK SEARCH completed\r\n"),
            "UID FETCH" => {
                format!("* 1 FETCH (UID 1 BODY[] {{5}}\r\nhallo)\r\n{tag} OK FETCH completed\r\n")
            }
            "LOGOUT" => {
                let _ = write
                    .write_all(format!("* BYE\r\n{tag} OK LOGOUT completed\r\n").as_bytes())
                    .await;
                return;
            }
            _ => format!("{tag} OK done\r\n"),
        };
        if write.write_all(reply.as_bytes()).await.is_err() {
            return;
        }
    }
}

/// Whether a command line the server received would change the mailbox. Written apart from the
/// guard, so the test does not judge the guard by its own rule.
fn writes(line: &str) -> bool {
    let upper = line.to_ascii_uppercase();
    let words: Vec<&str> = upper.split(' ').collect();
    let command = match words.get(1) {
        Some(&"UID") => words.get(2).copied().unwrap_or(""),
        other => other.copied().unwrap_or(""),
    };
    let writing = [
        "SELECT",
        "STORE",
        "COPY",
        "MOVE",
        "EXPUNGE",
        "APPEND",
        "CREATE",
        "DELETE",
        "RENAME",
        "SUBSCRIBE",
        "UNSUBSCRIBE",
        "CLOSE",
        "UNSELECT",
        "SETACL",
        "SETMETADATA",
        "SETQUOTA",
    ];
    writing.contains(&command)
        || (command == "FETCH"
            && (upper.contains("BODY[")
                || upper.contains("BINARY[")
                || upper.contains("RFC822 ")
                || upper.contains("RFC822)")
                || upper.ends_with("RFC822")
                || upper.contains("RFC822.TEXT")))
}

fn assert_nothing_written(seen: &Seen) {
    let seen = seen.lock().unwrap();
    assert!(!seen.is_empty(), "the server saw no command at all");
    let written: Vec<&String> = seen.iter().filter(|line| writes(line)).collect();
    assert!(
        written.is_empty(),
        "writing commands reached the server: {written:?}"
    );
}

#[tokio::test]
async fn a_login_looks_into_the_inbox_and_writes_nothing() {
    let (at, seen) = fake_server().await;
    let stream = TcpStream::connect(at).await.unwrap();
    assert_eq!(
        check_login(stream, "test@example.com", "app-passwort").await,
        Ok(3)
    );
    let names: Vec<String> = seen
        .lock()
        .unwrap()
        .iter()
        .map(|line| line.split(' ').nth(1).unwrap_or("").to_owned())
        .collect();
    assert_eq!(names, ["LOGIN", "EXAMINE", "LOGOUT"]);
    assert_nothing_written(&seen);
}

#[tokio::test]
async fn a_failed_login_says_why_in_german() {
    let (at, _) = fake_server().await;
    let wrong = check_login(
        TcpStream::connect(at).await.unwrap(),
        "test@example.com",
        "falsch",
    )
    .await;
    assert_eq!(wrong, Err(MailError::Rejected));
    let account = check_login(
        TcpStream::connect(at).await.unwrap(),
        "test@example.com",
        "konto-passwort",
    )
    .await;
    assert_eq!(account, Err(MailError::AppPasswordRequired));
    assert!(
        MailError::AppPasswordRequired
            .to_string()
            .contains("App-Passwort")
    );
}

macro_rules! held_back {
    ($call:expr) => {{
        let result = $call;
        assert!(
            matches!(&result, Err(Error::Io(error)) if error.kind() == std::io::ErrorKind::PermissionDenied),
            "not held back: {}",
            stringify!($call)
        );
    }};
}

#[tokio::test]
async fn every_writing_command_is_held_back_before_the_socket() {
    let (at, seen) = fake_server().await;
    let stream = TcpStream::connect(at).await.unwrap();
    let mut client = async_imap::Client::new(ReadOnly::new(stream));
    client.read_response().await.unwrap().unwrap();
    let mut session = client
        .login("test@example.com", "app-passwort")
        .await
        .map_err(|(error, _)| error)
        .unwrap();

    held_back!(session.select("INBOX").await);
    held_back!(session.select_condstore("INBOX").await);
    held_back!(session.store("1", "+FLAGS (\\Seen)").await);
    held_back!(session.uid_store("1", "+FLAGS (\\Deleted)").await);
    held_back!(session.copy("1", "Papierkorb").await);
    held_back!(session.uid_copy("1", "Papierkorb").await);
    held_back!(session.mv("1", "Papierkorb").await);
    held_back!(session.uid_mv("1", "Papierkorb").await);
    held_back!(session.expunge().await);
    held_back!(session.uid_expunge("1").await);
    held_back!(session.create("Neu").await);
    held_back!(session.delete("INBOX").await);
    held_back!(session.rename("INBOX", "Alt").await);
    held_back!(session.subscribe("INBOX").await);
    held_back!(session.unsubscribe("INBOX").await);
    held_back!(
        session
            .append("INBOX", None, None, b"Subject: x\r\n\r\nx")
            .await
    );
    held_back!(session.close().await);
    held_back!(session.check().await);
    held_back!(session.fetch("1", "BODY[]").await);
    held_back!(session.uid_fetch("1", "RFC822").await);
    held_back!(session.uid_fetch("1", "(UID RFC822.TEXT)").await);
    held_back!(session.uid_fetch("1", "BINARY[1]").await);

    // What reads still works after all of that.
    let inbox = session.examine("INBOX").await.unwrap();
    assert_eq!(inbox.exists, 3);
    let found = session.uid_search("ALL").await.unwrap();
    assert_eq!(found.len(), 3);
    let fetched: Vec<_> = session
        .uid_fetch("1", "(UID BODY.PEEK[])")
        .await
        .unwrap()
        .try_collect()
        .await
        .unwrap();
    assert_eq!(fetched[0].body(), Some(&b"hallo"[..]));
    session.logout().await.unwrap();

    assert_nothing_written(&seen);
    let names: Vec<String> = seen
        .lock()
        .unwrap()
        .iter()
        .map(|line| line.split(' ').nth(1).unwrap_or("").to_owned())
        .collect();
    assert_eq!(
        names,
        ["LOGIN", "EXAMINE", "UID", "UID", "LOGOUT"],
        "only the reading commands reached the server"
    );
}

#[test]
fn the_guard_knows_reading_from_writing() {
    for line in [
        "a1 LOGIN \"a@example.com\" \"x\"\r\n",
        "a2 EXAMINE \"INBOX\"\r\n",
        "a3 UID SEARCH SINCE 1-Nov-2026\r\n",
        "a4 UID FETCH 1:* (UID FLAGS RFC822.SIZE BODY.PEEK[HEADER])\r\n",
        "a5 fetch 1 (rfc822.header)\r\n",
        "a6 LIST \"\" \"*\"\r\n",
        "a7 STATUS \"INBOX\" (MESSAGES)\r\n",
        "a8 NOOP\r\n",
        "a9 LOGOUT\r\n",
    ] {
        assert!(guard::reads(line.as_bytes()), "{line}");
    }
    for line in [
        "a1 SELECT INBOX\r\n",
        "a2 uid store 1 +FLAGS (\\Seen)\r\n",
        "a3 FETCH 1 BODY[TEXT]\r\n",
        "a4 FETCH 1 (FLAGS RFC822)\r\n",
        "a5 UID EXPUNGE 1\r\n",
        "a6 UID EXAMINE INBOX\r\n",
        "a7 IDLE\r\n",
        "DONE\r\n",
        "a8 AUTHENTICATE PLAIN\r\n",
        "\r\n",
    ] {
        assert!(!guard::reads(line.as_bytes()), "{line}");
    }
}

#[tokio::test]
async fn a_literal_is_data_and_the_command_after_it_is_judged_again() {
    let (near, mut far) = tokio::io::duplex(4096);
    let mut guarded = ReadOnly::new(near);
    // A literal that looks like a command is data of the command before it.
    let data = "x STORE 1 +FLAGS (\\Deleted)\r\n";
    let login = format!("a1 LOGIN {{{}}}\r\n{data} \"pw\"\r\n", data.len());
    guarded.write_all(login.as_bytes()).await.unwrap();
    let store = guarded
        .write_all(b"a2 STORE 1 +FLAGS (\\Deleted)\r\n")
        .await;
    assert_eq!(
        store.unwrap_err().kind(),
        std::io::ErrorKind::PermissionDenied
    );
    guarded.write_all(b"a3 LOGOUT\r\n").await.unwrap();
    guarded.flush().await.unwrap();
    drop(guarded);
    let mut sent = String::new();
    far.read_to_string(&mut sent).await.unwrap();
    assert_eq!(sent, format!("{login}a3 LOGOUT\r\n"));
}

#[test]
fn addresses_and_app_passwords_are_kept_plainly() {
    assert_eq!(
        address("  Test@Example.com "),
        Some("test@example.com".into())
    );
    for wrong in [
        "",
        "test",
        "@example.com",
        "test@example",
        "te st@example.com",
        "a@b@example.com",
    ] {
        assert_eq!(address(wrong), None, "{wrong}");
    }
    assert_eq!(app_password("abcd efgh ijkl mnop"), "abcdefghijklmnop");
}

fn ready_archive(dir: &std::path::Path) -> Mutex<Opened> {
    Mutex::new(archive::open(&dir.join(archive::FILE_NAME), testing::t0()).unwrap())
}

#[tokio::test]
async fn a_mailbox_is_added_only_after_its_login_worked() {
    let dir = tempfile::tempdir().unwrap();
    let archive = ready_archive(dir.path());
    let secrets = InMemory::default();
    let fails = async |_: &str, _: &str| Err(MailError::Rejected);
    assert_eq!(
        add(
            &archive,
            &secrets,
            "test@example.com",
            "falsch",
            testing::t0(),
            fails
        )
        .await,
        Err(MailboxError::Mail(MailError::Rejected))
    );
    assert_eq!(secrets.get("test@example.com"), Ok(None), "nothing stored");
    assert!(with_archive(&archive, list).unwrap().is_empty());

    let works = async |_: &str, password: &str| {
        assert_eq!(password, "abcdefghijklmnop");
        Ok(3)
    };
    let id = add(
        &archive,
        &secrets,
        " Test@Example.com",
        "abcd efgh ijkl mnop",
        testing::t0(),
        works,
    )
    .await
    .unwrap();
    assert_eq!(
        secrets.get("test@example.com"),
        Ok(Some("abcdefghijklmnop".into()))
    );
    let listed = with_archive(&archive, list).unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].address, "test@example.com");
    assert_eq!(
        add(
            &archive,
            &secrets,
            "test@example.com",
            "x",
            testing::t0(),
            works
        )
        .await,
        Err(MailboxError::Known)
    );
    assert_eq!(
        add(
            &archive,
            &secrets,
            "keine adresse",
            "x",
            testing::t0(),
            works
        )
        .await,
        Err(MailboxError::Address)
    );

    let rejected = async |_: &str, _: &str| Err(MailError::Rejected);
    let problem = test(&archive, &secrets, id, testing::t0(), rejected)
        .await
        .unwrap();
    assert_eq!(problem, Some(MailError::Rejected.to_string()));
    assert_eq!(with_archive(&archive, list).unwrap()[0].problem, problem);
    assert_eq!(
        test(&archive, &secrets, id, testing::t0(), works).await,
        Ok(None)
    );

    remove(&archive, &secrets, id).unwrap();
    assert_eq!(secrets.get("test@example.com"), Ok(None));
    assert!(with_archive(&archive, list).unwrap().is_empty());
}

#[tokio::test]
async fn a_password_missing_from_the_keychain_is_named() {
    let dir = tempfile::tempdir().unwrap();
    let archive = ready_archive(dir.path());
    let secrets = InMemory::default();
    let works = async |_: &str, _: &str| Ok(1);
    let id = add(
        &archive,
        &secrets,
        "test@example.com",
        "x",
        testing::t0(),
        works,
    )
    .await
    .unwrap();
    secrets.delete("test@example.com").unwrap();
    assert_eq!(
        test(&archive, &secrets, id, testing::t0(), works).await,
        Ok(Some(MailboxError::NoPassword.to_string()))
    );
}

#[test]
fn the_password_never_reaches_the_archive_file() {
    let dir = tempfile::tempdir().unwrap();
    let archive = ready_archive(dir.path());
    let secrets = InMemory::default();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let works = async |_: &str, _: &str| Ok(1);
    runtime
        .block_on(add(
            &archive,
            &secrets,
            "test@example.com",
            "geheimes-app-passwort",
            testing::t0(),
            works,
        ))
        .unwrap();
    drop(archive);
    for entry in std::fs::read_dir(dir.path()).unwrap() {
        let bytes = std::fs::read(entry.unwrap().path()).unwrap_or_default();
        assert!(
            !bytes
                .windows(b"geheimes-app-passwort".len())
                .any(|window| window == b"geheimes-app-passwort"),
            "the password is in a file of the archive"
        );
    }
}
