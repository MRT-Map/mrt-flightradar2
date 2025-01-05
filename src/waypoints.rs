use std::sync::Arc;

use air_traffic_simulator::{WorldData, engine::world_data::Waypoint};
use color_eyre::Result;
use glam::Vec2;
use itertools::Itertools;
use smol_str::SmolStr;

use crate::utils::{get_url, parse_coords};

fn nearest_waypoints(waypoints: &[(SmolStr, Vec2, Vec<SmolStr>)], wp: Vec2) -> Vec<SmolStr> {
    let mut radius = 0.0;
    let mut nearest = vec![];
    while nearest.len() < 3 {
        radius += 1000.0;
        nearest = waypoints
            .iter()
            .filter(|(_, w, _)| *w != wp)
            .filter(|(_, w, _)| w.distance(wp) < radius)
            .map(|(a, _, _)| a.to_owned())
            .collect();
    }
    nearest
}

pub async fn waypoints(world_data: &mut WorldData) -> Result<()> {
    let string = get_url("https://docs.google.com/spreadsheets/d/11E60uIBKs5cOSIRHLz0O0nLCefpj7HgndS1gIXY_1hw/export?format=csv&gid=707730663").await?;
    let mut reader = csv::Reader::from_reader(string.as_bytes());

    let mut waypoints = reader
        .records()
        .filter_map(|res| {
            let res = res.unwrap();
            if res.get(0).unwrap().starts_with("AA") {
                return None;
            }
            Some((
                res.get(0).unwrap().into(),
                parse_coords(res.get(1).unwrap()),
                vec![],
            ))
        })
        .collect::<Vec<_>>();

    let mut airways = vec![];
    for (name, coords, _) in &waypoints {
        for nw in nearest_waypoints(&waypoints, *coords) {
            airways.push((name.to_owned(), nw.to_owned()))
        }
    }
    for (name, _, conns) in &mut waypoints {
        *conns = airways
            .iter()
            .filter_map(|(a, b)| {
                if *a == *name {
                    Some(b.to_owned())
                } else if *b == *name {
                    Some(a.to_owned())
                } else {
                    None
                }
            })
            .sorted()
            .dedup()
            .collect()
    }
    world_data.waypoints = waypoints
        .into_iter()
        .map(|(name, coords, conns)| {
            Arc::new(Waypoint {
                name,
                pos: coords,
                connections: conns.into(),
            })
        })
        .collect();
    Ok(())
}
