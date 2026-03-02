use std::{collections::HashMap, time::Instant};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VariableEntry {
    pub expression: String,
    pub value: f64,
}

pub struct YankFlash {
    pub start: usize,
    pub end: usize,
    pub expires_at: Instant,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct History {
    pub expression: String,
    pub result: Option<f64>,
    pub error: Option<String>,
}

impl std::fmt::Display for History {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.result, self.error.clone()) {
            (Some(result), _) => write!(f, "{} = {}", self.expression, result),
            (_, Some(error)) => write!(f, "'{}' resulted in error: {}", self.expression, error),
            (_, _) => write!(f, "{} 📈", self.expression),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    Input,
    History,
    Variables,
}

impl Focus {
    pub fn next(self) -> Self {
        match self {
            Focus::Input => Focus::History,
            Focus::History => Focus::Variables,
            Focus::Variables => Focus::Input, // wrap
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Focus::Input => Focus::Variables, // wrap
            Focus::History => Focus::Input,
            Focus::Variables => Focus::History,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppState {
    /// History of recorded messages
    pub history: Vec<History>,
    /// Variables stored in the calculator
    pub variables: HashMap<String, VariableEntry>,
    pub plot_data: Option<Vec<(f64, f64)>>,
}
