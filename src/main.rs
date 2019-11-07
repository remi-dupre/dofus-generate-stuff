pub mod character;
pub mod dofapi;
pub mod rls;
pub mod search;

#[macro_use]
extern crate lazy_static;
extern crate rand;
extern crate rayon;
extern crate regex;
extern crate serde_json;

use crate::character::Character;
use crate::dofapi::{
    CaracKind, ConditionAtom, Element, Equipement, ItemType, Set,
};
use crate::search::{eval_character, optimize_character};

use std::collections::HashMap;
use std::fs::File;
use std::io;

fn main() -> io::Result<()> {
    // ---
    eprintln!("-- Loading data...");

    let sets: HashMap<u64, Set> = {
        let file = File::open("./data/sets.json")?;
        let vec: Vec<_> = serde_json::from_reader(io::BufReader::new(file))?;
        vec.into_iter().map(|set: Set| (set._id, set)).collect()
    };

    let mut equipements: Vec<Equipement> = {
        let file = File::open("./data/allequipments.json")?;
        serde_json::from_reader(io::BufReader::new(file))?
    };

    equipements.extend({
        let file = File::open("./data/mounts.json")?;
        let vec: Vec<_> = serde_json::from_reader(io::BufReader::new(file))?;
        vec
    });

    equipements.extend({
        let file = File::open("./data/pets.json")?;
        let vec: Vec<_> = serde_json::from_reader(io::BufReader::new(file))?;
        vec
    });

    equipements.extend({
        let file = File::open("./data/allweapons.json")?;
        let vec: Vec<_> = serde_json::from_reader(io::BufReader::new(file))?;
        vec
    });

    let equipements: Vec<Equipement> = equipements
        .into_iter()
        // .filter(|item| item.level >= 150 || item.item_type ==
        // ItemType::Dofus)
        .map(|mut item| {
            // Fix trophy conditions, it seems to be overall a good fix except
            // for a few exceptions, e.g.:
            //  - Major Barbarian
            //  - Major Transporter
            //  - Cantile's Boots
            if item.item_type == ItemType::Trophy {
                let smithmage_value: f64 = item
                    .statistics
                    .as_map()
                    .iter()
                    .map(|(kind, value)| {
                        kind.smithmage_weight() * f64::from(*value.start())
                    })
                    .sum();

                if smithmage_value >= 72. {
                    item.conditions = ConditionAtom::RestrictSetBonuses.into();
                }
            }

            item
        })
        .collect();

    // ---
    eprintln!("-- Build random stuffs...");

    let target: Vec<_> = {
        let file = File::open("input.json")?;
        serde_json::from_reader(io::BufReader::new(file))?
    };

    let best =
        optimize_character(Character::new(&sets), 8, &target, &equipements)
            .into_iter()
            .max_by(|c1, c2| {
                let eval1 = eval_character(c1, &target);
                let eval2 = eval_character(c2, &target);
                eval1.partial_cmp(&eval2).unwrap()
            });

    // ---
    eprintln!("-- Result...");
    match best {
        None => println!("No feasible stuff found :("),
        Some(character) => {
            println!("------------------------------------------------");
            character
                .item_slots
                .iter()
                .for_each(|i| println!(" {:^46}", i.get_item().unwrap().name));
            println!("------------------------------------------------");

            let stats = &[
                CaracKind::AP,
                CaracKind::MP,
                CaracKind::Range,
                CaracKind::Vitality,
                CaracKind::Initiative,
                CaracKind::Stats(Element::Air),
                CaracKind::Stats(Element::Earth),
                CaracKind::Stats(Element::Fire),
                CaracKind::Stats(Element::Water),
                CaracKind::Power,
                CaracKind::Critical,
                CaracKind::CriticalDamage,
                CaracKind::Damage(Element::Air),
                CaracKind::Damage(Element::Earth),
                CaracKind::Damage(Element::Fire),
                CaracKind::Damage(Element::Water),
                CaracKind::Wisdom,
                CaracKind::APResistance,
                CaracKind::MPResistance,
                CaracKind::Lock,
                CaracKind::Dodge,
                CaracKind::PerResistance(Element::Air),
                CaracKind::PerResistance(Element::Earth),
                CaracKind::PerResistance(Element::Fire),
                CaracKind::PerResistance(Element::Water),
                CaracKind::PerResistance(Element::Neutral),
            ];

            let caracs = character.get_caracs();
            for stat in stats {
                println!(" {:35} {:>10}", stat, caracs.get_carac(stat));
            }
            println!("\nstats: {:?}", character.base_stats);
            println!(
                "conditions ({}): {:?}",
                character.condition_overflow(&character.all_conditions()),
                character.all_conditions()
            );
        }
    }

    Ok(())
}
