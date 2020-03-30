use rand::prelude::*;

use crate::character::{Character, RawCaracsValue};
use crate::dofapi::{CaracKind, Element, Equipement};
use crate::rls::rls;

const STEPS: u32 = 100_000;
const ASSIGNABLE_CARACS: &[CaracKind] = &[
    CaracKind::Vitality,
    CaracKind::Wisdom,
    CaracKind::Stats(Element::Air),
    CaracKind::Stats(Element::Earth),
    CaracKind::Stats(Element::Fire),
    CaracKind::Stats(Element::Water),
];

fn walk_character<'i>(
    init: &Character<'i>,
    rng: &mut impl rand::Rng,
    db_slot_pool: &[Vec<&'i Equipement>],
) -> Character<'i> {
    let mut new = init.clone();

    if rng.gen_bool(0.5) {
        // Swap some items
        let slot_i = rng.gen_range(0, db_slot_pool.len());
        let item = db_slot_pool[slot_i]
            .choose(rng)
            .expect("No available item for slot");
        new.item_slots[slot_i].equip(item);
        new
    } else {
        // Swap some statistics
        let kind = ASSIGNABLE_CARACS.iter().choose(rng).unwrap();
        let from = ASSIGNABLE_CARACS.iter().choose(rng).unwrap();

        if new
            .carac_spend_or_seek(kind, *[1, 5, 10].choose(rng).unwrap(), from)
            .is_err()
        {
            let _ = new.carac_spend_or_seek(kind, 1, from);
        }
        new
    }
}

pub fn eval_character(
    character: &Character<'_>,
    target: &[(RawCaracsValue, f64)],
) -> f64 {
    let target_min = |target: f64, width: f64, x: f64| -> f64 {
        1. / (1. + (-4. * (x - target) / width).exp())
    };

    let target_zero = |width: f64, x: f64| -> f64 {
        let nx = 2. * x / width; // normalize x
        (1. - (nx.exp() - (-nx).exp()) / (nx.exp() + (-nx).exp())).powi(2)
    };

    let caracs = character.get_caracs();
    let targets_weight: f64 = target
        .iter()
        .map(|(target_type, target_val)| {
            if let Ok(smithmage_weight) = target_type.approx_smithmage_weight()
            {
                let val = caracs.eval(target_type);
                let width = 100. / smithmage_weight;
                let invert =
                    if target_type.is_decreasing() { -1. } else { 1. };
                target_min(*target_val * invert, width, val * invert)
            } else {
                1.
            }
        })
        .product();

    let count_item_conflicts = character.count_item_conflicts();
    let conflicts_weight = 0.05f64.powi(count_item_conflicts.into());

    let conditions_weight = target_zero(
        200.,
        character.condition_overflow(&character.all_conditions()),
    );

    targets_weight * conflicts_weight * conditions_weight
}

pub fn optimize_character<'i>(
    init: Character<'i>,
    target: &[(RawCaracsValue, f64)],
    db_equipements: &'i [Equipement],
) -> Character<'i> {
    // Reorder set into pools assigned to each slot
    let slot_pool: Vec<_> = init
        .item_slots
        .iter()
        .map(|slot| {
            db_equipements
                .iter()
                .filter(|item| slot.get_allowed().contains(&item.item_type))
                .collect::<Vec<_>>()
        })
        .collect();

    rls(
        init,
        STEPS,
        &mut rand::thread_rng(),
        |character| eval_character(character, target),
        |character, rng| walk_character(&character, rng, &slot_pool),
    )
}
