use std::collections::{HashMap, HashSet};
use std::iter;

use crate::dofapi::carac::CaracKind;
use crate::dofapi::effect::Element;
use crate::dofapi::equipement::{Equipement, ItemType};

#[derive(Debug)]
pub struct ItemSlot<'i> {
    allowed: HashSet<ItemType>,
    item:    Option<&'i Equipement>,
}

impl<'i> ItemSlot<'i> {
    pub fn new(allowed: impl Iterator<Item = ItemType>) -> Self {
        ItemSlot {
            allowed: allowed.collect(),
            item:    None,
        }
    }

    pub fn equip(&mut self, item: &'i Equipement) {
        if !self.allowed.contains(&item.item_type) {
            panic!("Trying to equip incorrect item type");
        }
        self.item = Some(item);
    }

    pub fn get_allowed(&self) -> &HashSet<ItemType> {
        &self.allowed
    }

    pub fn get_item(&self) -> Option<&'i Equipement> {
        self.item
    }
}

#[derive(Debug)]
pub struct Character<'i> {
    pub item_slots: Vec<ItemSlot<'i>>,
    pub base_stats: HashMap<CaracKind, i16>,
}

impl<'i> Character<'i> {
    pub fn new() -> Self {
        Character {
            item_slots: vec![
                ItemSlot::new(iter::once(ItemType::Hat)),
                ItemSlot::new(
                    vec![ItemType::Cloak, ItemType::Backpack].into_iter(),
                ),
                ItemSlot::new(iter::once(ItemType::Amulet)),
                ItemSlot::new(iter::once(ItemType::Ring)),
                ItemSlot::new(iter::once(ItemType::Ring)),
                ItemSlot::new(iter::once(ItemType::Belt)),
                ItemSlot::new(iter::once(ItemType::Boots)),
                ItemSlot::new(iter::once(ItemType::Shield)),
                ItemSlot::new(
                    vec![ItemType::Dofus, ItemType::Trophy].into_iter(),
                ),
                ItemSlot::new(
                    vec![ItemType::Dofus, ItemType::Trophy].into_iter(),
                ),
                ItemSlot::new(
                    vec![ItemType::Dofus, ItemType::Trophy].into_iter(),
                ),
                ItemSlot::new(
                    vec![ItemType::Dofus, ItemType::Trophy].into_iter(),
                ),
                ItemSlot::new(
                    vec![ItemType::Dofus, ItemType::Trophy].into_iter(),
                ),
                ItemSlot::new(
                    vec![ItemType::Dofus, ItemType::Trophy].into_iter(),
                ),
            ],
            base_stats: vec![
                (CaracKind::AP, 7),
                (CaracKind::MP, 3),
                (CaracKind::Stats(Element::Air), 100),
                (CaracKind::Stats(Element::Earth), 100),
                (CaracKind::Stats(Element::Fire), 100),
                (CaracKind::Stats(Element::Water), 100),
                (CaracKind::Prospecting, 100),
                (CaracKind::Pods, 1000),
            ]
            .into_iter()
            .collect(),
        }
    }

    pub fn get_carac(&self, carac: &CaracKind) -> i16 {
        let base: i16 = *self.base_stats.get(carac).unwrap_or(&0);
        let additional: i16 = self
            .item_slots
            .iter()
            .filter_map(|slot| slot.item)
            .map(|item| item.statistics.get(carac).map_or(0, |x| *x.end()))
            .sum();
        base + additional
    }
}
