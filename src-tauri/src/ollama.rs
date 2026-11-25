//! The local model on Ollama (FA-19, #14), behind the trait `LocalModel` so another runtime can
//! replace it without touching the readers (knowledge/decision-stack.md).
//!
//! Ollama is reached only at `OLLAMA_HOST`, by default `127.0.0.1:11434`, the one row for the model
//! in "Outside connections" (knowledge/conventions.md). The client is built without TLS, follows no
//! redirect and uses no proxy, so it cannot reach any other host.

use std::time::Duration;

use serde_json::{Value, json};

pub const DEFAULT_HOST: &str = "127.0.0.1:11434";
/// How long a model stays loaded after the last document, so a game finds the GPU free soon.
pub const KEEP_ALIVE: &str = "1m";
/// The longest one answer may take before the job waits for the model.
pub const ANSWER_LIMIT: Duration = Duration::from_secs(180);

/// Why the model gave no answer. German, for the owner.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ModelError {
    #[error("Ollama antwortet nicht unter {0}. Ist Ollama gestartet?")]
    Unreachable(String),
    #[error("Ollama hat nicht rechtzeitig geantwortet.")]
    Timeout,
    #[error("Das Modell {0} ist in Ollama nicht installiert.")]
    NotInstalled(String),
    #[error("Die Antwort des Modells war unbrauchbar ({0}).")]
    BadAnswer(String),
    #[error("OLLAMA_HOST „{0}“ ist keine Adresse, die über http erreichbar ist.")]
    BadHost(String),
}

impl ModelError {
    /// Whether the job should wait for the model and try again, rather than go on without it.
    pub fn should_wait(&self) -> bool {
        matches!(
            self,
            Self::Unreachable(_) | Self::Timeout | Self::NotInstalled(_)
        )
    }
}

/// One question to the model: the instructions, the text, the JSON schema of the answer and, for
/// reading by image (#15), the pages as base64 PNG.
pub struct Question<'a> {
    pub system: &'a str,
    pub prompt: &'a str,
    pub schema: &'a Value,
    pub images: &'a [String],
}

pub trait LocalModel: Send + Sync {
    /// The name of the model, as stored with each value it read (`model:<name>`).
    fn name(&self) -> String;
    /// The answer of the model, a JSON text in the form of the schema.
    fn ask(&self, question: &Question) -> Result<String, ModelError>;
}

/// The base address of Ollama from `OLLAMA_HOST`: `host:port` or `http://host:port`. `0.0.0.0`, the
/// address Ollama is told to listen on, is reached on the machine itself. Another scheme, a path or
/// credentials are refused.
pub fn base_url(host: &str) -> Result<String, ModelError> {
    let refused = || ModelError::BadHost(host.to_owned());
    let trimmed = host.trim().trim_end_matches('/');
    let rest = match trimmed.split_once("://") {
        Some(("http", rest)) => rest,
        Some(_) => return Err(refused()),
        None => trimmed,
    };
    if rest.is_empty() || rest.contains(['/', '@', '?', '#']) || rest.contains(char::is_whitespace)
    {
        return Err(refused());
    }
    let with_port = if rest
        .rsplit_once(':')
        .is_some_and(|(_, port)| !port.is_empty() && port.bytes().all(|b| b.is_ascii_digit()))
        && !rest.ends_with(']')
    {
        rest.to_owned()
    } else {
        format!("{rest}:11434")
    };
    let local = with_port.replacen("0.0.0.0:", "127.0.0.1:", 1);
    Ok(format!("http://{local}"))
}

pub struct Ollama {
    base: String,
    model: String,
    agent: ureq::Agent,
}

impl Ollama {
    pub fn new(host: &str, model: &str, limit: Duration) -> Result<Self, ModelError> {
        let config = ureq::Agent::config_builder()
            .timeout_connect(Some(Duration::from_secs(2)))
            .timeout_global(Some(limit))
            .max_redirects(0)
            .proxy(None)
            .http_status_as_error(false)
            .build();
        Ok(Self {
            base: base_url(host)?,
            model: model.to_owned(),
            agent: config.into(),
        })
    }

