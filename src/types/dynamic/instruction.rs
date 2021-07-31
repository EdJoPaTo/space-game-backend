use serde::{Deserialize, Serialize};
use ts_rs::{export, TS};

use crate::types::fixed::facility::Service;

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum Instruction {
    ModuleSelf(InstructionModuleSelf),
    ModuleTargeted(InstructionModuleTargeted),
    UseFacility(InstructionUseFacility),
    Warp(InstructionWarp),
    Undock(InstructionUndock),
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct InstructionModuleSelf {
    pub module_index: u8,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct InstructionModuleTargeted {
    pub target_id_in_site: u8,
    pub module_index: u8,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct InstructionUseFacility {
    pub target_id_in_site: u8,
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
    Instruction => "instruction.ts",
    InstructionModuleSelf => "instruction-module-self.ts",
    InstructionModuleTargeted => "instruction-module-targeted.ts",
    InstructionUseFacility => "instruction-use-facility.ts",
    InstructionWarp => "instruction-warp.ts",
    InstructionUndock => "instruction-undock.ts",
}
