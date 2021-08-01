use std::collections::HashMap;
use std::fs;

use super::fixed::facility::{Facility, FacilityIdentifier};
use super::fixed::lifeless::Lifeless;
use super::fixed::modules::{ModulePassive, ModuleSelf, ModuleTargeted};
use super::fixed::shiplayout::ShipLayout;
use super::fixed::solarsystem::Solarsystem;

type Dict<T> = HashMap<String, T>;

type Facilites = HashMap<FacilityIdentifier, Facility>;
type LifelessThingies = Dict<Lifeless>;
type ModulesPassive = Dict<ModulePassive>;
type ModulesSelf = Dict<ModuleSelf>;
type ModulesTargeted = Dict<ModuleTargeted>;
type ShipLayouts = Dict<ShipLayout>;
type Solarsystems = Dict<Solarsystem>;

#[derive(Debug)]
pub struct Statics {
    pub facilities: Facilites,
    pub lifeless: LifelessThingies,
    pub modules_passive: ModulesPassive,
    pub modules_self: ModulesSelf,
    pub modules_targeted: ModulesTargeted,
    pub ship_layouts: ShipLayouts,
    pub solarsystems: Solarsystems,
}

pub fn parse_static() -> anyhow::Result<Statics> {
    Ok(Statics {
        facilities: import("facility")?,
        lifeless: import("lifeless")?,
        modules_passive: import("module-passive")?,
        modules_self: import("module-self")?,
        modules_targeted: import("module-targeted")?,
        ship_layouts: import("ship-layout")?,
        solarsystems: import("solarsystem")?,
    })
}

fn import<T>(filename: &str) -> anyhow::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let yaml_str = fs::read_to_string(&format!("static/{}.yaml", filename))?;
    let value = serde_yaml::from_str::<T>(&yaml_str)?;
    Ok(value)
}

#[cfg(test)]
fn export<K, V>(filename: &str, value: &HashMap<K, V>) -> anyhow::Result<()>
where
    K: serde::Serialize + std::cmp::Ord,
    V: serde::Serialize,
{
    use std::collections::BTreeMap;
    let ordered = value.iter().collect::<BTreeMap<_, _>>();

    let json_str = serde_json::to_string_pretty(&ordered)?;
    fs::write(&format!("static/{}.json", filename), json_str)?;

    let yaml_str = serde_yaml::to_string(&ordered)?;
    fs::write(&format!("static/{}.yaml", filename), yaml_str)?;

    Ok(())
}

#[test]
fn check_facility() -> anyhow::Result<()> {
    let filename = "facility";
    let all = import::<Facilites>(filename)?;
    assert!(!all.is_empty(), "is empty");
    export("facility", &all)
}

#[test]
fn check_lifeless() -> anyhow::Result<()> {
    let filename = "lifeless";
    let all = import::<LifelessThingies>(filename)?;
    assert!(!all.is_empty(), "is empty");

    for (key, value) in &all {
        assert!(key.starts_with("lifeless"), "starts wrong {}", key);
        assert!(value.hitpoints_structure > 0, "structure too low {}", key);
    }

    export(filename, &all)
}

#[test]
fn check_module_passive() -> anyhow::Result<()> {
    let filename = "module-passive";
    let all = import::<ModulesPassive>(filename)?;
    assert!(!all.is_empty(), "is empty");

    for (key, value) in &all {
        assert!(key.starts_with("modp"), "starts wrong {}", key);
        assert!(
            value.required_cpu.saturating_add(value.required_powergrid) > 0,
            "module requires nothing {}",
            key
        );
    }

    export(filename, &all)
}

#[test]
fn check_module_self() -> anyhow::Result<()> {
    let filename = "module-self";
    let all = import::<ModulesSelf>(filename)?;
    assert!(!all.is_empty(), "is empty");

    for (key, value) in &all {
        assert!(key.starts_with("mods"), "starts wrong {}", key);
        assert!(
            value.required_cpu.saturating_add(value.required_powergrid) > 0,
            "module requires nothing {}",
            key
        );
    }

    export(filename, &all)
}

#[test]
fn check_module_targeted() -> anyhow::Result<()> {
    let filename = "module-targeted";
    let all = import::<ModulesTargeted>(filename)?;
    assert!(!all.is_empty(), "is empty");

    for (key, value) in &all {
        assert!(key.starts_with("modt"), "starts wrong {}", key);
        assert!(
            value.required_cpu.saturating_add(value.required_powergrid) > 0,
            "module requires nothing {}",
            key
        );
    }

    export(filename, &all)
}

#[test]
fn check_ship_layout() -> anyhow::Result<()> {
    let filename = "ship-layout";
    let all = import::<ShipLayouts>(filename)?;
    assert!(!all.is_empty(), "is empty");

    for (key, value) in &all {
        assert!(key.starts_with("shiplayout"), "starts wrong {}", key);

        assert!(value.capacitor > 0, "capacitor {}", key);
        assert!(value.capacitor_recharge > 0, "cap recharge {}", key);
        assert!(value.hitpoints_structure > 0, "structure {}", key);
    }

    export(filename, &all)
}

#[test]
fn check_solarsystem() -> anyhow::Result<()> {
    let filename = "solarsystem";
    let all = import::<Solarsystems>(filename)?;
    assert!(!all.is_empty(), "is empty");

    for (key, value) in &all {
        assert!(key.starts_with("system"), "starts wrong {}", key);

        assert!(!value.name.is_empty(), "name {}", key);
        assert!(value.security <= 100, "security {}", key);

        for (target, planet) in &value.stargates {
            assert!(
                all.contains_key(target),
                "stargate target does not exist {} {}",
                key,
                target
            );
            assert!(planet <= &value.planets, "stargate planet {}", key);
        }

        for planet in &value.stations {
            assert!(planet <= &value.planets, "station planet {}", key);
        }
    }

    export(filename, &all)
}

#[test]
fn graphviz_solarsystems() -> anyhow::Result<()> {
    let mut text = String::new();
    text += "digraph {\n";

    let all = import::<Solarsystems>("solarsystem")?;

    let mut ordered = all.iter().collect::<Vec<_>>();
    ordered.sort_by(|a, b| b.1.security.cmp(&a.1.security));

    for (key, system) in ordered {
        text += "\t";
        text += &format!(r#"{}[label="{} ({})"];"#, key, system.name, system.security);
        text += "\n";

        for target in system.stargates.keys() {
            text += &format!("\t{} -> {};\n", key, target);
        }
    }

    text += "}\n";
    fs::write("static/solarsystems.graphviz", &text)?;
    drop(graphviz("svg", "solarsystems"));
    drop(graphviz("png", "solarsystems"));
    Ok(())
}

#[cfg(test)]
fn graphviz(format: &str, filename: &str) -> anyhow::Result<()> {
    let output = std::process::Command::new("dot")
        .arg(format!("-T{}", format))
        .arg(format!("{}.graphviz", filename))
        .current_dir("static")
        .output()?;

    if output.status.success() {
        fs::write(format!("static/{}.{}", filename, format), output.stdout)?;
        Ok(())
    } else {
        Err(anyhow::anyhow!("failed?"))
    }
}
