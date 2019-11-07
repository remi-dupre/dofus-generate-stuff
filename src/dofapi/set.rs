use std::collections::HashMap;

use serde::Deserialize;

use crate::dofapi::carac::CaracLines;

#[derive(Deserialize, Debug)]
pub struct Set {
    #[serde(rename = "ankamaId")]
    pub ankama_id: u64,

    #[serde(rename = "imgUrl")]
    pub img_url: String,

    pub _id:           u64,
    pub name:          String,
    pub level:         u8,
    pub url:           String,
    pub equipement_id: Option<u64>,
    pub weapon_id:     Option<u64>,
    pub bonus:         HashMap<u8, CaracLines>,
}
