use rmxp_types::rpg;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type Code = u16;
type Parameters = HashMap<Code, ParameterKind>;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct CommandDescription {
    pub code: Code,
    pub name: String,
    pub description: String,
    pub kind: CommandKind,
    pub parameters: Parameters,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandKind {
    Branch,
    Multi { code: Code },
    String,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum ParameterKind {
    Group {},
}
