use serde::Deserialize;
use std::ops::RangeInclusive;

//  _____ _                           _
// | ____| | ___ _ __ ___   ___ _ __ | |_
// |  _| | |/ _ \ '_ ` _ \ / _ \ '_ \| __|
// | |___| |  __/ | | | | |  __/ | | | |_
// |_____|_|\___|_| |_| |_|\___|_| |_|\__|
//

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq, Hash)]
pub enum Element {
    Earth,
    Water,
    Air,
    Fire,
    Neutral,
}

impl Element {
    /// Return the element that boosts the damages applied in this element.
    pub fn effective_stat(&self) -> Element {
        match self {
            Self::Neutral => Self::Earth,
            _ => *self,
        }
    }
}

//  ____                                   _     _
// |  _ \  __ _ _ __ ___   __ _  __ _  ___| |   (_)_ __   ___
// | | | |/ _` | '_ ` _ \ / _` |/ _` |/ _ \ |   | | '_ \ / _ \
// | |_| | (_| | | | | | | (_| | (_| |  __/ |___| | | | |  __/
// |____/ \__,_|_| |_| |_|\__,_|\__, |\___|_____|_|_| |_|\___|
//                              |___/

#[derive(Debug, Deserialize)]
pub enum Effect {
    Hit {
        element: Element,
        bounds:  RangeInclusive<u8>,

        #[serde(default)]
        lifesteal: bool,
    },
}

#[derive(Debug, Deserialize)]
pub struct SpellEffects {
    pub effect: Vec<Effect>,
    pub ranged: bool,

    #[serde(default = "default_false")]
    pub weapon: bool,

    #[serde(default = "default_spell_critical")]
    pub critical: u8,

    #[serde(default)]
    pub critical_effect: Vec<Effect>,
}

/// Default spell critical chances.
fn default_spell_critical() -> u8 {
    5
}

fn default_false() -> bool {
    false
}
