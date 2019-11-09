use serde::Deserialize;

use crate::character::RawCaracsValue;
use crate::dofapi::ItemType;

/// Input request for building a stuff.
#[derive(Deserialize)]
pub struct InputRequest {
    /// Level of the character to build a stuff for.
    #[serde(default = "default_level")]
    pub level: u8,

    /// Types of items that can't be used in the output.
    #[serde(default)]
    pub banned_types: Vec<ItemType>,

    /// List of approximate expected statistics in the output.
    pub target: Vec<(RawCaracsValue, f64)>,
}

/// Default level of a character.
fn default_level() -> u8 {
    200
}
