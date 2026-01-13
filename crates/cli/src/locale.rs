// Locale represents the locales accepted for delivering answers on this
// tool. That is, it's not about i18n on the strings for this application. but
// rather the different translations accepted in places like
// `Word.translations`.
pub enum Locale {
    English,
    Catalan,
}

impl Locale {
    // Returns the string representation for the locale's code.
    pub fn to_code(&self) -> &str {
        match self {
            Self::English => "en",
            Self::Catalan => "ca",
        }
    }
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::English => write!(f, "english"),
            Self::Catalan => write!(f, "català"),
        }
    }
}

/// Fetches the Locale object that is suitable for the current environment.
pub fn current_locale() -> Locale {
    let raw_locale = std::env::var("LC_ALL").unwrap_or("en".to_string());

    if raw_locale.starts_with("ca") {
        Locale::Catalan
    } else {
        Locale::English
    }
}
