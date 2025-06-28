use std::sync::Arc;

use air_traffic_simulator::{
    WorldData,
    engine::world_data::{AirportData, Runway},
};
use color_eyre::Result;
use glam::Vec2;
use serde::Deserialize;
use smol_str::SmolStr;

use crate::utils::{deserialize_coords, get_url};

#[derive(Clone, Debug, Deserialize)]
struct CSVAirport {
    code: SmolStr,
    name: SmolStr,
    world: SmolStr,
    r1_alt: Option<f32>,
    #[serde(deserialize_with = "deserialize_coords")]
    r1_pos1: Option<Vec2>,
    #[serde(deserialize_with = "deserialize_coords")]
    r1_pos2: Option<Vec2>,
    r1_dir1: Option<SmolStr>,
    r1_dir2: Option<SmolStr>,
    r1_size: Option<SmolStr>,
    r2_alt: Option<f32>,
    #[serde(deserialize_with = "deserialize_coords")]
    r2_pos1: Option<Vec2>,
    #[serde(deserialize_with = "deserialize_coords")]
    r2_pos2: Option<Vec2>,
    r2_dir1: Option<SmolStr>,
    r2_dir2: Option<SmolStr>,
    r2_size: Option<SmolStr>,
    r3_alt: Option<f32>,
    #[serde(deserialize_with = "deserialize_coords")]
    r3_pos1: Option<Vec2>,
    #[serde(deserialize_with = "deserialize_coords")]
    r3_pos2: Option<Vec2>,
    r3_dir1: Option<SmolStr>,
    r3_dir2: Option<SmolStr>,
    r3_size: Option<SmolStr>,
    r4_alt: Option<f32>,
    #[serde(deserialize_with = "deserialize_coords")]
    r4_pos1: Option<Vec2>,
    #[serde(deserialize_with = "deserialize_coords")]
    r4_pos2: Option<Vec2>,
    r4_dir1: Option<SmolStr>,
    r4_dir2: Option<SmolStr>,
    r4_size: Option<SmolStr>,
}

pub async fn airports(world_data: &mut WorldData) -> Result<()> {
    let string = get_url("https://docs.google.com/spreadsheets/d/126q9vTD9tNZTEb13TnHhK7M8-06kVSaeb16zJXx_9YM/export?format=csv&gid=0").await?;
    let mut reader = csv::Reader::from_reader(string.as_bytes());

    world_data.airports = reader
        .deserialize()
        .map(|res| {
            let res: CSVAirport = res?;
            if res.world == "Old" || res.code.is_empty() {
                return Ok(None);
            }

            let mut runways = vec![];
            for (alt, pos1, pos2, dir1, dir2, size) in [
                (
                    res.r1_alt,
                    res.r1_pos1,
                    res.r1_pos2,
                    res.r1_dir1,
                    res.r1_dir2,
                    res.r1_size,
                ),
                (
                    res.r2_alt,
                    res.r2_pos1,
                    res.r2_pos2,
                    res.r2_dir1,
                    res.r2_dir2,
                    res.r2_size,
                ),
                (
                    res.r3_alt,
                    res.r3_pos1,
                    res.r3_pos2,
                    res.r3_dir1,
                    res.r3_dir2,
                    res.r3_size,
                ),
                (
                    res.r4_alt,
                    res.r4_pos1,
                    res.r4_pos2,
                    res.r4_dir1,
                    res.r4_dir2,
                    res.r4_size,
                ),
            ] {
                let Some((pos1, pos2)) = pos1.and_then(|a| Some((a, pos2?))) else {
                    break;
                };
                let class = match size.as_deref() {
                    Some("XSmall") => "XS",
                    Some("Small") => "S",
                    Some("Medium") => "M",
                    _ => "L",
                };
                runways.push(Arc::new(Runway {
                    name: dir1.unwrap_or_default(),
                    start: pos1,
                    end: pos2,
                    altitude: alt.unwrap_or_default() - 63.0,
                    class: class.into(),
                }));
                runways.push(Arc::new(Runway {
                    name: dir2.unwrap_or_default(),
                    start: pos2,
                    end: pos1,
                    altitude: alt.unwrap_or_default() - 63.0,
                    class: class.into(),
                }));
            }
            if runways.is_empty() {
                return Ok(None);
            }
            Ok(Some(Arc::new(AirportData {
                name: res.name,
                code: res.code,
                runways: runways.into(),
            })))
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect();
    Ok(())
}
