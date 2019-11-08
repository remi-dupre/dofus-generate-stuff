use std::collections::HashSet;
use std::fmt;

use serde::{de, Deserialize, Deserializer};

use crate::dofapi::carac::CaracLines;
use crate::dofapi::condition::{Condition, ConditionAtom};

//  _____            _                                 _
// | ____|__ _ _   _(_)_ __   ___ _ __ ___   ___ _ __ | |_
// |  _| / _` | | | | | '_ \ / _ \ '_ ` _ \ / _ \ '_ \| __|
// | |__| (_| | |_| | | |_) |  __/ | | | | |  __/ | | | |_
// |_____\__, |\__,_|_| .__/ \___|_| |_| |_|\___|_| |_|\__|
//          |_|       |_|

#[derive(Copy, Clone, Deserialize, Debug, Eq, Hash, PartialEq)]
pub enum ItemType {
    Amulet,
    Backpack,
    Belt,
    Boots,
    Cloak,
    Dofus,
    Hat,
    Ring,
    Shield,
    Trophy,
    Petsmount,
    Pet,

    #[serde(rename = "Living object")]
    LivingObject,

    #[serde(rename = "Mounts")]
    Mount,

    Axe,
    Sword,
    Staff,
    Wand,
    Bow,
    Dagger,
    Shovel,
    Hammer,
    Scythe,
    Pickaxe,
    Tool,

    #[serde(rename = "Soul stone")]
    SoulStone,
}

/// List of all kinds of weapons.
const WEAPON_TYPES: &[ItemType] = &[
    ItemType::Axe,
    ItemType::Sword,
    ItemType::Staff,
    ItemType::Wand,
    ItemType::Bow,
    ItemType::Dagger,
    ItemType::Shovel,
    ItemType::Hammer,
    ItemType::Scythe,
    ItemType::Pickaxe,
    ItemType::Tool,
    ItemType::SoulStone,
];

#[derive(Clone, Deserialize, Debug)]
pub struct Equipement {
    #[serde(rename = "type")]
    pub item_type: ItemType,

    #[serde(rename = "ankamaId")]
    pub ankama_id: u64,

    pub _id:   u64,
    pub name:  String,
    pub level: u8,
    pub url:   String,

    #[serde(
        default,
        rename = "setId",
        deserialize_with = "deserialize_set_id"
    )]
    pub set_id: Option<u64>,

    #[serde(default)]
    pub description: String,

    #[serde(rename = "imgUrl")]
    pub img_url: String,

    #[serde(default)]
    pub statistics: CaracLines,

    #[serde(default)]
    pub conditions: Condition,
}

impl Equipement {
    /// Check wether this equipement is a weapon
    pub fn is_weapon(&self) -> bool {
        WEAPON_TYPES.contains(&self.item_type)
    }
}

/// Fix trophy conditions as big trophy are not referenced with a condition to
/// use them in the website.
///
/// This will infer it by finding out if a trophy is strictly better than
/// another one in the database.
pub fn fix_all_trophy(db: &mut [Equipement]) {
    let trophy_list: Vec<_> = db
        .into_iter()
        .filter(|item| item.item_type == ItemType::Trophy)
        .map(|item| item.clone())
        .collect();

    db.iter_mut()
        .filter(|item| item.item_type == ItemType::Trophy)
        .for_each(|item| {
            // A trophy is unique if no other trophy covers all its positive
            // bonuses.
            let is_unique = !trophy_list
                .iter()
                .filter(|other| other.level == item.level)
                .any(|other| {
                    let other_lines: HashSet<_> =
                        other.statistics.as_map().keys().collect();
                    item.statistics
                        .as_map()
                        .iter()
                        .filter(|(_kind, bounds)| *bounds.start() >= 0)
                        .all(|(kind, _bounds)| other_lines.contains(kind))
                });

            // A trophy is strong if it is unique or better than another trophy
            let is_strong = is_unique
                || trophy_list.iter().any(|other| {
                    item._id != other._id
                        && item.level <= other.level
                        && item.statistics.is_stronger_than(&other.statistics)
                });

            if is_strong {
                item.conditions = ConditionAtom::RestrictSetBonuses.into();
            }
        })
}

//  ____                      _       _ _
// |  _ \  ___  ___  ___ _ __(_) __ _| (_)_______ _ __
// | | | |/ _ \/ __|/ _ \ '__| |/ _` | | |_  / _ \ '__|
// | |_| |  __/\__ \  __/ |  | | (_| | | |/ /  __/ |
// |____/ \___||___/\___|_|  |_|\__,_|_|_/___\___|_|
//

fn deserialize_set_id<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(SetIdVisitor)
}

struct SetIdVisitor;

impl<'de> de::Visitor<'de> for SetIdVisitor {
    type Value = Option<u64>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("An integer, 0 for no set")
    }

    fn visit_u64<E>(self, val: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(if val == 0 { None } else { Some(val) })
    }
}
