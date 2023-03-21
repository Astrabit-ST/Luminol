use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoStaticStr};

type Code = u16;
type Parameters = Vec<Parameter>;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct CommandDescription {
    pub code: Code,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub kind: CommandKind,
    pub parameters: Parameters,
}

impl Default for CommandDescription {
    fn default() -> Self {
        CommandDescription {
            code: 0,
            name: "New Command".to_string(),
            description: "".to_string(),
            kind: CommandKind::Single,
            parameters: vec![],
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, Default, EnumIter, IntoStaticStr)]
pub enum CommandKind {
    Branch,
    Multi(Code),
    #[default]
    Single,
}

impl PartialEq for CommandKind {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct Parameter {
    #[serde(default)]
    pub index: Option<u8>,
    pub kind: ParameterKind,
}

#[derive(Deserialize, Serialize, Clone, Debug, EnumIter, IntoStaticStr, Default)]
pub enum ParameterKind {
    Selection {
        parameters: Parameters,
    },
    Group {
        parameters: Parameters,
    },
    Switch,
    Variable,

    #[default]
    Undefined,
}

impl PartialEq for ParameterKind {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}
