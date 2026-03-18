#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppWarning {
    pub code: &'static str,
    pub message: String,
    pub context: Vec<(String, String)>,
}

impl AppWarning {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            context: Vec::new(),
        }
    }

    pub fn with_context(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.context.push((k.into(), v.into()));
        self
    }
}