    /// Ollama at `OLLAMA_HOST`, or at the default when it is not set.
    pub fn from_env(model: &str) -> Result<Self, ModelError> {
        let host = std::env::var("OLLAMA_HOST")
            .ok()
            .filter(|host| !host.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_HOST.to_owned());
        Self::new(&host, model, ANSWER_LIMIT)
    }

    pub fn base(&self) -> &str {
        &self.base
    }

    fn post(&self, path: &str, body: &Value) -> Result<(u16, String), ModelError> {
        let mut response = self
            .agent
            .post(format!("{}{path}", self.base))
            .send_json(body)
            .map_err(|error| self.failure(&error))?;
        let status = response.status().as_u16();
        let text = response
            .body_mut()
            .read_to_string()
            .map_err(|error| self.failure(&error))?;
        Ok((status, text))
    }

    fn failure(&self, error: &ureq::Error) -> ModelError {
        match error {
            // On Windows a closed port is refused only when the connect times out: nobody listens.
            ureq::Error::Timeout(ureq::Timeout::Connect | ureq::Timeout::Resolve) => {
                ModelError::Unreachable(self.base.clone())
            }
            ureq::Error::Timeout(_) => ModelError::Timeout,
            ureq::Error::Json(error) => ModelError::BadAnswer(error.to_string()),
            _ => ModelError::Unreachable(self.base.clone()),
        }
    }

    /// Unloads the model at once, so the GPU is free (`keep_alive` 0).
    pub fn unload(&self) -> Result<(), ModelError> {
        self.post(
            "/api/generate",
            &json!({ "model": self.model, "keep_alive": 0 }),
        )
        .map(|_| ())
    }

    /// How many bytes of the model lie in GPU memory, while it is loaded.
    pub fn loaded_bytes(&self) -> Option<u64> {
        let mut response = self
            .agent
            .get(format!("{}/api/ps", self.base))
            .call()
            .ok()?;
        let running: Value = response.body_mut().read_json().ok()?;
        running["models"].as_array()?.iter().find_map(|model| {
            (model["name"].as_str() == Some(self.model.as_str()))
                .then(|| model["size_vram"].as_u64())
                .flatten()
        })
    }
}

impl LocalModel for Ollama {
    fn name(&self) -> String {
        self.model.clone()
    }

