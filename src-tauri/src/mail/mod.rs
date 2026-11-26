//! The owner's mailboxes, read-only (FA-34, #29, knowledge/decision-mail.md). Gmail over IMAP with
//! TLS and an app password. Nothing in a mailbox ever changes, on three levels: folders are opened
//! with `EXAMINE`, bodies fetched with `BODY.PEEK`, and [`guard::ReadOnly`] holds back every writing
//! command before it reaches the socket.

#[cfg(test)]
mod fake;
pub mod guard;
pub mod reader;
pub mod round;
pub mod secrets;
pub mod text;

use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use jiff::Timestamp;
use rusqlite::{Connection, OptionalExtension, params};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};

use crate::archive::Opened;
use guard::ReadOnly;
use secrets::{SecretError, Secrets};

/// The only host the mail client reaches, in the table "Outside connections" of conventions.md.
pub const GMAIL_HOST: &str = "imap.gmail.com";
const GMAIL_PORT: u16 = 993;
/// A login, from the connection to the logout, gives up after this long.
const TIMEOUT: Duration = Duration::from_secs(30);

/// Why a mailbox could not be read, in German for the owner. Never a message of the server.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MailError {
    #[error("imap.gmail.com ist nicht erreichbar. Besteht eine Verbindung zum Internet?")]
    Unreachable,
    #[error("imap.gmail.com antwortet nicht.")]
    Timeout,
    #[error("Die sichere Verbindung zu imap.gmail.com ließ sich nicht aufbauen.")]
    Tls,
    #[error("Gmail hat die Anmeldung abgelehnt: Die Adresse oder das App-Passwort stimmt nicht.")]
    Rejected,
    #[error(
        "Gmail verlangt ein App-Passwort, nicht das Passwort des Google-Kontos. Du legst es im Google-Konto unter Sicherheit, App-Passwörter an."
    )]
    AppPasswordRequired,
    #[error("Gmail hat die Verbindung unerwartet beendet.")]
    Lost,
    #[error("Das Postfach hat etwas geantwortet, das die App nicht versteht.")]
    Protocol,
    #[error("Die App hat einen schreibenden Befehl zurückgehalten.")]
    Refused,
}

fn classify(error: async_imap::error::Error) -> MailError {
    use async_imap::error::Error;
    match error {
        Error::No(text)
            if text
                .to_ascii_lowercase()
                .contains("application-specific password") =>
        {
            MailError::AppPasswordRequired
        }
        Error::No(_) => MailError::Rejected,
        Error::Io(io) if io.kind() == std::io::ErrorKind::PermissionDenied => MailError::Refused,
        Error::Io(_) | Error::ConnectionLost => MailError::Lost,
        _ => MailError::Protocol,
    }
}

/// An address as the archive keeps it: trimmed and in lower case, `None` when it is none.
pub fn address(input: &str) -> Option<String> {
    let address = input.trim().to_lowercase();
    let (user, domain) = address.split_once('@')?;
    let fits = !user.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !domain.contains('@')
        && !address.chars().any(char::is_whitespace);
    fits.then_some(address)
}

