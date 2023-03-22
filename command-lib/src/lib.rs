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
    #[serde(default)]
    pub hidden: bool,
}

impl CommandDescription {
    pub fn parameter_count(&self) -> u8 {
        self.parameters.len() as u8
            + self
                .parameters
                .iter()
                .map(Parameter::parameter_count)
                .sum::<u8>()
    }
}

impl Default for CommandDescription {
    fn default() -> Self {
        CommandDescription {
            code: 0,
            name: "New Command".to_string(),
            description: "".to_string(),
            kind: CommandKind::Single,
            parameters: vec![],
            hidden: false,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, Default, EnumIter, IntoStaticStr)]
pub enum CommandKind {
    Branch(Code),
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
    pub description: String,
    pub name: String,
    pub kind: ParameterKind,
}

impl Parameter {
    pub fn parameter_count(&self) -> u8 {
        match self.kind {
            ParameterKind::Group { ref parameters } => {
                parameters.len() as u8
                    + parameters
                        .iter()
                        .map(Parameter::parameter_count)
                        .sum::<u8>()
            }
            ParameterKind::Selection { ref parameters } => {
                parameters.len() as u8
                    + parameters
                        .iter()
                        .map(|(_, p)| p)
                        .map(Parameter::parameter_count)
                        .sum::<u8>()
            }
            _ => 0,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, EnumIter, IntoStaticStr, Default)]
pub enum ParameterKind {
    Selection {
        parameters: Vec<(i8, Parameter)>,
    },
    Group {
        parameters: Parameters,
    },
    Switch,
    Variable,
    SelfSwitch,

    String,
    StringMulti {
        highlight: bool,
    },

    Int,
    IntBool,

    Enum {
        variants: Vec<(String, i8)>,
    },

    #[default]
    Dummy,
}

impl PartialEq for ParameterKind {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}
