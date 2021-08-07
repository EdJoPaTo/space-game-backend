use typings::fixed::Statics;
use typings::persist::ship::{Fitting, Status};

pub fn calc_max(statics: &Statics, fitting: &Fitting) -> anyhow::Result<Status> {
    let layout = statics
        .ship_layouts
        .get(&fitting.layout)
        .ok_or_else(|| anyhow::anyhow!("statics dont contain layout {}", fitting.layout))?;

    let mut status = Status {
        capacitor: layout.capacitor,
        hitpoints_armor: layout.hitpoints_armor,
        hitpoints_structure: layout.hitpoints_structure,
    };

    for passive_identifier in &fitting.slots_passive {
        if let Some(module) = statics.modules_passive.get(passive_identifier) {
            if let Some(armor) = module.hitpoints_armor {
                status.hitpoints_armor = status.hitpoints_armor.saturating_add(armor);
            }
            if let Some(capacitor) = module.capacitor {
                status.capacitor = status.capacitor.saturating_add(capacitor);
            }
        }
    }

    Ok(status)
}

#[test]
fn without_modules_works() -> anyhow::Result<()> {
    let statics = Statics::default();
    let expected = statics.ship_layouts.get("shiplayoutFrigate").unwrap();
    let fitting = Fitting {
        layout: "shiplayoutFrigate".to_string(),
        slots_targeted: vec![],
        slots_untargeted: vec![],
        slots_passive: vec![],
    };
    let result = calc_max(&statics, &fitting)?;
    assert_eq!(
        result,
        Status {
            capacitor: expected.capacitor,
            hitpoints_armor: expected.hitpoints_armor,
            hitpoints_structure: expected.hitpoints_structure,
        }
    );
    Ok(())
}

#[test]
fn default_fitting_works() -> anyhow::Result<()> {
    let statics = Statics::default();
    let fitting = Fitting::default();
    let expected_layout = statics.ship_layouts.get(&fitting.layout).unwrap();
    let expected_passive = statics
        .modules_passive
        .get(&fitting.slots_passive[0])
        .unwrap();
    let result = calc_max(&statics, &fitting)?;
    assert_eq!(
        result,
        Status {
            capacitor: expected_layout.capacitor,
            hitpoints_armor: expected_layout.hitpoints_armor
                + expected_passive.hitpoints_armor.unwrap(),
            hitpoints_structure: expected_layout.hitpoints_structure,
        }
    );
    Ok(())
}
