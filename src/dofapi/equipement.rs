use std::collections::{HashMap, HashSet};
use std::fmt;
use std::ops::Range;

use crate::dofapi::carac::CaracKind;
use serde::{de, Deserialize, Deserializer};

#[derive(Copy, Clone, Deserialize, Debug)]
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

//  _____            _                                 _
// | ____|__ _ _   _(_)_ __   ___ _ __ ___   ___ _ __ | |_
// |  _| / _` | | | | | '_ \ / _ \ '_ ` _ \ / _ \ '_ \| __|
// | |__| (_| | |_| | | |_) |  __/ | | | | |  __/ | | | |_
// |_____\__, |\__,_|_| .__/ \___|_| |_| |_|\___|_| |_|\__|
//          |_|       |_|

#[derive(Deserialize, Debug)]
pub struct Equipement {
    #[serde(rename = "type")]
    pub type_name: ItemType,

    #[serde(rename = "ankamaId")]
    pub ankama_id: u64,

    #[serde(rename = "imgUrl")]
    pub img_url: String,

    pub _id:         u64,
    pub name:        String,
    pub level:       u8,
    pub url:         String,
    pub description: String,

    #[serde(default, deserialize_with = "deserialize_statistics")]
    pub statistics: HashMap<String, Range<i16>>,
}

struct StatisticsVisitor;

impl<'de> de::Visitor<'de> for StatisticsVisitor {
    type Value = HashMap<String, Range<i16>>;

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
            Carac(HashMap<String, Bounds>),
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

                    ret.insert(
                        carac_kind,
                        Range {
                            start: bounds.min.unwrap_or(0),
                            end:   bounds
                                .max
                                .or(bounds.min)
                                .map_or(0, |x| x + 1),
                        },
                    );
                }
                ItemLine::Emote { emote: _ } => (),
                ItemLine::Title { title: _ } => (),
            }
        }

        Ok(ret)
    }
}

fn deserialize_statistics<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, Range<i16>>, D::Error>
where
    D: Deserializer<'de>,
{
    // let _: Vec<HashSet<HashMap<String, HashMap<String, i64>>>> =
    //     deserializer.deserialize_any()?;
    deserializer.deserialize_seq(StatisticsVisitor)
    // Ok(HashMap::new())
}
