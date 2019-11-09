use std::collections::HashMap;
use std::convert::{From, Into};
use std::fmt;
use std::ops::RangeInclusive;

use serde::{de, Deserialize, Deserializer};

use crate::dofapi::effect::Element;

//   ____                    _____
//  / ___|__ _ _ __ __ _  __|_   _|   _ _ __   ___
// | |   / _` | '__/ _` |/ __|| || | | | '_ \ / _ \
// | |__| (_| | | | (_| | (__ | || |_| | |_) |  __/
//  \____\__,_|_|  \__,_|\___||_| \__, | .__/ \___|
//                                |___/|_|

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum CaracKind {
    AP,
    APReduction,
    APResistance,
    Critical,
    CriticalDamage,
    CriticalResistance,
    Damage(Element),
    Dodge,
    Heals,
    Initiative,
    Lock,
    MP,
    MPReduction,
    MPResistance,
    PerMeleeDamage,
    PerMeleeResistance,
    PerRangedDamage,
    PerRangedResistance,
    PerResistance(Element),
    PerSpellDamage,
    PerWeaponDamage,
    Pods,
    Power,
    Prospecting,
    PushbackDamage,
    PushbackResistance,
    Range,
    RawDamage,
    ReflectDamage,
    Resistance(Element),
    Special(String),
    Stats(Element),
    Summons,
    TrapDamage,
    TrapPower,
    Vitality,
    Wisdom,
}

impl CaracKind {
    pub fn smithmage_weight(&self) -> Result<f64, ()> {
        use CaracKind::*;

        if let Special(_) = self {
            Err(())
        } else {
            Ok(match self {
                Vitality => 1. / 5.,
                Stats(_) => 1.,
                Initiative => 1. / 10.,
                Wisdom => 3.,
                Prospecting => 3.,
                Power => 2.,
                Resistance(_) => 2.,
                PerResistance(_) => 6.,
                PushbackResistance | CriticalResistance => 2.,
                APResistance | APReduction | MPReduction | MPResistance => 7.,
                Pods => 2.5 / 10.,
                Lock | Dodge => 4.,
                RawDamage => 20.,
                Damage(_) | CriticalDamage | PushbackDamage | TrapDamage => 5.,
                PerMeleeDamage | PerMeleeResistance | PerRangedDamage
                | PerRangedResistance | PerWeaponDamage | PerSpellDamage => {
                    15.
                }
                TrapPower => 2.,
                Heals | Critical | ReflectDamage => 10.,
                Summons => 30.,
                Range => 50.,
                MP => 90.,
                AP => 100.,
                Special(_) => unreachable!(),
            })
        }
    }
}

impl From<&str> for CaracKind {
    fn from(from: &str) -> Self {
        use CaracKind::*;
        match from {
            "Agility" => Stats(Element::Air),
            "Air Damage" => Damage(Element::Air),
            "% Air Resistance" => PerResistance(Element::Air),
            "Air Resistance" => Resistance(Element::Air),
            "AP" => AP,
            "AP Reduction" => APReduction,
            "AP Resistance" | "AP Parry" => APResistance,
            "Chance" => Stats(Element::Water),
            "% Critical" | "Critical" => Critical,
            "Critical Damage" => CriticalDamage,
            "Critical Resistance" => CriticalResistance,
            "Damage" => RawDamage,
            "Dodge" => Dodge,
            "Earth Damage" => Damage(Element::Earth),
            "% Earth Resistance" => PerResistance(Element::Earth),
            "Earth Resistance" => Resistance(Element::Earth),
            "Fire Damage" => Damage(Element::Fire),
            "% Fire Resistance" => PerResistance(Element::Fire),
            "Fire Resistance" => Resistance(Element::Fire),
            "Heals" => Heals,
            "Initiative" => Initiative,
            "Intelligence" => Stats(Element::Fire),
            "Lock" => Lock,
            "% Melee Damage" => PerMeleeDamage,
            "% Melee Resistance" => PerMeleeResistance,
            "MP" => MP,
            "MP Reduction" => MPReduction,
            "MP Resistance" | "MP Parry" => MPResistance,
            "Neutral Damage" => Damage(Element::Neutral),
            "% Neutral Resistance" => PerResistance(Element::Neutral),
            "Neutral Resistance" => Resistance(Element::Neutral),
            "Pods" => Pods,
            "Power" => Power,
            "Power (traps)" => TrapPower,
            "Prospecting" => Prospecting,
            "Pushback Damage" => PushbackDamage,
            "Pushback Resistance" => PushbackResistance,
            "% Ranged Damage" => PerRangedDamage,
            "% Ranged Resistance" => PerRangedResistance,
            "Range" => Range,
            "Reflects  damage" => ReflectDamage,
            "% Spell Damage" => PerSpellDamage,
            "Strength" => Stats(Element::Earth),
            "Summons" => Summons,
            "Trap Damage" => TrapDamage,
            "Vitality" => Vitality,
            "Water Damage" => Damage(Element::Water),
            "% Water Resistance" => PerResistance(Element::Water),
            "Water Resistance" => Resistance(Element::Water),
            "% Weapon Damage" => PerWeaponDamage,
            "Wisdom" => Wisdom,
            special => Special(String::from(special)),
        }
    }
}

