use std::sync::Arc;

use air_traffic_simulator::{WorldData, engine::world_data::Flight};
use color_eyre::Result;
use gatelogue_types::{AirFlight, AirMode, GD};

pub fn flights(world_data: &mut WorldData, gd: &GD) -> Result<()> {
    world_data.flights = Some(
        gd.nodes_of_type::<AirFlight>()?
            .into_iter()
            .map(|af| {
                if let Some(ac) = af.aircraft(gd)?
                    && ![AirMode::WarpPlane, AirMode::TrainCartsPlane].contains(&ac.mode(gd)?)
                {
                    return Ok(None);
                }
                Ok(Some(Arc::new(Flight {
                    airline: af.airline(gd)?.name(gd)?.into(),
                    code: af.code(gd)?.into(),
                    from: {
                        let code = af.from(gd)?.airport(gd)?.code(gd)?;
                        if !world_data.airports.iter().any(|a| a.code == code) {
                            return Ok(None);
                        }
                        code.into()
                    },
                    to: {
                        let code = af.to(gd)?.airport(gd)?.code(gd)?;
                        if !world_data.airports.iter().any(|a| a.code == code) {
                            return Ok(None);
                        }
                        code.into()
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
