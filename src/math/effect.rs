#![allow(clippy::match_same_arms, dead_code)]

use typings::fixed::module::Effect;
use typings::persist::ship::Status;

fn apply_damage(status: &mut Status, damage: u32) {
    let mut dmg_remaining = damage;

    let armor_dmg = status.hitpoints_armor.min(dmg_remaining);
    dmg_remaining -= armor_dmg;
    status.hitpoints_armor -= armor_dmg;

    status.hitpoints_structure = status.hitpoints_structure.saturating_sub(dmg_remaining);
}

#[allow(
    clippy::cast_lossless,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
pub fn apply_to_status(before: &Status, max: &Status, effect: &Effect) -> Status {
    let mut result = before.clone();

    match effect {
        Effect::Capacitor(amount) => {
            let before = before.capacitor as i64;
            let after = before
                .saturating_add(*amount as i64)
                .max(0)
                .min(max.capacitor as i64);
            result.capacitor = after as u32;
        }
        Effect::ArmorRepair(amount) => {
            result.hitpoints_armor = before
                .hitpoints_armor
                .saturating_add(*amount)
                .min(max.hitpoints_armor);
        }
        Effect::Damage(damage) => {
            apply_damage(&mut result, *damage);
        }
        Effect::Mine(_) | Effect::WarpDisruption => { /* No effect */ }
    }
    result
}

#[test]
fn damage_against_armor() {
    let mut status = Status {
        capacitor: 0,
        hitpoints_armor: 42,
        hitpoints_structure: 42,
    };
    apply_damage(&mut status, 10);
    assert_eq!(
        status,
        Status {
            capacitor: 0,
            hitpoints_armor: 32,
            hitpoints_structure: 42,
        }
    );
}

#[test]
fn damage_against_structure() {
    let mut status = Status {
        capacitor: 0,
        hitpoints_armor: 0,
        hitpoints_structure: 42,
    };
    apply_damage(&mut status, 10);
    assert_eq!(
        status,
        Status {
            capacitor: 0,
            hitpoints_armor: 0,
            hitpoints_structure: 32,
        }
    );
}

#[test]
fn damage_against_armor_and_structure() {
    let mut status = Status {
        capacitor: 0,
        hitpoints_armor: 3,
        hitpoints_structure: 42,
    };
    apply_damage(&mut status, 10);
    assert_eq!(
        status,
        Status {
            capacitor: 0,
            hitpoints_armor: 0,
            hitpoints_structure: 35,
        }
    );
}

#[test]
fn damage_against_structure_min_zero() {
    let mut status = Status {
        capacitor: 0,
        hitpoints_armor: 0,
        hitpoints_structure: 2,
    };
    apply_damage(&mut status, 10);
    assert_eq!(
        status,
        Status {
            capacitor: 0,
            hitpoints_armor: 0,
            hitpoints_structure: 0,
        }
    );
}
