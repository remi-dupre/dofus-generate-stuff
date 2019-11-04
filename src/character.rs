use std::collections::HashMap;

use crate::dofapi::carac::CaracKind;
use crate::dofapi::effect::Element;
use crate::dofapi::equipement::{Equipement, ItemType};
use crate::rls::Blackbox;

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

#[derive(Clone, Debug)]
pub struct Character<'i> {
    pub item_slots: Vec<ItemSlot<'static, 'i>>,
    pub base_stats: HashMap<CaracKind, i16>,
}

impl<'i> Character<'i> {
    pub fn new() -> Self {
        Character {
            item_slots: vec![
                ItemSlot::new(&[ItemType::Hat]),
                ItemSlot::new(&[ItemType::Cloak, ItemType::Backpack]),
                ItemSlot::new(&[ItemType::Amulet]),
                ItemSlot::new(&[ItemType::Ring]),
                ItemSlot::new(&[ItemType::Ring]),
                ItemSlot::new(&[ItemType::Belt]),
                ItemSlot::new(&[ItemType::Boots]),
                ItemSlot::new(&[ItemType::Shield]),
                ItemSlot::new(&[ItemType::Dofus, ItemType::Trophy]),
                ItemSlot::new(&[ItemType::Dofus, ItemType::Trophy]),
                ItemSlot::new(&[ItemType::Dofus, ItemType::Trophy]),
                ItemSlot::new(&[ItemType::Dofus, ItemType::Trophy]),
                ItemSlot::new(&[ItemType::Dofus, ItemType::Trophy]),
                ItemSlot::new(&[ItemType::Dofus, ItemType::Trophy]),
            ],
            base_stats: vec![
                (CaracKind::AP, 8),
                (CaracKind::MP, 4),
                (CaracKind::Range, 1),
                (CaracKind::Stats(Element::Air), 100),
                (CaracKind::Stats(Element::Earth), 100),
                (CaracKind::Stats(Element::Fire), 100),
                (CaracKind::Stats(Element::Water), 100),
                (CaracKind::Wisdom, 100),
                (CaracKind::Prospecting, 100),
                (CaracKind::Pods, 1000),
            ]
            .into_iter()
            .collect(),
        }
    }

    pub fn get_caracs(&self) -> RawCaracs {
        let mut ret = HashMap::new();

        self.item_slots
            .iter()
            .filter_map(|slot| {
                slot.item.map(|item| {
                    item.statistics.iter().map(|(kind, bounds)| {
                        (kind, std::cmp::max(bounds.start(), bounds.end()))
                    })
                })
            })
            .flatten()
            .chain(self.base_stats.iter())
            .for_each(|(kind, &val)| {
                ret.entry(kind).and_modify(|x| *x += val).or_insert(val);
            });

        RawCaracs(ret)
    }

    pub fn count_item_conflicts(&self) -> u8 {
        let mut conflicts = 0;

        // Read first item
        for (i, item1) in self.item_slots.iter().enumerate() {
            if let Some(item1) = item1.item {
                // Read second item
                for item2 in &self.item_slots[i + 1..self.item_slots.len()] {
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

//  _____                    _  _____
// |_   _|_ _ _ __ __ _  ___| ||_   _|   _ _ __   ___
//   | |/ _` | '__/ _` |/ _ \ __|| || | | | '_ \ / _ \
//   | | (_| | | | (_| |  __/ |_ | || |_| | |_) |  __/
//   |_|\__,_|_|  \__, |\___|\__||_| \__, | .__/ \___|
//                |___/              |___/|_|

pub enum RawCaracsValue<'c> {
    Carac(&'c CaracKind),
    Resiliance,
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

    fn get_raw_carac(&self, kind: &CaracKind) -> i16 {
        *self.as_map().get(kind).unwrap_or(&0)
    }

    pub fn get_carac(&self, kind: &CaracKind) -> i16 {
        use CaracKind::*;
        use Element::*;

        let res = self.get_raw_carac(kind);
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
            RawCaracsValue::Resiliance => self.resiliance(),
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
}

//  ____                      _
// / ___|  ___  __ _ _ __ ___| |__
// \___ \ / _ \/ _` | '__/ __| '_ \
//  ___) |  __/ (_| | | | (__| | | |
// |____/ \___|\__,_|_|  \___|_| |_|
//

impl Blackbox for Character<'_> {
    fn eval(&self) -> f64 {
        let carac_targets = {
            use CaracKind::*;
            use RawCaracsValue::*;

            [
                (Carac(&AP), 10),
                (Carac(&MP), 6),
                (Carac(&Range), 0),
                (Carac(&APResistance), 40),
                (Carac(&MPResistance), 40),
                (Carac(&Stats(Element::Water)), 900),
                (Resiliance, 3000),
            ]
        };

        let target = |target: f64, width: f64, x: f64| -> f64 {
            1. / (1. + (-(x - target) / width).exp())
        };

        let caracs = self.get_caracs();
        let targets_weight: f64 = carac_targets
            .iter()
            .map(|(target_type, target_val)| {
                let val = caracs.eval(target_type);
                let width = match target_type {
                    RawCaracsValue::Resiliance => 150.,
                    RawCaracsValue::Carac(kind) => {
                        100. / kind.smithmage_weight()
                    }
                };
                target(*target_val as f64, width, val)
            })
            .product();

        let count_item_conflicts = self.count_item_conflicts();
        let conflicts_weight = 0.5f64.powi(count_item_conflicts.into());

        targets_weight * conflicts_weight
    }
}
