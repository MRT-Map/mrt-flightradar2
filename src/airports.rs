use std::sync::Arc;

use air_traffic_simulator::{
    WorldData,
    engine::world_data::{AirportData, Runway},
};
use color_eyre::Result;

use crate::utils::{get_url, parse_coords};

pub async fn airports(world_data: &mut WorldData) -> Result<()> {
    let string = get_url("https://docs.google.com/spreadsheets/d/11E60uIBKs5cOSIRHLz0O0nLCefpj7HgndS1gIXY_1hw/export?format=csv&gid=0").await?;
    let mut reader = csv::Reader::from_reader(string.as_bytes());

    world_data.airports = reader
        .records()
        .map(|res| {
            let res = res?;
            if !res.get(1).unwrap().contains("Airfield") && !res.get(1).unwrap().contains("Airport")
            {
                return Ok(None);
            }
            let mut runways = vec![];
            for i in [3, 7, 11, 15] {
                if res.get(i).is_none_or(|a| a.is_empty()) {
                    if i == 3 {
                        return Ok(None);
                    }
                    break;
                }
                runways.push(Arc::new(Runway {
                    name: res.get(i + 2).unwrap().into(),
                    start: parse_coords(res.get(i).unwrap()),
                    end: parse_coords(res.get(i + 1).unwrap()),
                    altitude: 0.0,
                    class: res
                        .get(i + 3)
                        .unwrap()
                        .chars()
                        .next()
                        .unwrap()
                        .to_string()
                        .into(),
                }));
                runways.push(Arc::new(Runway {
                    name: res.get(i + 2).unwrap().into(),
                    start: parse_coords(res.get(i + 1).unwrap()),
                    end: parse_coords(res.get(i).unwrap()),
                    altitude: 0.0,
                    class: res
                        .get(i + 3)
                        .unwrap()
                        .chars()
                        .next()
                        .unwrap()
                        .to_string()
                        .into(),
                }));
            }
            Ok(Some(Arc::new(AirportData {
                name: res.get(0).unwrap().into(),
                code: res.get(0).unwrap().into(),
                runways: runways.into(),
            })))
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect();
    Ok(())
}
