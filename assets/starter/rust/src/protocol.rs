use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AppAction {
    SetLevel { value: u8 },
    Activate,
    Deactivate,
}

impl AppAction {
    pub(crate) fn required_scope(&self) -> &'static str {
        match self {
            Self::SetLevel { .. } | Self::Activate | Self::Deactivate => "example.control",
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppState {
    pub active: bool,
    pub level: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppSnapshot {
    pub authority_generation: String,
    pub revision: u64,
    pub state: AppState,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CommandRequest {
    pub authority_generation: String,
    pub principal_id: String,
    pub grant_id: String,
    pub command_id: String,
    pub scope: String,
    pub expected_revision: Option<u64>,
    pub action: AppAction,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Applied {
    pub command_id: String,
    pub ok: bool,
    pub revision: u64,
    pub state: AppState,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Rejected {
    pub command_id: String,
    pub ok: bool,
    pub revision: u64,
    pub error: &'static str,
}

impl Rejected {
    pub(crate) fn new(command_id: &str, revision: u64, error: &'static str) -> Self {
        Self {
            command_id: command_id.chars().take(128).collect(),
            ok: false,
            revision,
            error,
        }
    }
}