    fn ask(&self, question: &Question) -> Result<String, ModelError> {
        let mut user = json!({ "role": "user", "content": question.prompt });
        if !question.images.is_empty() {
            user["images"] = json!(question.images);
        }
        let body = json!({
            "model": self.model,
            "messages": [{ "role": "system", "content": question.system }, user],
            "format": question.schema,
            "stream": false,
            "think": false,
            "keep_alive": KEEP_ALIVE,
            "options": { "temperature": 0, "seed": 1 },
        });
        let (status, text) = self.post("/api/chat", &body)?;
        let answer: Value = serde_json::from_str(&text)
            .map_err(|error| ModelError::BadAnswer(error.to_string()))?;
        if status == 404 {
            return Err(ModelError::NotInstalled(self.model.clone()));
        }
        if status != 200 {
            let reason = answer["error"].as_str().unwrap_or("ohne Grund").to_owned();
            return Err(ModelError::BadAnswer(format!("{status}: {reason}")));
        }
        answer["message"]["content"]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| ModelError::BadAnswer("keine Nachricht".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    fn question(schema: &Value) -> Question<'_> {
        Question {
            system: "Lies.",
            prompt: "1: Rechnungsbetrag 11,90",
            schema,
            images: &[],
        }
    }

    #[test]
    fn the_host_comes_from_ollama_host_and_nothing_else() {
        assert_eq!(
            base_url("127.0.0.1:11434").unwrap(),
            "http://127.0.0.1:11434"
        );
        assert_eq!(
            base_url("http://localhost:11434/").unwrap(),
            "http://localhost:11434"
        );
        assert_eq!(base_url("0.0.0.0").unwrap(), "http://127.0.0.1:11434");
        assert_eq!(base_url("[::1]:8080").unwrap(), "http://[::1]:8080");
        for refused in [
            "https://ollama.example:443",
            "ftp://127.0.0.1",
            "http://127.0.0.1:11434/api",
            "http://user@127.0.0.1",
            "",
            "127.0.0.1:11434?x=1",
        ] {
            assert!(base_url(refused).is_err(), "{refused:?}");
        }
    }

    #[test]
    fn a_question_goes_to_the_chat_of_the_configured_host_with_its_schema() {
        let (address, requests) = crate::testing::fake_server(crate::testing::http_response(
            "200 OK",
            "",
            r#"{"message":{"role":"assistant","content":"{\"gross\":{\"printed\":\"11,90\",\"line\":1}}"},"done":true}"#,
        ));
        let model = Ollama::new(&address, "beispiel:1b", Duration::from_secs(5)).unwrap();
        let schema = json!({ "type": "object" });
        let answer = model.ask(&question(&schema)).unwrap();
        assert!(answer.contains("11,90"));
        let request = requests.recv().unwrap();
        assert!(request.starts_with("POST /api/chat "), "{request}");
        assert!(request.to_lowercase().contains(&format!("host: {address}")));
        let body: Value = serde_json::from_str(request.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!(body["model"], "beispiel:1b");
        assert_eq!(body["format"], schema);
        assert_eq!(body["stream"], false);
        assert_eq!(body["think"], false);
        assert_eq!(body["keep_alive"], KEEP_ALIVE);
        assert_eq!(body["options"]["temperature"], 0);
    }

    #[test]
    fn a_redirect_to_another_host_is_never_followed() {
        let (elsewhere, visits) =
            crate::testing::fake_server(crate::testing::http_response("200 OK", "", "{}"));
        let (address, _) = crate::testing::fake_server(crate::testing::http_response(
            "307 Temporary Redirect",
            &format!("Location: http://{elsewhere}/api/chat\r\n"),
            "{}",
        ));
        let model = Ollama::new(&address, "beispiel:1b", Duration::from_secs(5)).unwrap();
        let schema = json!({});
        assert!(matches!(
            model.ask(&question(&schema)),
            Err(ModelError::BadAnswer(_))
        ));
        assert!(
            visits.recv_timeout(Duration::from_millis(300)).is_err(),
            "the other host was called"
        );
    }

    #[test]
    fn a_missing_ollama_a_missing_model_and_a_slow_answer_say_why() {
        let closed = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = closed.local_addr().unwrap().to_string();
        drop(closed);
        let model = Ollama::new(&address, "beispiel:1b", Duration::from_secs(5)).unwrap();
        let schema = json!({});
        let error = model.ask(&question(&schema)).unwrap_err();
        assert!(matches!(error, ModelError::Unreachable(_)), "{error:?}");
        assert!(error.should_wait());
        assert!(error.to_string().contains("Ollama"));

        let (address, _) = crate::testing::fake_server(crate::testing::http_response(
            "404 Not Found",
            "",
            r#"{"error":"model 'beispiel:1b' not found"}"#,
        ));
        let model = Ollama::new(&address, "beispiel:1b", Duration::from_secs(5)).unwrap();
        assert_eq!(
            model.ask(&question(&schema)),
            Err(ModelError::NotInstalled("beispiel:1b".into()))
        );

        let silent = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = silent.local_addr().unwrap().to_string();
        let held = std::thread::spawn(move || {
            let connection = silent.accept();
            std::thread::sleep(Duration::from_millis(1500));
            drop(connection);
        });
        let model = Ollama::new(&address, "beispiel:1b", Duration::from_millis(400)).unwrap();
        assert_eq!(model.ask(&question(&schema)), Err(ModelError::Timeout));
        held.join().unwrap();
    }
}
