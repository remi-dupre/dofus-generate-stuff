use std::collections::HashMap;
use std::convert::TryInto;

use serde::Deserialize;

use crate::dofapi::{
    CaracKind, Condition, ConditionAtom, Effect, Element, Equipement,
    ItemType, Set, SpellEffects,
};

#[derive(Clone, Debug)]
pub struct ItemSlot<'a, 'i> {
    allowed: &'a [ItemType],
    item:    Option<&'i Equipement>,
}

impl<'a, 'i> ItemSlot<'a, 'i> {
    pub fn new(allowed: &'a [ItemType]) -> Self {
        ItemSlot {
            allowed,
            item: None,
        }
    }

    pub fn equip(&mut self, item: &'i Equipement) {
        if !self.allowed.contains(&item.item_type) {
            panic!("Trying to equip incorrect item type");
        }
        self.item = Some(item);
    }

    pub fn get_allowed(&self) -> &'a [ItemType] {
        &self.allowed
    }

    pub fn get_item(&self) -> Option<&'i Equipement> {
        self.item
    }
}

//   ____ _                          _
//  / ___| |__   __ _ _ __ __ _  ___| |_ ___ _ __
// | |   | '_ \ / _` | '__/ _` |/ __| __/ _ \ '__|
// | |___| | | | (_| | | | (_| | (__| ||  __/ |
//  \____|_| |_|\__,_|_|  \__,_|\___|\__\___|_|
//

#[derive(Debug, Eq, PartialEq)]
pub enum CharacterError<'c> {
    NotEnoughPoints,
    NotEnoughCaracs(&'c CaracKind),
}

#[derive(Clone, Debug)]
pub struct Character<'i> {
    pub item_slots: Vec<ItemSlot<'static, 'i>>,
    pub base_stats: HashMap<&'i CaracKind, u16>,
    pub unspent:    u16,
    // Contextual attributes
    sets: &'i HashMap<u64, Set>,
}

impl<'i> Character<'i> {
    pub fn new(level: u8, sets: &'i HashMap<u64, Set>) -> Self {
        Character {
            item_slots: vec![
                ItemSlot::new(&[ItemType::Hat]),
                ItemSlot::new(&[ItemType::Cloak, ItemType::Backpack]),
                ItemSlot::new(&[ItemType::Amulet]),
                ItemSlot::new(&[ItemType::Ring]),
                ItemSlot::new(&[ItemType::Ring]),
                ItemSlot::new(&[ItemType::Belt]),
                ItemSlot::new(&[ItemType::Boots]),
                ItemSlot::new(&[
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
                ]),
                ItemSlot::new(&[
                    ItemType::Mount,
                    ItemType::Pet,
                    ItemType::Petsmount,
                ]),
                ItemSlot::new(&[ItemType::Shield]),
                ItemSlot::new(&[ItemType::Dofus, ItemType::Trophy]),
                ItemSlot::new(&[ItemType::Dofus, ItemType::Trophy]),
                ItemSlot::new(&[ItemType::Dofus, ItemType::Trophy]),
                ItemSlot::new(&[ItemType::Dofus, ItemType::Trophy]),
                ItemSlot::new(&[ItemType::Dofus, ItemType::Trophy]),
                ItemSlot::new(&[ItemType::Dofus, ItemType::Trophy]),
            ],
            base_stats: HashMap::new(),
            unspent: 5 * (u16::from(level) - 1),
            sets,
        }
    }

    /// Iterator over items currently equiped.
    pub fn iter_items(&self) -> impl Iterator<Item = &Equipement> {
        self.item_slots.iter().filter_map(|slot| slot.item)
    }

