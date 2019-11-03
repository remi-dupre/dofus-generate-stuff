pub mod dofapi;

extern crate serde_json;

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
