use serde::{Deserialize, Serialize};
use ts_rs::{export, TS};

use crate::types::fixed::facility::Service;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Instruction {
    Undock(InstructionUndock),
    Warp(InstructionWarp),
    UseFacility(InstructionUseFacility),
    ModuleTargeted(InstructionModuleTargeted),
    ModuleSelf(InstructionModuleSelf),
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct InstructionModuleSelf {
    pub module_index: u8,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct InstructionModuleTargeted {
    pub target_index_in_site: u8,
    pub module_index: u8,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct InstructionUseFacility {
    pub target_index_in_site: u8,
    pub service: Service,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct InstructionWarp {
    pub site_unique: String,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct InstructionUndock {
    ship_id: u8,
}

export! {
    // Instruction => "instruction.ts",
    InstructionModuleSelf => "instruction-module-self.ts",
    InstructionModuleTargeted => "instruction-module-targeted.ts",
    InstructionUseFacility => "instruction-use-facility.ts",
    InstructionWarp => "instruction-warp.ts",
    InstructionUndock => "instruction-undock.ts",
}

#[test]
fn can_identify_undock() -> anyhow::Result<()> {
    let data = Instruction::Undock(InstructionUndock { ship_id: 42 });
    let json = serde_json::to_string_pretty(&data)?;
    println!("json {}", json);
    let some = serde_json::from_str::<Instruction>(&json)?;
    if let Instruction::Undock(_) = some {
        Ok(())
    } else {
        panic!("wrong!");
    }
}

#[test]
fn can_identify_warp() -> anyhow::Result<()> {
    let data = Instruction::Warp(InstructionWarp {
        site_unique: "666".to_string(),
    });
    let json = serde_json::to_string_pretty(&data)?;
    println!("json {}", json);
    let some = serde_json::from_str::<Instruction>(&json)?;
    if let Instruction::Warp(_) = some {
        Ok(())
    } else {
        panic!("wrong!");
    }
}

#[test]
fn can_identify_facility() -> anyhow::Result<()> {
    let data = Instruction::UseFacility(InstructionUseFacility {
        target_index_in_site: 42,
        service: Service::Dock,
    });
    let json = serde_json::to_string_pretty(&data)?;
    println!("json {}", json);
    let some = serde_json::from_str::<Instruction>(&json)?;
    if let Instruction::UseFacility(_) = some {
        Ok(())
    } else {
        panic!("wrong!");
    }
}

#[test]
fn can_identify_module_self() -> anyhow::Result<()> {
    let data = Instruction::ModuleSelf(InstructionModuleSelf { module_index: 4 });
    let json = serde_json::to_string_pretty(&data)?;
    println!("json {}", json);
    let some = serde_json::from_str::<Instruction>(&json)?;
    if let Instruction::ModuleSelf(_) = some {
        Ok(())
    } else {
        panic!("wrong!");
    }
}

#[test]
fn can_identify_module_targeted() -> anyhow::Result<()> {
    let data = Instruction::ModuleTargeted(InstructionModuleTargeted {
        target_index_in_site: 42,
        module_index: 4,
    });
    let json = serde_json::to_string_pretty(&data)?;
    println!("json {}", json);
    let some = serde_json::from_str::<Instruction>(&json)?;
    if let Instruction::ModuleTargeted(_) = some {
        Ok(())
    } else {
        panic!("wrong!");
    }
}
