pub mod dofapi;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate strum_macros;

extern crate regex;
extern crate serde_json;
extern crate strum;

use crate::dofapi::equipement::Equipement;

use std::fs::File;
use std::io;

fn main() -> io::Result<()> {
    // ---
    println!("-- Loading data...");
    let file = File::open("./data/allequipments.json")?;
    let equipements: Vec<Equipement> =
        serde_json::from_reader(io::BufReader::new(file))?;

    // ---
    println!("{:#?}", equipements[0]);
    Ok(())
}