    pub fn get_caracs(&self) -> RawCaracs {
        let items_vals = self
            .iter_items()
            .map(|item| {
                item.statistics.as_map().iter().map(|(kind, bounds)| {
                    (kind, *std::cmp::max(bounds.start(), bounds.end()))
                })
            })
            .flatten();

        let sets_vals = self
            .iter_set_synergies()
            .filter_map(|(set_id, count)| {
                Some(
                    self.sets.get(&set_id)?.bonus.get(&count)?.as_map().iter(),
                )
            })
            .flatten()
            .map(|(kind, bounds)| {
                (kind, *std::cmp::max(bounds.start(), bounds.end()))
            });

        let base_vals = self.base_stats.iter().map(|(&kind, val)| {
            let val: i16 =
                val.clone().try_into().expect("Base statistic overflow");
            (kind, val)
        });

        let mut ret = HashMap::new();
        for (kind, val) in base_vals.chain(items_vals).chain(sets_vals) {
            ret.entry(kind).and_modify(|x| *x += val).or_insert(val);
        }
        RawCaracs(ret)
    }

    pub fn iter_set_synergies(&self) -> impl Iterator<Item = (u64, u8)> {
        let mut synergies = HashMap::new();

        for item in self.iter_items() {
            if let Some(set_id) = item.set_id {
                synergies
                    .entry(set_id)
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }
        }

        synergies.into_iter()
    }

    //  ____                    ____
    // | __ )  __ _ ___  ___   / ___|__ _ _ __ __ _  ___ ___
    // |  _ \ / _` / __|/ _ \ | |   / _` | '__/ _` |/ __/ __|
    // | |_) | (_| \__ \  __/ | |__| (_| | | | (_| | (__\__ \
    // |____/ \__,_|___/\___|  \____\__,_|_|  \__,_|\___|___/
    //

    /// Compute the number of points to spend to reach a value for an initially
    /// zero characteristic.
    ///
    /// `val` must be positive.
    ///
    /// # Examples
    ///
    /// ```
    /// use dofus_stuff::character::*;
    /// use dofus_stuff::dofapi::{CaracKind::*, Element::*};
    ///
    /// assert_eq!(Character::carac_cost_from_zero(&Wisdom, 100), 300);
    /// assert_eq!(Character::carac_cost_from_zero(&Stats(Air), 100), 100);
    /// assert_eq!(Character::carac_cost_from_zero(&Stats(Air), 150), 200);
    /// ```
    pub fn carac_cost_from_zero(kind: &CaracKind, val: u16) -> u16 {
        match kind {
            CaracKind::Vitality => val,
            CaracKind::Wisdom => 3 * val,
            CaracKind::Stats(_) => {
                // cost(q * 100 + r)
                //   = 100 * (1 + 2 + ... + q) + r * (q + 1)
                //   = 100 * (q * (q + 1) / 2) + r * (q + 1)
                let q = val / 100;
                let r = val % 100;
                (q + 1) * (50 * q + r)
            }
            other => {
                panic!(format!("Impossible to spend points for `{:?}`", other))
            }
        }
    }

    /// Compute the number of points to spend to increase a characteristic.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// use dofus_stuff::character::*;
    /// use dofus_stuff::dofapi::{CaracKind::*, Element::*};
    ///
    /// let db_sets = HashMap::new();
    /// let mut character = Character::new(200, &db_sets);
    ///
    /// assert_eq!(character.carac_spend_cost(&Stats(Air), 100), 100);
    /// assert_eq!(character.carac_spend_cost(&Stats(Air), 150), 200);
    ///
    /// character.carac_spend(&Stats(Air), 100).unwrap();
    /// assert_eq!(character.carac_spend_cost(&Stats(Air), 50), 100);
    /// assert_eq!(character.carac_spend_cost(&Stats(Air), 150), 350);
    /// ```
    pub fn carac_spend_cost(&self, kind: &CaracKind, amount: u16) -> u16 {
        let current = *self.base_stats.get(kind).unwrap_or(&0);
        Self::carac_cost_from_zero(kind, current + amount)
            - Self::carac_cost_from_zero(kind, current)
    }