impl fmt::Display for CaracKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CaracKind::Damage(element) => {
                format!("{:?} damage", element).fmt(f)
            }
            CaracKind::PerResistance(element) => {
                format!("% {:?} resistance", element).fmt(f)
            }
            CaracKind::Resistance(element) => {
                format!("{:?} resistance", element).fmt(f)
            }
            CaracKind::Special(s) => s.fmt(f),
            CaracKind::Stats(element) => (match element {
                Element::Air => "Agility",
                Element::Earth => "Force",
                Element::Fire => "Intelligence",
                Element::Water => "Chance",
                Element::Neutral => {
                    panic!("Invalid `Neutral` element for stats")
                }
            })
            .fmt(f),
            _ => format!("{:?}", self).fmt(f),
        }
    }
}

//  ____                      _       _ _
// |  _ \  ___  ___  ___ _ __(_) __ _| (_)_______
// | | | |/ _ \/ __|/ _ \ '__| |/ _` | | |_  / _ \
// | |_| |  __/\__ \  __/ |  | | (_| | | |/ /  __/
// |____/ \___||___/\___|_|  |_|\__,_|_|_/___\___|
//

struct CaracKindVisitor;

impl<'de> Deserialize<'de> for CaracKind {
    fn deserialize<D>(deserializer: D) -> Result<CaracKind, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(CaracKindVisitor)
    }
}

impl<'de> de::Visitor<'de> for CaracKindVisitor {
    type Value = CaracKind;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("A sequence of item characteristics")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(v.into())
    }
}

// Deserializer for item line

#[derive(Clone, Debug)]
pub struct CaracLines(HashMap<CaracKind, RangeInclusive<i16>>);

impl CaracLines {
    /// Use the carac lines as a map
    pub fn as_map(&self) -> &HashMap<CaracKind, RangeInclusive<i16>> {
        let Self(map) = self;
        map
    }

    /// Check if this item's statistics are greater than or equal to another
    /// item. This is essentialy usefull to fix trophy conditions.
    pub fn is_stronger_than(&self, other: &Self) -> bool {
        other.as_map().iter().all(|(kind, other_bounds)| {
            // Check if all stats of `other` are covered by this item.
            self.as_map()
                .get(kind)
                .map(|self_bounds| self_bounds.start() >= other_bounds.end())
                .unwrap_or(*other_bounds.end() <= 0)
        }) && self.as_map().iter().all(|(kind, self_bounds)| {
            // Check if all stats of this item are covered by `other`.
            // This is required since there may be some negative values in this
            // item that are not in `other`.
            other
                .as_map()
                .get(kind)
                .map(|other_bounds| self_bounds.start() >= other_bounds.end())
                .unwrap_or(*self_bounds.start() >= 0)
        })
    }
}

impl Default for CaracLines {
    fn default() -> Self {
        CaracLines(HashMap::new())
    }
}

impl From<HashMap<CaracKind, RangeInclusive<i16>>> for CaracLines {
    fn from(map: HashMap<CaracKind, RangeInclusive<i16>>) -> Self {
        Self(map)
    }
}

#[allow(clippy::implicit_hasher)]
impl From<CaracLines> for HashMap<CaracKind, RangeInclusive<i16>> {
    fn from(caracs: CaracLines) -> Self {
        let CaracLines(map) = caracs;
        map
    }
}

struct CaracLinesVisitor;

impl<'de> Deserialize<'de> for CaracLines {
    fn deserialize<D>(deserializer: D) -> Result<CaracLines, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(CaracLinesVisitor)
    }
}

impl<'de> de::Visitor<'de> for CaracLinesVisitor {
    type Value = CaracLines;

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

        Ok(CaracLines(ret))
    }
}
