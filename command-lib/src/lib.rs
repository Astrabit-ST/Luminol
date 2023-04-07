use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoStaticStr};

type Code = u16;
type Parameters = Vec<Parameter>;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct CommandDescription {
    /// The code of this command
    pub code: Code,
    /// The name of the command
    pub name: String,
    /// The description for the command
    /// Shows up when hovering over the command and in the command ui
    #[serde(default)]
    pub description: String,
    /// The type of command this is
    #[serde(default)]
    pub kind: CommandKind,
    /// Hide this in the command ui
    #[serde(default)]
    pub hidden: bool,

    /// The text used by lumi!
    #[serde(default)]
    pub lumi_text: String,

    /// A unique guid
    ///
    /// Used mainly in command-gen to prevent conflicts with egui::Id
    #[serde(skip)]
    #[serde(default = "rand::random")]
    pub guid: u64,
}

impl CommandDescription {
    /// How many total parameters does the command have?
    pub fn parameter_count(&self) -> u8 {
        match self.kind {
            CommandKind::Branch { ref parameters, .. } | CommandKind::Single(ref parameters) => {
                parameters.len() as u8
                    + parameters
                        .iter()
                        .map(Parameter::parameter_count)
                        .sum::<u8>()
            }
            CommandKind::Multi { .. } => 1,
        }
    }
}

impl Default for CommandDescription {
    fn default() -> Self {
        CommandDescription {
            code: 0,
            name: "New Command".to_string(),
            description: "".to_string(),
            kind: CommandKind::default(),
            hidden: false,
            lumi_text: "".to_string(),
            guid: rand::random(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, EnumIter, IntoStaticStr)]
pub enum CommandKind {
    /// This command is a branch
    ///
    /// The Code is used to place an empty EventCommand at the end of the branch
    Branch {
        end_code: Code,
        parameters: Parameters,
    },
    /// This command spans multiple event commands
    ///
    /// This type is reserved for multiline text commands
    Multi { code: Code, highlight: bool },
    /// This is a basic command
    Single(Parameters),
}

impl Default for CommandKind {
    fn default() -> Self {
        CommandKind::Single(vec![])
    }
}

// This is for the sake of command-gen
impl PartialEq for CommandKind {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, EnumIter, IntoStaticStr)]
pub enum Parameter {
    Selection {
        /// The index for this parameter
        #[serde(default)]
        index: Index,
        /// Parameters are stored in a Vec of (i8, Parameter)
        ///
        /// The first tuple field (the i8) denotes what value should select this parameter
        parameters: Vec<(i8, Parameter)>,

        /// Ignore, used for command-gen
        #[serde(skip)]
        #[serde(default = "rand::random")]
        guid: u64,
    },
    Group {
        /// Groups parameters together
        parameters: Parameters,

        /// Ignore, used for command-gen
        #[serde(skip)]
        #[serde(default = "rand::random")]
        guid: u64,
    },
    Single {
        /// The index for this parameter
        #[serde(default)]
        index: Index,
        /// Description of this parameter
        description: String,
        /// Parameter name
        name: String,
        /// Type of parameter
        kind: ParameterKind,

        /// Ignore, used for command-gen
        #[serde(skip)]
        #[serde(default = "rand::random")]
        guid: u64,
    },

    /// A dummy parameter used for padding
    #[default]
    Dummy,

    /// A parameter used as a label
    Label(String),
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Index {
    /// The index is assumed
    ///
    /// The algorithm for assuming indices is as follows:
    /// - Global index starts at 0
    /// - The command iterates over its parameters
    /// - If the parameter is single, it adds 1 to the global index
    /// - If a group, it calculates the indexes for its parameters and adds them to the global index
    /// - If a selection, add 1 to the global index, store the global index for each selection, and add the max value to the global index.
    ///   All selections inside a selection will start off with the same index
    Assumed(u8),
    /// The index is set by the user
    Overridden(u8),
}

impl Index {
    /// Convert this to a u8
    pub fn as_u8(self) -> u8 {
        match self {
            Self::Assumed(i) | Self::Overridden(i) => i,
        }
    }

    pub fn as_usize(self) -> usize {
        self.as_u8() as usize
    }
}

impl Default for Index {
    fn default() -> Self {
        Self::Assumed(0)
    }
}

impl PartialEq for Parameter {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl Parameter {
    pub fn parameter_count(&self) -> u8 {
        match self {
            Self::Group { ref parameters, .. } => {
                parameters.len() as u8
                    + parameters
                        .iter()
                        .map(Parameter::parameter_count)
                        .sum::<u8>()
            }
            Self::Selection { ref parameters, .. } => {
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
    /// Parameter is a switch
    Switch,
    /// Parameter is a variable
    Variable,
    /// Parameter is a self switch
    SelfSwitch,

    /// Parameter is a string
    String,

    /// Parameter is a signed integer
    #[default]
    Int,
    /// Parameter is a bool stored as an integer
    IntBool,

    /// Parameter is a choice between a set of enums
    ///
    /// The variants are a Vec of (String, i8) with the String being the variant, and the i8 being the value
    Enum { variants: Vec<(String, i8)> },
}

impl PartialEq for ParameterKind {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}
