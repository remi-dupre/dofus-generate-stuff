pub mod character;
pub mod dofapi;

extern crate rand;
extern crate rayon;
extern crate serde_json;

use rand::prelude::*;
use rayon::prelude::*;

use crate::character::Character;
use crate::dofapi::carac::CaracKind;
use crate::dofapi::effect::Element;
use crate::dofapi::equipement::Equipement;

use std::fs::File;
use std::io;

fn main() -> io::Result<()> {
    // ---
    eprintln!("-- Loading data...");
    let file = File::open("./data/allequipments.json")?;
    let equipements: Vec<Equipement> =
        serde_json::from_reader(io::BufReader::new(file))?;

    // --- Index per slot
    let slot_pool: Vec<_> = Character::new()
        .item_slots
        .iter()
        .map(|slot| {
            equipements
                .iter()
                .filter(|item| slot.get_allowed().contains(&item.item_type))
                .collect::<Vec<_>>()
        })
        .collect();

    // ---
    eprintln!("-- Build random stuffs...");
    let best = (0..1_000_000)
        .into_par_iter()
        .map_init(
            || rand::thread_rng(),
            |mut rng, _| {
                let mut character = Character::new();
                for (i, slot) in character.item_slots.iter_mut().enumerate() {
                    let item = slot_pool[i]
                        .choose(&mut rng)
                        .expect("No available item for slot");
                    slot.equip(item);
                }

                character
            },
        )
        .filter(|character| {
            character.get_carac(&CaracKind::Vitality) >= 1500
                && (character.get_carac(&CaracKind::Stats(Element::Fire))
                    + character.get_carac(&CaracKind::Power))
                    >= 500
        })
        .max_by_key(|character| {
            3 * (character.get_carac(&CaracKind::Stats(Element::Fire))
                + character.get_carac(&CaracKind::Power))
                + character.get_carac(&CaracKind::Vitality)
        });

    // ---
    eprintln!("-- Result...");
    match best {
        None => println!("No feasible stuff found :("),
        Some(character) => {
            println!("-----------------------------------------------");
            character
                .item_slots
                .iter()
                .for_each(|i| println!(" {:^45}", i.get_item().unwrap().name));
            println!("-----------------------------------------------");
            println!(
                " Vitality       {:>30}",
                character.get_carac(&CaracKind::Vitality)
            );
            println!(
                " Intelligence   {:>30}",
                character.get_carac(&CaracKind::Stats(Element::Fire))
            );
            println!(
                " Power          {:>30}",
                character.get_carac(&CaracKind::Power)
            );
        }
    }

    Ok(())
}
