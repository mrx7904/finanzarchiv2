//! The app password of a mailbox lies in the keychain of the system (`keyring`), never in the
//! archive, a file or a log (#29).

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Der Schlüsselbund des Systems ließ sich nicht erreichen.")]
pub struct SecretError;

/// The service name of every entry of the app in the keychain, the address is the user.
pub const SERVICE: &str = "de.mrx7904.finanzarchiv2.mail";

pub trait Secrets: Send + Sync {
    fn set(&self, address: &str, password: &str) -> Result<(), SecretError>;
    /// `None` when the keychain holds no password for the address.
    fn get(&self, address: &str) -> Result<Option<String>, SecretError>;
    /// Removing a password that is not there is no fault.
    fn delete(&self, address: &str) -> Result<(), SecretError>;
}

/// The keychain of the system: Windows Credential Manager, the macOS keychain.
pub struct Keychain;

fn entry(address: &str) -> Result<keyring::Entry, SecretError> {
    keyring::Entry::new(SERVICE, address).map_err(|_| SecretError)
}

impl Secrets for Keychain {
    fn set(&self, address: &str, password: &str) -> Result<(), SecretError> {
        entry(address)?
            .set_password(password)
            .map_err(|_| SecretError)
    }

    fn get(&self, address: &str) -> Result<Option<String>, SecretError> {
        match entry(address)?.get_password() {
            Ok(password) => Ok(Some(password)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err(SecretError),
        }
    }

    fn delete(&self, address: &str) -> Result<(), SecretError> {
        match entry(address)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err(SecretError),
        }
    }
}

/// A keychain in memory, for tests.
#[cfg(test)]
#[derive(Default)]
pub struct InMemory(std::sync::Mutex<std::collections::HashMap<String, String>>);

#[cfg(test)]
impl Secrets for InMemory {
    fn set(&self, address: &str, password: &str) -> Result<(), SecretError> {
        self.0
            .lock()
            .map_err(|_| SecretError)?
            .insert(address.to_owned(), password.to_owned());
        Ok(())
    }

    fn get(&self, address: &str) -> Result<Option<String>, SecretError> {
        Ok(self
            .0
            .lock()
            .map_err(|_| SecretError)?
            .get(address)
            .cloned())
    }

    fn delete(&self, address: &str) -> Result<(), SecretError> {
        self.0.lock().map_err(|_| SecretError)?.remove(address);
        Ok(())
    }
}
