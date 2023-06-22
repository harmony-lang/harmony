use crate::token::SourceLocation;

#[derive(Debug, Clone)]
pub struct HarmonyError {
    pub kind: HarmonyErrorKind,
    pub message: String,
    pub hint: Option<String>,
    pub location: SourceLocation,
}

#[derive(Debug, Clone)]
pub enum HarmonyErrorKind {
    Syntax,
    Semantic,
    Type,
    CompileTime,
    Runtime,
}

impl HarmonyErrorKind {
    pub fn to_string(&self) -> String {
        match self {
            HarmonyErrorKind::Syntax => "Syntax Error".to_string(),
            HarmonyErrorKind::Semantic => "Semantic Error".to_string(),
            HarmonyErrorKind::Type => "Type Error".to_string(),
            HarmonyErrorKind::CompileTime => "Compile Time Error".to_string(),
            HarmonyErrorKind::Runtime => "Runtime Error".to_string(),
        }
    }
}

impl HarmonyError {
    pub fn new(
        kind: HarmonyErrorKind,
        message: String,
        hint: Option<String>,
        location: SourceLocation,
    ) -> HarmonyError {
        HarmonyError {
            kind,
            message,
            hint,
            location,
        }
    }

    pub fn to_string(&self) -> String {
        let mut output: String = String::new();

        output.push_str(&format!(
            "{} [{}]: {}",
            self.kind.to_string(),
            self.location.to_string(),
            self.message.to_string()
        ));

        if let Some(hint) = &self.hint {
            output.push_str(&format!("\nHint: {}", hint));
        }

        output
    }
}
