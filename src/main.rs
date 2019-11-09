pub mod character;
pub mod dofapi;
pub mod input;
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

use crate::character::{Character, RawCaracsValue};
use crate::dofapi::{fix_all_trophy, CaracKind, Element, Equipement, Set};
use crate::input::InputRequest;
use crate::search::{eval_character, optimize_character};

/// List of files containing the list of equipements.
const EQUIPEMENT_FILES: [&str; 4] = [
    "./data/equipments.json",
    "./data/mounts.json",
    "./data/pets.json",
    "./data/weapons.json",
];

/// File containing the list of sets.
const SET_FILE: &str = "./data/sets.json";

/// Default file to read as input when no parameter is specified.
const DEFAULT_INPUT_PATH: &str = "input.json";

fn main() -> io::Result<()> {
    // --- Open item database
    eprintln!("-- Loading data...");

    let sets: HashMap<u64, Set> = {
        let file = File::open(SET_FILE).unwrap_or_else(|err| {
            panic!(
                "Could not open `{}`, make sure you downloaded the item \
                 database: {}",
                SET_FILE, err
            )
        });
        let vec: Vec<_> = serde_json::from_reader(io::BufReader::new(file))
            .unwrap_or_else(|err| {
                panic!("Could not parse `{}`: {}", SET_FILE, err)
            });
        vec.into_iter().map(|set: Set| (set._id, set)).collect()
    };

    let mut equipements: Vec<Equipement> = EQUIPEMENT_FILES
        .iter()
        .map(|path| {
            let file = File::open(path).unwrap_or_else(|err| {
                panic!(
                    "Could not open `{}`, make sure you downloaded the item \
                     database: {}",
                    path, err
                )
            });
            serde_json::from_reader(io::BufReader::new(file)).unwrap_or_else(
                |err| panic!("Could not parse `{}`: {}", path, err),
            )
        })
        .fold(Vec::new(), |mut acc, db: Vec<_>| {
            acc.extend(db);
            acc
        });

    // --- Fix broken elements of the database
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

    // --- Read output and generate appropriate stuff and character.
    eprintln!("-- Reading input...");

    let input_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_INPUT_PATH.into());

    let input: InputRequest = {
        let file = File::open(&input_path).unwrap_or_else(|err| {
            panic!("Could not open input file `{}`: {}", input_path, err)
        });
        serde_json::from_reader(io::BufReader::new(file)).unwrap_or_else(
            |err| {
                panic!("Could not parse input file `{}`: {}", input_path, err)
            },
        )
    };

    let filtered_equipements: Vec<_> = equipements
        .iter()
        .filter(|item| item.level <= input.level)
        .filter(|item| !input.banned_types.contains(&item.item_type))
        .cloned()
        .collect();

    let init_character = Character::new(input.level, &sets);

    for target_line in &input.target {
        if let RawCaracsValue::Carac(CaracKind::Special(ref s)) = target_line.0
        {
            eprintln!(r"/!\ Unrecognised target in the input: `{}`", s);
        }
    }

    // --- Build the stuff
    eprintln!("-- Building random stuffs...");

    let best = optimize_character(
        init_character,
        8,
        &input.target,
        &filtered_equipements,
    )
    .into_iter()
    .max_by(|c1, c2| {
        let eval1 = eval_character(c1, &input.target);
        let eval2 = eval_character(c2, &input.target);
        eval1.partial_cmp(&eval2).unwrap_or_else(|| {
            println!(r"/!\ {} can't be compared to {}", eval1, eval2);
            std::cmp::Ordering::Equal
        })
    });

    // --- Show results
    eprintln!("-- Result...");
    match best {
        None => println!("No feasible stuff found :("),
        Some(character) => {
            println!("------------------------------------------------");
            character
                .item_slots
                .iter()
                .filter_map(|slot| slot.get_item())
                .for_each(|item| println!(" {:^46}  {}", item.name, item.url));
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
            println!("------------------------------------------------");
            for (target, val) in input.target {
                println!(
                    " - {:?}: {:.2} / {}",
                    target,
                    character.get_caracs().eval(&target),
                    val
                );
            }
            println!("------------------------------------------------");
            println!("\nstats: {:?}", character.base_stats);
            println!(
                "conditions ({}): {:?}",
                character.condition_overflow(&character.all_conditions()),
                character.all_conditions()
            );
            println!(
                "conflicts: {}",
                character.condition_overflow(&character.all_conditions())
            );
        }
    }

    Ok(())
}
