use crate::dofapi::effect::Element;

use serde::Deserialize;

//   ____                    _____
//  / ___|__ _ _ __ __ _  __|_   _|   _ _ __   ___
// | |   / _` | '__/ _` |/ __|| || | | | '_ \ / _ \
// | |__| (_| | | | (_| | (__ | || |_| | |_) |  __/
//  \____\__,_|_|  \__,_|\___||_| \__, | .__/ \___|
//                                |___/|_|

#[derive(Debug, Deserialize, Eq, Hash, PartialEq)]
pub enum CaracKind {
    Stats(Element),
    Wisdom,
    Power,
    PA,
    PM,
    PO,
    RawDamage,
    Damage(Element),
    SpellDamage,
    PushDamage,
    Critical,
    Unknown(String),
}

//   ____
//  / ___|__ _ _ __ __ _  ___
// | |   / _` | '__/ _` |/ __|
// | |__| (_| | | | (_| | (__
//  \____\__,_|_|  \__,_|\___|
//

#[derive(Debug)]
pub struct Carac {
    pub kind:  CaracKind,
    pub value: i64,
}
