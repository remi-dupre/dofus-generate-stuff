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

use std::collections::HashMap;
use std::fs::File;
use std::io;

use regex::Regex;

use crate::character::Character;
use crate::dofapi::{fix_all_trophy, CaracKind, Element, Equipement, Set};
use crate::search::{eval_character, optimize_character};

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

    fix_all_trophy(&mut equipements);
    let equipements: Vec<Equipement> = equipements
        .into_iter()
        .map(|mut item| {
            if item.is_weapon() {
                // Fix weapon effects: remove PA reduction and damage lines
                lazy_static! {
                    static ref RE_DMG: Regex = Regex::new(r"\(.*\)").unwrap();
                }

                item.statistics = HashMap::from(item.statistics)
                    .into_iter()
                    .filter(|carac| match carac {
                        (CaracKind::AP, bounds) => {
                            // Weapons AP reduction is strangly formatted and
                            // have bounds of different sizes
                            bounds.start() * bounds.end() >= 0
                        }
                        (CaracKind::Special(desc), _) => {
                            // Weapons damage line are surrounded with
                            // parenthesis
                            !RE_DMG.is_match(desc)
                        }
                        _ => true,
                    })
                    .collect::<HashMap<_, _>>()
                    .into();
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
        optimize_character(Character::new(&sets), 32, &target, &equipements)
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
                .filter_map(|slot| slot.get_item())
                .for_each(|item| {
                    println!(" {:^46}   {}", item.name, item.url)
                });
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
