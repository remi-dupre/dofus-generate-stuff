use std::cmp::Ordering;
use std::convert::From;
use std::fmt;

use regex::Regex;
use serde::{de, Deserialize, Deserializer};

use crate::dofapi::carac::CaracKind;

#[derive(Clone, Debug)]
pub enum ConditionAtom {
    Other(String),
    Stats(CaracKind, Ordering, i16),
    RestrictSetBonuses,
}

impl ConditionAtom {
    pub fn is_stronger_than(&self, other: &ConditionAtom) -> bool {
        use ConditionAtom::*;
        match (self, other) {
            (Stats(lkind, lord, lval), Stats(rkind, rord, rval))
                if lkind == rkind
                    && (lord == rord || *lord == Ordering::Equal) =>
            {
                lval == rval
                    || lval.cmp(rval) == *lord
                    || lval.cmp(rval) == *rord
            }
            (RestrictSetBonuses, RestrictSetBonuses) => true,
            (Other(s1), Other(s2)) => s1 == s2,
            _ => false,
        }
    }
}

/// A clause of atoms, i.e. written in the form (atom1 or atom2 ...) and
/// (atom_i or atom_j ...) ...
#[derive(Clone, Debug, Default)]
pub struct Condition(Vec<Vec<ConditionAtom>>);

impl Condition {
    pub fn new() -> Self {
        Condition(vec![])
    }

    /// Get clauses (the list of disjunction of a big `and` operator).
    pub fn clauses(&self) -> &Vec<Vec<ConditionAtom>> {
        match self {
            Condition(clauses) => clauses,
        }
    }

    /// Move into clauses (the list of disjunction of a big `and` operator).
    pub fn into_clauses(self) -> Vec<Vec<ConditionAtom>> {
        match self {
            Condition(clauses) => clauses,
        }
    }

    /// Build a clause which is true if and only if both `cond1` and `cond2`
    /// are true.
    pub fn and(cond1: Self, cond2: Self) -> Self {
        // Function to check if a clause is stronger than another. Note that
        // this is a partial order, two clauses may not be comparable.
        let clause_stronger_than = |clause1: &Vec<ConditionAtom>,
                                    clause2: &Vec<ConditionAtom>|
         -> bool {
            clause1.iter().all(|atom1| {
                clause2.iter().any(|atom2| atom1.is_stronger_than(atom2))
            })
        };

        let mut clauses = cond1.into_clauses();
        for next_clause in cond2.into_clauses() {
            // If the clause is weaker than current expression abort insertion
            let next_is_weaker = clauses.iter().any(|prev_clause| {
                clause_stronger_than(prev_clause, &next_clause)
            });
            if next_is_weaker {
                continue;
            }

            // Otherwises remove all weaker clauses from the expression and
            // insert
            let mut i = 0;
            while i < clauses.len() {
                if clause_stronger_than(&next_clause, &clauses[i]) {
                    clauses.swap_remove(i);
                } else {
                    i += 1;
                }
            }
            clauses.push(next_clause);
        }

        Condition(clauses)
    }
}

impl From<ConditionAtom> for Condition {
    fn from(atom: ConditionAtom) -> Self {
        Condition(vec![vec![atom]])
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

        let try_into_eq = |from: &str| {
            if let Some(captures) = RE_CMP.captures(from) {
                let kind = captures.name("carac").unwrap().as_str().into();
                let ordering = match captures.name("sign").unwrap().as_str() {
                    "<" => Ordering::Less,
                    ">" => Ordering::Greater,
                    _ => unreachable!(),
                };
                let value = captures.name("value").unwrap().as_str().parse();

                if let Ok(value) = value {
                    return Some(ConditionAtom::Stats(kind, ordering, value));
                }
            }
            None
        };

        Condition(
            from.split(" and ")
                .map(|clause| {
                    clause
                        .split(" or ")
                        .map(|atom| {
                            try_into_eq(atom).unwrap_or_else(|| {
                                ConditionAtom::Other(String::from(atom))
                            })
                        })
                        .collect()
                })
                .collect(),
        )
    }
}
