use typings::fixed::module::Effect;
use typings::persist::ship::Status;

const fn apply_damage(mut status: Status, damage: u16) -> Status {
    let structure_dmg = damage.saturating_sub(status.hitpoints_armor);
    status.hitpoints_armor = status.hitpoints_armor.saturating_sub(damage);
    status.hitpoints_structure = status.hitpoints_structure.saturating_sub(structure_dmg);
    status
}

#[allow(clippy::cast_sign_loss)]
const fn can_apply_to_origin(status: Status, effect: Effect) -> bool {
    match effect {
        Effect::Capacitor(amount) => {
            if amount >= 0 {
                true
            } else {
                let amount = amount.saturating_abs() as u16;
                status.capacitor.checked_sub(amount).is_some()
            }
        }
        Effect::ArmorRepair(_) | Effect::Damage(_) | Effect::Mine(_) | Effect::WarpDisruption => {
            true
        }
    }
}

#[allow(clippy::cast_sign_loss)]
const fn saturating_apply(mut status: Status, effect: Effect) -> Status {
    match effect {
        Effect::Capacitor(amount) => {
            status.capacitor = if amount >= 0 {
                status.capacitor.saturating_add(amount as u16)
            } else {
                let amount = amount.saturating_abs() as u16;
                status.capacitor.saturating_sub(amount)
            };
            status
        }
        Effect::ArmorRepair(amount) => {
            status.hitpoints_armor = status.hitpoints_armor.saturating_add(amount);
            status
        }
        Effect::Damage(damage) => apply_damage(status, damage),
        Effect::Mine(_) | Effect::WarpDisruption => status,
    }
}

/// Applies effects to self when possible or returns None.
///
/// Ignores ship limitations! Status might have more armor than ship layout can have.
pub fn apply_to_origin(mut status: Status, effects: &[Effect]) -> Option<Status> {
    let can_apply_all = effects.iter().all(|e| can_apply_to_origin(status, *e));
    if can_apply_all {
        for effect in effects {
            status = saturating_apply(status, *effect);
        }
        Some(status)
    } else {
        None
    }
}

/// Applies effects in a saturating way. Example: Capacitor 2 - 5 → 0
///
/// Ignores ship limitations! Status might have more armor than ship layout can have.
pub fn apply_to_target(mut status: Status, effects: &[Effect]) -> Status {
    for effect in effects {
        status = saturating_apply(status, *effect);
    }
    status
}

#[test]
fn damage_against_armor() {
    let before = Status {
        capacitor: 0,
        hitpoints_armor: 42,
        hitpoints_structure: 42,
    };
    assert_eq!(
        apply_damage(before, 10),
        Status {
            capacitor: 0,
            hitpoints_armor: 32,
            hitpoints_structure: 42,
        }
    );
}

#[test]
fn damage_against_structure() {
    let before = Status {
        capacitor: 0,
        hitpoints_armor: 0,
        hitpoints_structure: 42,
    };
    assert_eq!(
        apply_damage(before, 10),
        Status {
            capacitor: 0,
            hitpoints_armor: 0,
            hitpoints_structure: 32,
        }
    );
}

#[test]
fn damage_against_armor_and_structure() {
    let before = Status {
        capacitor: 0,
        hitpoints_armor: 3,
        hitpoints_structure: 42,
    };
    assert_eq!(
        apply_damage(before, 10),
        Status {
            capacitor: 0,
            hitpoints_armor: 0,
            hitpoints_structure: 35,
        }
    );
}

#[test]
fn damage_against_structure_min_zero() {
    let before = Status {
        capacitor: 0,
        hitpoints_armor: 0,
        hitpoints_structure: 2,
    };
    assert_eq!(
        apply_damage(before, 10),
        Status {
            capacitor: 0,
            hitpoints_armor: 0,
            hitpoints_structure: 0,
        }
    );
}

#[test]
fn module_with_cap_works_on_origin() {
    let before = Status {
        capacitor: 10,
        hitpoints_armor: 0,
        hitpoints_structure: 10,
    };
    let result = apply_to_origin(before, &[Effect::ArmorRepair(5), Effect::Capacitor(-5)]);
    assert_eq!(
        result,
        Some(Status {
            capacitor: 5,
            hitpoints_armor: 5,
            hitpoints_structure: 10,
        })
    );
}

#[test]
fn module_without_cap_doesnt_work_on_origin() {
    let before = Status {
        capacitor: 2,
        hitpoints_armor: 0,
        hitpoints_structure: 10,
    };
    let result = apply_to_origin(before, &[Effect::ArmorRepair(5), Effect::Capacitor(-5)]);
    assert_eq!(result, None);
}

#[cfg(test)]
const TEST_DEFAULT_STATUS: Status = Status {
    capacitor: 10,
    hitpoints_armor: 10,
    hitpoints_structure: 10,
};

#[test]
fn saturating_apply_reduces_capacitor() {
    let result = saturating_apply(TEST_DEFAULT_STATUS, Effect::Capacitor(-5));
    assert_eq!(
        result,
        Status {
            capacitor: 5,
            hitpoints_armor: 10,
            hitpoints_structure: 10,
        }
    );
}

#[test]
fn saturating_apply_increases_capacitor() {
    let result = saturating_apply(TEST_DEFAULT_STATUS, Effect::Capacitor(5));
    assert_eq!(
        result,
        Status {
            capacitor: 15,
            hitpoints_armor: 10,
            hitpoints_structure: 10,
        }
    );
}

#[test]
fn saturating_apply_increases_armor() {
    let result = saturating_apply(TEST_DEFAULT_STATUS, Effect::ArmorRepair(5));
    assert_eq!(
        result,
        Status {
            capacitor: 10,
            hitpoints_armor: 15,
            hitpoints_structure: 10,
        }
    );
}
