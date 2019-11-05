use std::collections::HashMap;
use std::fmt;
use std::ops::RangeInclusive;

use serde::{de, Deserialize, Deserializer};

use crate::dofapi::carac::CaracKind;
use crate::dofapi::condition::Condition;

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

    #[serde(rename = "Living object")]
    LivingObject,
}

#[derive(Deserialize, Debug)]
pub struct Equipement {
    #[serde(rename = "type")]
    pub item_type: ItemType,

    #[serde(rename = "ankamaId")]
    pub ankama_id: u64,

    #[serde(rename = "imgUrl")]
    pub img_url: String,

    pub _id:         u64,
    pub name:        String,
    pub level:       u8,
    pub url:         String,
    pub description: String,

    #[serde(default, deserialize_with = "deserialize_set_id")]
    pub set_id: Option<u64>,

    #[serde(default, deserialize_with = "deserialize_statistics")]
    pub statistics: HashMap<CaracKind, RangeInclusive<i16>>,

    #[serde(default)]
    pub conditions: Condition,
}

//  ____                      _       _ _
// |  _ \  ___  ___  ___ _ __(_) __ _| (_)_______ _ __
// | | | |/ _ \/ __|/ _ \ '__| |/ _` | | |_  / _ \ '__|
// | |_| |  __/\__ \  __/ |  | | (_| | | |/ /  __/ |
// |____/ \___||___/\___|_|  |_|\__,_|_|_/___\___|_|
//

fn deserialize_statistics<'de, D>(
    deserializer: D,
) -> Result<HashMap<CaracKind, RangeInclusive<i16>>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(StatisticsVisitor)
}

struct StatisticsVisitor;

impl<'de> de::Visitor<'de> for StatisticsVisitor {
    type Value = HashMap<CaracKind, RangeInclusive<i16>>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("A sequence of item characteristics")
    }

    fn visit_seq<D>(self, mut access: D) -> Result<Self::Value, D::Error>
    where
        D: de::SeqAccess<'de>,
    {
        let mut ret = HashMap::new();

        #[derive(Deserialize)]
        struct Bounds {
            min: Option<i16>,
            max: Option<i16>,
        }

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum ItemLine {
            Carac(HashMap<CaracKind, Bounds>),
            Emote { emote: String },
            Title { title: String },
        }

        while let Some(line) = access.next_element()? {
            let line: ItemLine = line;

            match line {
                ItemLine::Carac(carac) => {
                    let (carac_kind, bounds) = carac
                        .into_iter()
                        .next()
                        .expect("Invalid empty item line");

                    let min = bounds.min.unwrap_or(0);
                    let max = bounds.max.unwrap_or(min);

                    ret.insert(carac_kind, min..=max);
                }
                ItemLine::Emote { emote: _s } => (),
                ItemLine::Title { title: _s } => (),
            }
        }

        Ok(ret)
    }
}

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
