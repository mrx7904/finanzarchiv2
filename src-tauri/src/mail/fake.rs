//! A fake IMAP server for tests: it holds invented mails under the UIDs 1, 2, 3 …, records every
//! command and answers like Gmail would. It accepts writing commands too, so only the client can
//! keep them away.

use std::sync::{Arc, Mutex};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

/// Every command line the fake server received, in order.
pub type Seen = Arc<Mutex<Vec<String>>>;

pub async fn fake_server(mails: Vec<Vec<u8>>) -> (std::net::SocketAddr, Seen) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let at = listener.local_addr().unwrap();
    let seen: Seen = Arc::default();
    let record = Arc::clone(&seen);
    let mails = Arc::new(mails);
    tokio::spawn(async move {
        while let Ok((socket, _)) = listener.accept().await {
            tokio::spawn(serve(socket, Arc::clone(&record), Arc::clone(&mails)));
        }
    });
    (at, seen)
}

/// The UIDs of a set like `1,3,5:*`.
fn uids_of(set: &str, highest: u32) -> Vec<u32> {
    let mut uids = Vec::new();
    for part in set.split(',') {
        match part.split_once(':') {
            Some((from, to)) => {
                let from: u32 = from.parse().unwrap_or(1);
                let to: u32 = if to == "*" {
                    highest
                } else {
                    to.parse().unwrap_or(highest)
                };
                let (low, high) = (from.min(to), from.max(to));
                uids.extend(low..=high);
            }
            None => uids.extend(part.parse::<u32>().ok()),
        }
    }
    uids.retain(|uid| *uid >= 1 && *uid <= highest);
    uids
}

fn header_of(mail: &[u8]) -> &[u8] {
    mail.windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map_or(mail, |end| &mail[..end + 4])
}

async fn serve(socket: TcpStream, seen: Seen, mails: Arc<Vec<Vec<u8>>>) {
    let highest = u32::try_from(mails.len()).unwrap();
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
        let words: Vec<&str> = line.split(' ').collect();
        let tag = words.first().copied().unwrap_or("*");
        let mut command = words.get(1).copied().unwrap_or("").to_ascii_uppercase();
        if command == "UID" {
            command = format!(
                "UID {}",
                words.get(2).copied().unwrap_or("").to_ascii_uppercase()
            );
        }
        let mut reply: Vec<u8> = Vec::new();
        match command.as_str() {
            "LOGIN" if line.ends_with("\"app-passwort\"") => {
                reply.extend(format!("{tag} OK LOGIN completed\r\n").bytes());
            }
            "LOGIN" if line.ends_with("\"konto-passwort\"") => reply.extend(
                format!("{tag} NO [ALERT] Application-specific password required (Failure)\r\n")
                    .bytes(),
            ),
            "LOGIN" => reply.extend(
                format!("{tag} NO [AUTHENTICATIONFAILED] Invalid credentials (Failure)\r\n")
                    .bytes(),
            ),
            "LIST" => reply.extend(
                format!(
                    "* LIST (\\HasNoChildren) \"/\" \"INBOX\"\r\n* LIST (\\All \\HasNoChildren) \"/\" \"[Gmail]/Alle Nachrichten\"\r\n{tag} OK LIST completed\r\n"
                )
                .bytes(),
            ),
            "EXAMINE" => reply.extend(
                format!(
                    "* FLAGS (\\Seen)\r\n* {highest} EXISTS\r\n* 0 RECENT\r\n* OK [UIDVALIDITY 7] UIDs valid\r\n{tag} OK [READ-ONLY] selected\r\n"
                )
                .bytes(),
            ),
            "UID SEARCH" => {
                let criteria = words[3..].join(" ");
                let found: Vec<u32> = if let Some(set) = criteria.strip_prefix("UID ") {
                    let mut found = uids_of(set, highest);
                    // As RFC 9051 says: `n:*` holds the last mail even when n lies above it.
                    if found.is_empty() && highest > 0 {
                        found.push(highest);
                    }
                    found
                } else {
                    (1..=highest).collect()
                };
                let found: Vec<String> = found.iter().map(u32::to_string).collect();
                reply.extend(format!("* SEARCH {}\r\n{tag} OK SEARCH completed\r\n", found.join(" ")).bytes());
            }
            "UID FETCH" => {
                let items = words[4..].join(" ").to_ascii_uppercase();
                for uid in uids_of(words.get(3).copied().unwrap_or(""), highest) {
                    let mail = &mails[usize::try_from(uid - 1).unwrap()];
                    if items.contains("BODY.PEEK[HEADER]") {
                        let header = header_of(mail);
                        reply.extend(
                            format!(
                                "* {uid} FETCH (UID {uid} RFC822.SIZE {} BODY[HEADER] {{{}}}\r\n",
                                mail.len(),
                                header.len()
                            )
                            .bytes(),
                        );
                        reply.extend(header);
                    } else {
                        reply.extend(
                            format!("* {uid} FETCH (UID {uid} BODY[] {{{}}}\r\n", mail.len())
                                .bytes(),
                        );
                        reply.extend(mail.iter());
                    }
                    reply.extend(b")\r\n");
                }
                reply.extend(format!("{tag} OK FETCH completed\r\n").bytes());
            }
            "LOGOUT" => {
                let _ = write
                    .write_all(format!("* BYE\r\n{tag} OK LOGOUT completed\r\n").as_bytes())
                    .await;
                return;
            }
            _ => reply.extend(format!("{tag} OK done\r\n").bytes()),
        }
        if write.write_all(&reply).await.is_err() {
            return;
        }
    }
}

/// Whether a command line the server received would change the mailbox. Written apart from the
/// guard, so the test does not judge the guard by its own rule.
pub fn writes(line: &str) -> bool {
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

pub fn assert_nothing_written(seen: &Seen) {
    let seen = seen.lock().unwrap();
    assert!(!seen.is_empty(), "the server saw no command at all");
    let written: Vec<&String> = seen.iter().filter(|line| writes(line)).collect();
    assert!(
        written.is_empty(),
        "writing commands reached the server: {written:?}"
    );
}