    /// Compute the number of points recovered by decreasing a characteristic.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// use dofus_stuff::character::*;
    /// use dofus_stuff::dofapi::{CaracKind::*, Element::*};
    ///
    /// let db_sets = HashMap::new();
    /// let mut character = Character::new(200, &db_sets);
    /// character.carac_spend(&Stats(Air), 200).unwrap();
    ///
    /// assert_eq!(character.carac_unspend_recover(&Stats(Air), 100), Ok(200));
    /// assert_eq!(character.carac_unspend_recover(&Stats(Air), 200), Ok(300));
    /// assert!(character.carac_unspend_recover(&Stats(Air), 201).is_err());
    /// ```
    pub fn carac_unspend_recover(
        &self,
        kind: &'i CaracKind,
        amount: u16,
    ) -> Result<u16, CharacterError<'i>> {
        let current = *self.base_stats.get(kind).unwrap_or(&0);

        if current < amount {
            Err(CharacterError::NotEnoughCaracs(kind))
        } else {
            Ok(Self::carac_cost_from_zero(kind, current)
                - Self::carac_cost_from_zero(kind, current - amount))
        }
    }

    /// Try to spend points to increase characteristic.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// use dofus_stuff::character::*;
    /// use dofus_stuff::dofapi::{CaracKind::*, Element::*};
    ///
    /// let db_sets = HashMap::new();
    /// let mut character = Character::new(200, &db_sets);
    ///
    /// assert!(character.carac_spend(&Stats(Air), 100).is_ok());
    /// assert!(character.carac_spend(&Stats(Air), 400).is_err());
    /// ```
    pub fn carac_spend(
        &mut self,
        kind: &'i CaracKind,
        amount: u16,
    ) -> Result<(), CharacterError> {
        let cost = self.carac_spend_cost(kind, amount);

        if cost > self.unspent {
            Err(CharacterError::NotEnoughPoints)
        } else {
            self.unspent -= cost;
            self.base_stats
                .entry(kind)
                .and_modify(|x| *x += amount)
                .or_insert(amount);
            Ok(())
        }
    }

    /// Try to recover points by decreasing a characteristic.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// use dofus_stuff::character::*;
    /// use dofus_stuff::dofapi::{CaracKind::*, Element::*};
    ///
    /// let db_sets = HashMap::new();
    /// let mut character = Character::new(200, &db_sets);
    /// character.carac_spend(&Stats(Air), 200).unwrap();
    ///
    /// assert_eq!(character.carac_unspend(&Stats(Air), 100), Ok(()));
    /// assert_eq!(character.carac_unspend(&Stats(Air), 99), Ok(()));
    /// assert!(character.carac_unspend(&Stats(Air), 201).is_err());
    /// ```
    pub fn carac_unspend(
        &mut self,
        kind: &'i CaracKind,
        amount: u16,
    ) -> Result<(), CharacterError> {
        let recovered = self.carac_unspend_recover(kind, amount)?;
        self.unspent += recovered;

        if let Some(val) = self.base_stats.get_mut(kind) {
            *val -= amount;
        }

        Ok(())
    }

    /// Try to spend points to increase a characteristic, if there is not
    /// enough unspent points, try to seek points from another characteristic.
    ///
    /// Returns Character::NotEnoughCaracs(seek_from) if it is not possible to
    /// find enough points to increase properly this stat.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// use dofus_stuff::character::*;
    /// use dofus_stuff::dofapi::{CaracKind::*, Element::*};
    ///
    /// let db_sets = HashMap::new();
    /// let mut character = Character::new(200, &db_sets);
    ///
    /// // Seek from unspent points
    /// assert!(
    ///     character
    ///         .carac_spend_or_seek(&Stats(Air), 150, &Wisdom)
    ///         .is_ok()
    /// );
    /// assert!(
    ///     character
    ///         .carac_spend_or_seek(&Vitality, 795, &Wisdom)
    ///         .is_ok()
    /// );
    ///
    /// // Seek from another carac
    /// assert!(
    ///     character
    ///         .carac_spend_or_seek(&Vitality, 200, &Stats(Air))
    ///         .is_ok()
    /// );
    /// assert!(
    ///     character
    ///         .carac_spend_or_seek(&Vitality, 1, &Stats(Air))
    ///         .is_err()
    /// );
    /// ```
    pub fn carac_spend_or_seek(
        &mut self,
        kind: &'i CaracKind,
        amount: u16,
        seek_from: &'i CaracKind,
    ) -> Result<(), CharacterError> {
        let cost = self.carac_spend_cost(kind, amount);

        // If there is not enough unspent points, seek `cost - self.unspent`
        // points by decreasing another carac.
        if self.unspent < cost {
            let required = cost - self.unspent;
            let current = *self.base_stats.get(seek_from).unwrap_or(&0);

            // If seek_from can't free enough points, abort computation.
            if self.carac_unspend_recover(seek_from, current).unwrap()
                < required
            {
                return Err(CharacterError::NotEnoughCaracs(seek_from));
            }

            let decrease = {
                let mut min = 1;
                let mut max = current + 1;

                while min + 1 < max {
                    let mid = (min + max - 1) / 2;

                    if self.carac_unspend_recover(seek_from, mid).unwrap()
                        < required
                    {
                        min = mid + 1;
                    } else {
                        max = mid + 1;
                    }
                }

                debug_assert_eq!(min + 1, max);
                min
            };

            self.carac_unspend(seek_from, decrease).unwrap();
        }

        self.carac_spend(kind, amount).unwrap();
        Ok(())
    }

    // __     __    _ _     _ _ _
    // \ \   / /_ _| (_) __| (_) |_ _   _
    //  \ \ / / _` | | |/ _` | | __| | | |
    //   \ V / (_| | | | (_| | | |_| |_| |
    //    \_/ \__,_|_|_|\__,_|_|\__|\__, |
    //                              |___/

    /// Return a condition equivalent to the union of all item's conditions.
    pub fn all_conditions(&self) -> Condition {
        self.iter_items().fold(Condition::new(), |acc, item| {
            Condition::and(acc, item.conditions.clone())
        })
    }

    /// Compute an approximate smithmage weight value required to complie to a
    /// condition.
    pub fn condition_overflow(&self, cond: &Condition) -> f64 {
        // NOTE: this is costly and there may be a way to implement cleaningly
        // a cache mechanic.
        let caracs = self.get_caracs();

        let atom_overflow = |atom: &ConditionAtom| match atom {
            ConditionAtom::Stats(kind, order, target) => {
                let current = caracs.get_carac(kind);

                if current.cmp(target) != *order {
                    kind.smithmage_weight().unwrap_or(0.)
                        * f64::from((current - target).abs() + 1)
                } else {
                    0.
                }
            }
            ConditionAtom::RestrictSetBonuses => {
                let count_synergies: u8 = self
                    .iter_set_synergies()
                    .map(|(_, count)| count)
                    .filter(|&count| count > 1)
                    .sum();

                if count_synergies > 2 {
                    CaracKind::AP.smithmage_weight().unwrap()
                        * f64::from(count_synergies - 2)
                } else {
                    0.
                }
            }
            ConditionAtom::Other(_) => 0.,
        };

        cond.clauses()
            .iter()
            .map(|clause| {
                clause
                    .iter()
                    .map(|atom| atom_overflow(atom))
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .expect("Empty clause are not allowed")
            })
            .sum()
    }

    pub fn count_item_conflicts(&self) -> u8 {
        let mut conflicts = 0;

        // Read first item
        for (i, item1) in self.item_slots.iter().enumerate() {
            if let Some(item1) = item1.item {
                // Read second item
                for item2 in &self.item_slots[(i + 1)..self.item_slots.len()] {
                    if let Some(item2) = item2.item {
                        if item1._id == item2._id
                            && (item1.set_id.is_some()
                                || item1.item_type == ItemType::Trophy
                                || item1.item_type == ItemType::Dofus)
                        {
                            conflicts += 1;
                        }
                    }
                }
            }
        }

        conflicts
    }
}

