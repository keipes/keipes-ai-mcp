use std::sync::OnceLock;

struct EnvVar {
    key: &'static str,
    default: &'static str,
    options: &'static [&'static str],
    required: &'static bool,
}

static DOTENV_INITIALIZED: OnceLock<()> = OnceLock::new();

pub fn use_tokio_console() -> bool {
    let use_tokio_console = EnvVar {
        key: "TOKIO_CONSOLE".into(),
        default: "false".into(),
        options: &["true", "false"],
        required: &false,
    }
    .equals("true");
    println!("use_tokio_console: {}", use_tokio_console);
    use_tokio_console
}

pub fn tls_cert_dir() -> String {
    EnvVar {
        key: "TLS_CERT".into(),
        default: "".into(),
        options: &[],
        required: &true,
    }
    .retrieve()
    .unwrap()
}

pub fn ensure_dotenv() {
    DOTENV_INITIALIZED.get_or_init(|| match dotenvy::dotenv() {
        Ok(_) => {
            tracing::info!("Environment variables loaded successfully");
        }
        Err(e) => {
            let cwd = std::env::current_dir().unwrap();
            tracing::warn!(
                "Failed to load environment variables. Working directory: {}, {}",
                cwd.display(),
                e
            );
        }
    });
}

impl EnvVar {
    fn equals(&self, compare: &str) -> bool {
        ensure_dotenv();
        match self.retrieve() {
            Ok(v) if v == compare => true,
            _ => false,
        }
    }

    pub fn retrieve(&self) -> Result<String, String> {
        ensure_dotenv();
        std::env::var(self.key)
            // fallback to default if allowed
            .or_else(|e| {
                if !*self.required {
                    println!(
                        "Environment variable {} not set, using default: {}",
                        self.key, self.default
                    );
                    Ok(self.default.to_string())
                } else {
                    println!("Required environment variable {} is not set", self.key);
                    Err(format!(
                        "Failed to retrieve environment variable {}: {}",
                        self.key, e
                    ))
                }
            })
            // check validity of selected value
            .and_then(|v| {
                if self.valid_option(&v) {
                    Ok(v)
                } else {
                    if !*self.required && self.valid_option(self.default) {
                        println!(
                            "Environment variable {} is invalid, using default: {}",
                            self.key, self.default
                        );
                        Ok(self.default.to_string())
                    } else {
                        println!("Invalid value '{}' for {}", v, self.key);
                        Err(format!("Invalid value '{}' for {}.", v, self.key))
                    }
                }
            })
    }

    fn valid_option(&self, value: &str) -> bool {
        self.options.is_empty() || self.options.contains(&value)
    }
}
