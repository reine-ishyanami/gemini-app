use std::fmt;

pub enum LanguageModel {
    Gemini1_0Pro,
    Gemini1_5Pro,
    Gemini1_5Flash,
}

impl fmt::Display for LanguageModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LanguageModel::Gemini1_0Pro => write!(f, "gemini-1.0-pro"),
            LanguageModel::Gemini1_5Pro => write!(f, "gemini-1.5-pro"),
            LanguageModel::Gemini1_5Flash => write!(f, "gemini-1.5-flash"),
        }
    }
}
