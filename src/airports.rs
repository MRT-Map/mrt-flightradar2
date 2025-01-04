use std::sync::Arc;
use air_traffic_simulator::engine::world_data::{AirportData, Runway};
use air_traffic_simulator::WorldData;
use color_eyre::Report;
use glam::Vec2;
use color_eyre::Result;

pub async fn airports(world_data: &mut WorldData) -> Result<()> {
    let client = surf::client().with(surf::middleware::Redirect::new(5));
    let string = client.send(surf::get("https://docs.google.com/spreadsheets/d/11E60uIBKs5cOSIRHLz0O0nLCefpj7HgndS1gIXY_1hw/export?format=csv&gid=0"))
        .await.map_err(|a| Report::msg(a.to_string()))?
        .body_string()
        .await.map_err(|a| Report::msg(a.to_string()))?;
    let mut reader = csv::Reader::from_reader(string.as_bytes());
    fn parse_coords(c: &str) -> Vec2 {
        let mut a = c.trim().split(' ');
        Vec2::new(a.next().and_then(|a| a.parse().ok()).unwrap(), a.next().and_then(|a| a.parse().ok()).unwrap())
    }
    world_data.airports = reader.records().map(|res| {
        let res = res?;
        if !res.get(1).unwrap().contains("Airfield") && !res.get(1).unwrap().contains("Airport") {
            return Ok(None)
        }
        let mut runways = vec![];
        for i in [3, 7, 11, 15] {
            if res.get(i).is_none_or(|a| a.is_empty()) {
                if i == 3 {
                    return Ok(None)
                }
                break
            }
            runways.push(Arc::new(Runway {
                name: res.get(i+2).unwrap().into(),
                start: parse_coords(res.get(i).unwrap()),
                end: parse_coords(res.get(i+1).unwrap()),
                altitude: 0.0,
                class: res.get(i+3).unwrap().chars().next().unwrap().to_string().into(),
            }));
            runways.push(Arc::new(Runway {
                name: res.get(i+2).unwrap().into(),
                start: parse_coords(res.get(i+1).unwrap()),
                end: parse_coords(res.get(i).unwrap()),
                altitude: 0.0,
                class: res.get(i+3).unwrap().chars().next().unwrap().to_string().into(),
            }));
        }
        Ok(Some(Arc::new(AirportData {
            name: res.get(0).unwrap().into(),
            code: res.get(0).unwrap().into(),
            runways: runways.into(),
        })))
    }).collect::<Result<Vec<_>>>()?.into_iter().flatten().collect();
    Ok(())
}