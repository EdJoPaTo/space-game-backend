use typings::fixed::module::Effect;
use typings::persist::ship::Status;

fn apply_damage(status: &mut Status, damage: u16) {
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
/// Doesnt care about whats possible with a given ship!
pub fn apply_to_status(before: Status, effects: &[Effect]) -> Status {
    let mut capacitor = before.capacitor as i32;
    let mut armor = before.hitpoints_armor;
    let mut damage_sum: u16 = 0;

    for effect in effects {
        match effect {
            Effect::Capacitor(amount) => {
                capacitor = capacitor.saturating_add(*amount as i32);
            }
            Effect::ArmorRepair(amount) => {
                armor = armor.saturating_add(*amount);
            }
            Effect::Damage(damage) => {
                damage_sum = damage_sum.saturating_add(*damage);
            }
            Effect::Mine(_) | Effect::WarpDisruption => { /* No effect */ }
        }
    }

    let mut result = Status {
        capacitor: capacitor.max(0) as u16,
        hitpoints_armor: armor,
        hitpoints_structure: before.hitpoints_structure,
    };
    apply_damage(&mut result, damage_sum);
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