/// Gmail shows an app password in groups of four, the spaces are not part of it.
pub fn app_password(input: &str) -> String {
    input.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Logs in over any stream, opens the inbox with `EXAMINE` and logs out. Returns how many mails
/// lie in the inbox. Every command passes [`ReadOnly`].
pub async fn check_login<S>(stream: S, address: &str, password: &str) -> Result<u32, MailError>
where
    S: AsyncRead + AsyncWrite + Unpin + Debug + Send,
{
    let mut client = async_imap::Client::new(ReadOnly::new(stream));
    match client.read_response().await {
        Ok(Some(_greeting)) => {}
        Ok(None) | Err(_) => return Err(MailError::Lost),
    }
    let mut session = client
        .login(address, password)
        .await
        .map_err(|(error, _)| classify(error))?;
    let inbox = session.examine("INBOX").await.map_err(classify)?;
    // A failed logout changes nothing: the server ends the session by itself.
    let _ = session.logout().await;
    Ok(inbox.exists)
}

fn tls_config() -> Result<ClientConfig, MailError> {
    let roots = RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    Ok(ClientConfig::builder_with_provider(Arc::new(
        tokio_rustls::rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .map_err(|_| MailError::Tls)?
    .with_root_certificates(roots)
    .with_no_client_auth())
}

/// The TLS connection to Gmail. The host is fixed here, so no other host can be reached.
pub async fn connect_gmail() -> Result<tokio_rustls::client::TlsStream<TcpStream>, MailError> {
    let tcp = TcpStream::connect((GMAIL_HOST, GMAIL_PORT))
        .await
        .map_err(|_| MailError::Unreachable)?;
    let name = ServerName::try_from(GMAIL_HOST).map_err(|_| MailError::Tls)?;
    TlsConnector::from(Arc::new(tls_config()?))
        .connect(name, tcp)
        .await
        .map_err(|_| MailError::Tls)
}

/// Logs in to Gmail and looks into the inbox, see [`check_login`].
pub async fn login_gmail(address: &str, password: &str) -> Result<u32, MailError> {
    tokio::time::timeout(TIMEOUT, async {
        let tls = connect_gmail().await?;
        check_login(tls, address, password).await
    })
    .await
    .map_err(|_| MailError::Timeout)?
}

/// A mailbox as "Postfächer" shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mailbox {
    pub id: i64,
    pub address: String,
    pub checked_at: Option<String>,
    /// Why the last login failed, in German, `None` when it worked.
    pub problem: Option<String>,
    /// When the last round through it ended without a fault.
    pub read_at: Option<String>,
}

pub fn list(conn: &Connection) -> rusqlite::Result<Vec<Mailbox>> {
    conn.prepare(
        "SELECT id, address, checked_at, problem, read_at FROM mailboxes ORDER BY address",
    )?
    .query_map([], |row| {
        Ok(Mailbox {
            id: row.get(0)?,
            address: row.get(1)?,
            checked_at: row.get(2)?,
            problem: row.get(3)?,
            read_at: row.get(4)?,
        })
    })?
    .collect()
}

/// Why a mailbox was not added or tested, in German.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MailboxError {
    #[error("Das ist keine Mailadresse.")]
    Address,
    #[error("Dieses Postfach ist schon eingetragen.")]
    Known,
    #[error("Dieses Postfach ist nicht eingetragen.")]
    NotFound,
    #[error("Das App-Passwort fehlt im Schlüsselbund. Entferne das Postfach und trage es neu ein.")]
    NoPassword,
    #[error(transparent)]
    Mail(#[from] MailError),
    #[error(transparent)]
    Secret(#[from] SecretError),
    #[error("Das Archiv ließ sich nicht lesen.")]
    Database,
}

fn with_archive<T>(
    archive: &Mutex<Opened>,
    run: impl FnOnce(&Connection) -> rusqlite::Result<T>,
) -> Result<T, MailboxError> {
    let guard = archive.lock().map_err(|_| MailboxError::Database)?;
    let Opened::Ready(ready) = &*guard else {
        return Err(MailboxError::Database);
    };
    run(ready.connection()).map_err(|_| MailboxError::Database)
}

/// Adds a mailbox once its login worked: the password goes to the keychain, the address to the
/// archive. A login that fails adds nothing.
pub async fn add(
    archive: &Mutex<Opened>,
    secrets: &dyn Secrets,
    address_input: &str,
    password_input: &str,
    now: Timestamp,
    login: impl AsyncFn(&str, &str) -> Result<u32, MailError>,
) -> Result<i64, MailboxError> {
    let address = address(address_input).ok_or(MailboxError::Address)?;
    let password = app_password(password_input);
    let known = with_archive(archive, |conn| {
        conn.query_row(
            "SELECT id FROM mailboxes WHERE address = ?1",
            [&address],
            |row| row.get::<_, i64>(0),
        )
        .optional()
    })?;
    if known.is_some() {
        return Err(MailboxError::Known);
    }
    login(&address, &password).await?;
    secrets.set(&address, &password)?;
    with_archive(archive, |conn| {
        conn.execute(
            "INSERT INTO mailboxes (address, host, added_at, checked_at) VALUES (?1, ?2, ?3, ?3)",
            params![address, GMAIL_HOST, now.to_string()],
        )?;
        Ok(conn.last_insert_rowid())
    })
}

/// Tests the login of a mailbox again and notes the outcome. Returns the problem, `None` when the
/// login worked.
pub async fn test(
    archive: &Mutex<Opened>,
    secrets: &dyn Secrets,
    id: i64,
    now: Timestamp,
    login: impl AsyncFn(&str, &str) -> Result<u32, MailError>,
) -> Result<Option<String>, MailboxError> {
    let address = with_archive(archive, |conn| {
        conn.query_row("SELECT address FROM mailboxes WHERE id = ?1", [id], |row| {
            row.get::<_, String>(0)
        })
        .optional()
    })?
    .ok_or(MailboxError::NotFound)?;
    let problem = match secrets.get(&address)? {
        None => Some(MailboxError::NoPassword.to_string()),
        Some(password) => login(&address, &password)
            .await
            .err()
            .map(|error| error.to_string()),
    };
    with_archive(archive, |conn| {
        conn.execute(
            "UPDATE mailboxes SET checked_at = ?2, problem = ?3 WHERE id = ?1",
            params![id, now.to_string(), problem],
        )
    })?;
    Ok(problem)
}

/// Removes a mailbox: its password from the keychain, its row from the archive. Documents that
/// came from it stay.
pub fn remove(archive: &Mutex<Opened>, secrets: &dyn Secrets, id: i64) -> Result<(), MailboxError> {
    let address = with_archive(archive, |conn| {
        conn.query_row("SELECT address FROM mailboxes WHERE id = ?1", [id], |row| {
            row.get::<_, String>(0)
        })
        .optional()
    })?
    .ok_or(MailboxError::NotFound)?;
    secrets.delete(&address)?;
    with_archive(archive, |conn| {
        conn.execute("DELETE FROM mailboxes WHERE id = ?1", [id])
    })?;
    Ok(())
}

#[cfg(test)]
mod tests;
