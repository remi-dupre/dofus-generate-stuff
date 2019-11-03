use serde::Deserialize;

//  _____ _                           _
// | ____| | ___ _ __ ___   ___ _ __ | |_
// |  _| | |/ _ \ '_ ` _ \ / _ \ '_ \| __|
// | |___| |  __/ | | | | |  __/ | | | |_
// |_____|_|\___|_| |_| |_|\___|_| |_|\__|
//

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq, Hash)]
pub enum Element {
    Earth,
    Water,
    Air,
    Fire,
    Neutral,
}

//  ____                                   _     _
// |  _ \  __ _ _ __ ___   __ _  __ _  ___| |   (_)_ __   ___
// | | | |/ _` | '_ ` _ \ / _` |/ _` |/ _ \ |   | | '_ \ / _ \
// | |_| | (_| | | | | | | (_| | (_| |  __/ |___| | | | |  __/
// |____/ \__,_|_| |_| |_|\__,_|\__, |\___|_____|_|_| |_|\___|
//                              |___/

#[derive(Debug, Deserialize)]
pub struct DamageLine {
    pub element:   Element,
    pub min:       i64,
    pub max:       i64,
    pub lifesteal: bool,
}