//  ____                 ____
// |  _ \ __ ___      __/ ___|__ _ _ __ __ _  ___ ___
// | |_) / _` \ \ /\ / / |   / _` | '__/ _` |/ __/ __|
// |  _ < (_| |\ V  V /| |__| (_| | | | (_| | (__\__ \
// |_| \_\__,_| \_/\_/  \____\__,_|_|  \__,_|\___|___/
//

pub struct RawCaracs<'c>(HashMap<&'c CaracKind, i16>);

impl RawCaracs<'_> {
    fn as_map(&self) -> &HashMap<&CaracKind, i16> {
        match self {
            RawCaracs(map) => map,
        }
    }

    pub fn get_base_carac(&self, kind: &CaracKind) -> i16 {
        match kind {
            CaracKind::Vitality => 1050 + 100,
            CaracKind::AP => 7 + 1,
            CaracKind::MP => 3 + 1,
            CaracKind::Range => 1,
            CaracKind::Stats(_) => 100,
            CaracKind::Wisdom => 100,
            CaracKind::Prospecting => 100,
            CaracKind::Initiative => 1000,
            _ => 0,
        }
    }

    fn get_raw_carac(&self, kind: &CaracKind) -> i16 {
        *self.as_map().get(kind).unwrap_or(&0)
    }

    pub fn get_carac(&self, kind: &CaracKind) -> i16 {
        use CaracKind::*;
        use Element::*;

        let res = self.get_raw_carac(kind) + self.get_base_carac(kind);
        match kind {
            AP => std::cmp::min(res, 12),
            MP => std::cmp::min(res, 6),
            Range => std::cmp::min(res, 6),
            Initiative => {
                res + [Air, Earth, Fire, Water]
                    .iter()
                    .map(|&elem| self.get_raw_carac(&Stats(elem)))
                    .sum::<i16>()
            }
            Damage(_elem) => res + self.get_raw_carac(&RawDamage),
            Prospecting => res + self.get_raw_carac(&Stats(Water)) / 10,
            Pods => res + 5 * self.get_raw_carac(&Stats(Earth)),
            Dodge | Lock => res + self.get_raw_carac(&Stats(Air)) / 10,
            APReduction | APResistance | MPReduction | MPResistance => {
                res + self.get_carac(&Wisdom) / 10
            }
            PerResistance(_) => std::cmp::min(50, res),
            _ => res,
        }
    }

    pub fn eval(&self, value: &RawCaracsValue) -> f64 {
        match value {
            RawCaracsValue::Carac(carac) => self.get_carac(carac) as f64,
            RawCaracsValue::PowStats(elem) => {
                self.get_carac(&CaracKind::Stats(elem.effective_stat())) as f64
                    + self.get_carac(&CaracKind::Power) as f64
            }
            RawCaracsValue::MeanExtraDamage(elem) => {
                self.mean_extra_damage(*elem)
            }
            RawCaracsValue::PerResVariance => {
                let kinds = [
                    CaracKind::PerResistance(Element::Air),
                    CaracKind::PerResistance(Element::Earth),
                    CaracKind::PerResistance(Element::Fire),
                    CaracKind::PerResistance(Element::Water),
                    CaracKind::PerResistance(Element::Neutral),
                ];

                let mean = self.mean_per_resistance();
                let square_diffs = kinds
                    .iter()
                    .map(|kind| (self.get_carac(kind) as f64 - mean).powi(2));
                (square_diffs.sum::<f64>() / 5.).sqrt()
            }
            RawCaracsValue::Resiliance => self.resiliance(),
            RawCaracsValue::MeanDamage(effects) => {
                self.mean_weapon_damage(effects)
            }
        }
    }

    pub fn mean_per_resistance(&self) -> f64 {
        let kinds = [
            CaracKind::PerResistance(Element::Air),
            CaracKind::PerResistance(Element::Earth),
            CaracKind::PerResistance(Element::Fire),
            CaracKind::PerResistance(Element::Water),
            CaracKind::PerResistance(Element::Neutral),
        ];

        kinds
            .iter()
            .map(|kind| self.get_carac(kind) as f64)
            .sum::<f64>()
            / kinds.len() as f64
    }

    pub fn resiliance(&self) -> f64 {
        let vitality = self.get_carac(&CaracKind::Vitality) as f64;
        let mean_res = self.mean_per_resistance();
        vitality / (1. - mean_res / 100.)
    }

    pub fn mean_extra_damage(&self, element: Element) -> f64 {
        let base_dmg: f64 = self.get_carac(&CaracKind::Damage(element)).into();
        let critical: f64 = (10 + self.get_carac(&CaracKind::Critical)).into();
        let crit_dmg: f64 = self.get_carac(&CaracKind::CriticalDamage).into();
        base_dmg + (crit_dmg * critical / 100.)
    }

    pub fn mean_effect_damage(
        &self,
        effect: &Effect,
        is_crit: bool,
        is_spell: bool,
        is_dist: bool,
    ) -> f64 {
        match effect {
            Effect::Hit {
                element, bounds, ..
            } => {
                let mut damage: f64 = self
                    .eval(&RawCaracsValue::PowStats(element.effective_stat()))
                    * (f64::from(*bounds.start()) + f64::from(*bounds.end()))
                    / 200.;

                damage +=
                    f64::from(self.get_carac(&CaracKind::Damage(*element)));

                if is_crit {
                    damage +=
                        f64::from(self.get_carac(&CaracKind::CriticalDamage));
                }

                if is_spell {
                    damage *= 1.
                        + f64::from(
                            self.get_carac(&CaracKind::PerSpellDamage),
                        ) / 100.;
                } else {
                    damage *= 1.
                        + f64::from(
                            self.get_carac(&CaracKind::PerWeaponDamage),
                        ) / 100.;
                }

                if is_dist {
                    damage *= 1.
                        + f64::from(
                            self.get_carac(&CaracKind::PerRangedDamage),
                        ) / 100.
                } else {
                    damage *= 1.
                        + f64::from(self.get_carac(&CaracKind::PerMeleeDamage))
                            / 100.
                }

                damage
            }
        }
    }

    pub fn mean_weapon_damage(&self, effects: &SpellEffects) -> f64 {
        let mean_dmg_nocrit: f64 = effects
            .effect
            .iter()
            .map(|effect| {
                self.mean_effect_damage(
                    effect,
                    false,
                    !effects.weapon,
                    effects.ranged,
                )
            })
            .sum();
        let mean_dmg_crit: f64 = effects
            .critical_effect
            .iter()
            .map(|effect| {
                self.mean_effect_damage(
                    effect,
                    true,
                    !effects.weapon,
                    effects.ranged,
                )
            })
            .sum();
        let crit: f64 = f64::from(
            i16::from(effects.critical) + self.get_carac(&CaracKind::Critical),
        ) / 100.;
        (1. - crit) * mean_dmg_nocrit + crit * mean_dmg_crit
    }
}

