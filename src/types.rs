use std::time::Instant;

#[derive(Debug, Clone)]
pub struct VariableEntry {
    pub expression: String,
    pub value: f64,
}

pub struct YankFlash {
    pub start: usize,
    pub end: usize,
    pub expires_at: Instant,
}
