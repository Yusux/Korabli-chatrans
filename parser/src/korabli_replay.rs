use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(non_snake_case)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VehicleInfoMeta {
    pub shipId: u64,
    pub relation: u32,
    pub id: i64, // Account ID?
    pub name: String,
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReplayMeta {
    pub matchGroup: String,
    pub gameMode: u32,
    pub clientVersionFromExe: String,
    pub scenarioUiCategoryId: u32,
    pub mapDisplayName: String,
    pub mapId: u32,
    pub clientVersionFromXml: String,
    pub weatherParams: HashMap<String, Vec<String>>,
    pub duration: u32,
    pub name: String,
    pub scenario: String,
    pub playerID: u32,
    pub vehicles: Vec<VehicleInfoMeta>,
    pub playersPerTeam: u32,
    pub dateTime: String,
    pub mapName: String,
    pub playerName: String,
    pub scenarioConfigId: u32,
    pub teamsCount: u32,
    pub playerVehicle: String,
    pub battleDuration: u32,
}
