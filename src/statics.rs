use std::fs;

use typings::data_export::{
    Facilites, LifelessThingies, ModulesPassive, ModulesTargeted, ModulesUntargeted, ShipLayouts,
    Solarsystems,
};
#[derive(Debug)]
pub struct Statics {
    pub facilities: Facilites,
    pub lifeless: LifelessThingies,
    pub modules_passive: ModulesPassive,
    pub modules_untargeted: ModulesUntargeted,
    pub modules_targeted: ModulesTargeted,
    pub ship_layouts: ShipLayouts,
    pub solarsystems: Solarsystems,
}

impl Statics {
    pub fn import(basepath: &str) -> anyhow::Result<Self> {
        Ok(Self {
            facilities: import(basepath, "facility")?,
            lifeless: import(basepath, "lifeless")?,
            modules_passive: import(basepath, "module-passive")?,
            modules_untargeted: import(basepath, "module-untargeted")?,
            modules_targeted: import(basepath, "module-targeted")?,
            ship_layouts: import(basepath, "ship-layout")?,
            solarsystems: import(basepath, "solarsystem")?,
        })
    }
}

fn import<T>(basepath: &str, filename: &str) -> anyhow::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let yaml_str = fs::read_to_string(&format!("{}/{}.yaml", basepath, filename))?;
    let value = serde_yaml::from_str::<T>(&yaml_str)?;
    Ok(value)
}
