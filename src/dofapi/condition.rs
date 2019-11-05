use std::boxed::Box;
use std::cmp::Ordering;
use std::convert::From;
use std::fmt;

use regex::Regex;
use serde::{de, Deserialize, Deserializer};

use crate::dofapi::carac::CaracKind;

#[derive(Debug)]
pub enum Condition {
    And(Box<Vec<Condition>>),
    Or(Box<Vec<Condition>>),
    Other(String),
    Stats(CaracKind, Ordering, i16),
    RestrictSetBonuses,
}

impl Default for Condition {
    fn default() -> Self {
        Condition::And(Box::new(Vec::new()))
    }
}

//  ____                      _       _ _
// |  _ \  ___  ___  ___ _ __(_) __ _| (_)_______ _ __
// | | | |/ _ \/ __|/ _ \ '__| |/ _` | | |_  / _ \ '__|
// | |_| |  __/\__ \  __/ |  | | (_| | | |/ /  __/ |
// |____/ \___||___/\___|_|  |_|\__,_|_|_/___\___|_|
//

struct ConditionVisitor;

impl<'de> Deserialize<'de> for Condition {
    fn deserialize<D>(deserializer: D) -> Result<Condition, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(ConditionVisitor)
    }
}

impl<'de> de::Visitor<'de> for ConditionVisitor {
    type Value = Condition;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("A condition")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(v.into())
    }
}

impl From<&str> for Condition {
    fn from(from: &str) -> Self {
        lazy_static! {
            static ref RE_CMP: Regex =
                Regex::new(r"(?P<carac>.+) (?P<sign>>|<) (?P<value>\d+)")
                    .unwrap();
        }

        if from.split(" or ").count() > 1 {
            return Condition::Or(Box::new(
                from.split(" or ").map(|term| term.into()).collect(),
            ));
        } else if from.split(" and").count() > 1 {
            return Condition::And(Box::new(
                from.split(" and ").map(|term| term.into()).collect(),
            ));
        } else if let Some(captures) = RE_CMP.captures(from) {
            let kind = captures.name("carac").unwrap().as_str().into();
            let ordering = match captures.name("sign").unwrap().as_str() {
                "<" => Ordering::Less,
                ">" => Ordering::Greater,
                _ => unreachable!(),
            };
            let value = captures.name("value").unwrap().as_str().parse();

            if let Ok(value) = value {
                return Condition::Stats(kind, ordering, value);
            }
        }

        Condition::Other(String::from(from))
    }
}
