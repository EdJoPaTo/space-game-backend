use typings::fixed::shiplayout::{ShipQualities, ShipQuality};
use typings::persist::ship::Status;

pub fn apply_round(mut status: Status, qualities: &ShipQualities) -> Status {
    for (q, amount) in qualities {
        match q {
            ShipQuality::HitpointsArmor
            | ShipQuality::HitpointsStructure
            | ShipQuality::Capacitor => { /* passive */ }
            ShipQuality::CapacitorRecharge => {
                status.capacitor = add(status.capacitor, *amount);
            }
        }
    }
    status
}

#[allow(clippy::cast_sign_loss)]
const fn add(base: u16, add: i16) -> u16 {
    if add >= 0 {
        base.saturating_add(add as u16)
    } else {
        let b = add.saturating_abs() as u16;
        base.saturating_sub(b)
    }
}

#[test]
fn passive_is_ignored() {
    let before = Status {
        capacitor: 10,
        hitpoints_armor: 10,
        hitpoints_structure: 10,
    };
    let mut qualities = ShipQualities::new();
    qualities.insert(ShipQuality::Capacitor, 666);
    let result = apply_round(before, &qualities);
    assert_eq!(
        result,
        Status {
            capacitor: 10,
            hitpoints_armor: 10,
            hitpoints_structure: 10,
        }
    );
}

#[test]
fn refills_cap() {
    let before = Status {
        capacitor: 10,
        hitpoints_armor: 10,
        hitpoints_structure: 10,
    };
    let mut qualities = ShipQualities::new();
    qualities.insert(ShipQuality::CapacitorRecharge, 2);
    let result = apply_round(before, &qualities);
    assert_eq!(
        result,
        Status {
            capacitor: 12,
            hitpoints_armor: 10,
            hitpoints_structure: 10,
        }
    );
}
