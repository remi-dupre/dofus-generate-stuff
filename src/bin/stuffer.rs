#[macro_use]
extern crate lazy_static;
extern crate dofus_stuff;
extern crate rand;
extern crate regex;
extern crate serde_json;

use std::collections::HashMap;
use std::fs::File;
use std::io;

use regex::Regex;

use dofus_stuff::character::{Character, RawCaracsValue};
use dofus_stuff::dofapi::{
    fix_all_trophy, CaracKind, Element, Equipement, ItemType, Set,
};
use dofus_stuff::search::optimize_character;
use serde::Deserialize;

//   ____                _              _
//  / ___|___  _ __  ___| |_ __ _ _ __ | |_ ___
// | |   / _ \| '_ \/ __| __/ _` | '_ \| __/ __|
// | |__| (_) | | | \__ \ || (_| | | | | |_\__ \
//  \____\___/|_| |_|___/\__\__,_|_| |_|\__|___/
//

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

//  ___                   _
// |_ _|_ __  _ __  _   _| |_
//  | || '_ \| '_ \| | | | __|
//  | || | | | |_) | |_| | |_
// |___|_| |_| .__/ \__,_|\__|
//           |_|

/// Input request for building a stuff.
#[derive(Deserialize)]
pub struct InputRequest {
    /// Level of the character to build a stuff for.
    #[serde(default = "default_level")]
    pub level: u8,

    /// Types of items that can't be used in the output.
    #[serde(default)]
    pub banned_types: Vec<ItemType>,

    /// List of approximate expected statistics in the output.
    #[serde(default)]
    pub target: Vec<(RawCaracsValue, f64)>,
}

/// Default level of a character.
fn default_level() -> u8 {
    200
}

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

    let character = optimize_character(
        init_character,
        &input.target,
        &filtered_equipements,
    );

    // --- Show results
    eprintln!("-- Result...");
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

    Ok(())
}
