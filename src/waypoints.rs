use std::sync::Arc;

use air_traffic_simulator::{engine::world_data::Waypoint, WorldData};
use color_eyre::{eyre::eyre, Report, Result};
use gatelogue_types::GatelogueData;
use glam::{DVec2, Vec2};
use itertools::Itertools;
use rand::prelude::*;
use smol_str::SmolStr;

use crate::utils::{get_url, parse_coords};

struct WaypointNameGenerator(Vec<SmolStr>, StdRng);
impl WaypointNameGenerator {
    fn new(seed: u64) -> Self {
        WaypointNameGenerator(vec![], StdRng::seed_from_u64(seed))
    }
}

pub const CONSONANTS: &str = "BBCCCDDDFFGGHHJKKLLLMMMNNNPPQRRRSSSTTTVVWWXYZZ";
pub const VOWELS: &str = "AAEEIIOOUUY";

impl Iterator for WaypointNameGenerator {
    type Item = SmolStr;

    fn next(&mut self) -> Option<Self::Item> {
        let mut new = SmolStr::default();
        while new.is_empty() || self.0.contains(&new) {
            new = format!(
                "{}{}{}{}{}",
                CONSONANTS.chars().choose(&mut thread_rng()).unwrap(),
                VOWELS.chars().choose(&mut self.1).unwrap(),
                CONSONANTS.chars().choose(&mut thread_rng()).unwrap(),
                VOWELS.chars().choose(&mut self.1).unwrap(),
                CONSONANTS.chars().choose(&mut thread_rng()).unwrap(),
            )
            .into();
            if new.contains("YY") {
                new = "".into();
            }
        }
        Some(new)
    }
}

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
    let data = GatelogueData::surf_get_no_sources()
        .await
        .map_err(|e| eyre!("{e}"))?;
    let mut gen = WaypointNameGenerator::new(0);

    let mut waypoints: Vec<(SmolStr, Vec2, Vec<SmolStr>)> = data
        .nodes
        .values()
        .filter_map(|a| {
            a.as_town()
                .and_then(|a| a.common.coordinates.to_owned())
                .or_else(|| {
                    a.as_air_airport()
                        .and_then(|a| a.common.coordinates.to_owned())
                })
        })
        .map(|c| (gen.next().unwrap(), { DVec2::from(*c).as_vec2() }, vec![]))
        .collect();

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
