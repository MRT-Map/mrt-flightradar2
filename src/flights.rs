use std::sync::Arc;

use air_traffic_simulator::{engine::world_data::Flight, WorldData};
use color_eyre::{eyre::OptionExt, Result};
use gatelogue_types::{AirMode, GatelogueData};

pub async fn flights(world_data: &mut WorldData, gatelogue_data: &GatelogueData) -> Result<()> {
    world_data.flights = Some(
        gatelogue_data
            .air_flights()
            .map(|a| {
                if a.gates.len() < 2 || a.mode.as_ref().is_some_and(|a| **a != AirMode::WarpPlane) {
                    return Ok(None);
                }
                Ok(Some(Arc::new(Flight {
                    airline: gatelogue_data
                        .get_air_airline(*a.airline)?
                        .name
                        .to_owned()
                        .into(),
                    code: a.codes.first().ok_or_eyre("No code")?.into(),
                    from: {
                        let gate = gatelogue_data
                            .get_air_gate(**a.gates.first().ok_or_eyre("No from")?)?;
                        let code = gatelogue_data
                            .get_air_airport(*gate.airport)?
                            .code
                            .to_owned()
                            .into();
                        if !world_data.airports.iter().any(|a| a.code == code) {
                            return Ok(None);
                        };
                        code
                    },
                    to: {
                        let gate =
                            gatelogue_data.get_air_gate(**a.gates.get(1).ok_or_eyre("No to")?)?;
                        let code = gatelogue_data
                            .get_air_airport(*gate.airport)?
                            .code
                            .to_owned()
                            .into();
                        if !world_data.airports.iter().any(|a| a.code == code) {
                            return Ok(None);
                        };
                        code
                    },
                    plane: ["Airplane".into()].into(),
                })))
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect(),
    );
    Ok(())
}
