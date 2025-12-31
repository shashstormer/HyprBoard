use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monitor {
    pub name: String,
    pub resolution: String,
    pub position: String,
    pub scale: String,
    pub extras: Vec<String>,
    pub disabled: bool,
    pub raw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybind {
    pub bind_type: String,
    pub mods: String,
    pub key: String,
    pub dispatcher: String,
    pub params: String,
    pub raw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowRule {
    pub name: Option<String>,
    pub rule_type: String,
    pub props: Vec<(String, String)>,
    pub effects: Vec<(String, String)>,
    pub raw: String,
    pub is_block: bool,
}

impl WindowRule {
    pub fn effect_str(&self) -> String {
        self.effects
            .iter()
            .map(|(k, v)| {
                if v.is_empty() || v == "on" {
                    k.clone()
                } else {
                    format!("{} {}", k, v)
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn match_str(&self) -> String {
        self.props
            .iter()
            .map(|(k, v)| format!("{} {}", k, v))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerRule {
    pub props: Vec<(String, String)>,
    pub effects: Vec<(String, String)>,
    pub raw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecCommand {
    pub exec_type: String,
    pub command: String,
    pub raw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    pub name: String,
    pub value: String,
    pub raw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gesture {
    pub fingers: i32,
    pub direction: String,
    pub action: String,
    pub dispatcher: String,
    pub params: String,
    pub mod_key: String,
    pub scale: String,
    pub raw: String,
}