//  ____    ______     __    _
// |  _ \  / ___\ \   / /_ _| |_   _  ___
// | |_) || |    \ \ / / _` | | | | |/ _ \
// |  _ < | |___ _\ V / (_| | | |_| |  __/
// |_| \_(_)____(_)\_/ \__,_|_|\__,_|\___|
//

/// Enumeration of values that can be computed from a `RawCaracs`.
#[derive(Debug, Deserialize)]
pub enum RawCaracsValue {
    Carac(CaracKind),
    PowStats(Element),
    MeanExtraDamage(Element),
    MeanDamage(SpellEffects),
    PerResVariance,
    Resiliance,
}

impl RawCaracsValue {
    pub fn approx_smithmage_weight(&self) -> Result<f64, ()> {
        Ok(match self {
            RawCaracsValue::Carac(kind) => kind.smithmage_weight()?,
            RawCaracsValue::Resiliance => {
                0.75 * CaracKind::Vitality.smithmage_weight().unwrap()
            }
            RawCaracsValue::PowStats(elem) => {
                CaracKind::Stats(*elem).smithmage_weight().unwrap()
            }
            RawCaracsValue::MeanExtraDamage(elem) => {
                CaracKind::Damage(*elem).smithmage_weight().unwrap()
            }
            RawCaracsValue::PerResVariance => {
                CaracKind::PerResistance(Element::Neutral)
                    .smithmage_weight()
                    .unwrap()
            }
            RawCaracsValue::MeanDamage(_) => {
                CaracKind::Vitality.smithmage_weight().unwrap()
            }
        })
    }

    pub fn is_decreasing(&self) -> bool {
        match self {
            Self::PerResVariance => true,
            _ => false,
        }
    }
}
