use std::sync::Arc;
use air_traffic_simulator::engine::world_data::Waypoint;
use air_traffic_simulator::WorldData;
use color_eyre::Report;
use glam::Vec2;
use color_eyre::Result;
use itertools::Itertools;

fn nearest_waypoints(waypoints: &[(String, Vec2, Vec<&String>)], wp: Vec2) -> Vec<String> {
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
    let client = surf::client().with(surf::middleware::Redirect::new(5));
    let string = client.send(surf::get("https://docs.google.com/spreadsheets/d/11E60uIBKs5cOSIRHLz0O0nLCefpj7HgndS1gIXY_1hw/export?format=csv&gid=707730663"))
        .await.map_err(|a| Report::msg(a.to_string()))?
        .body_string()
        .await.map_err(|a| Report::msg(a.to_string()))?;
    let mut reader = csv::Reader::from_reader(string.as_bytes());
    fn parse_coords(c: &str) -> Vec2 {
        let mut a = c.trim().split(' ');
        Vec2::new(a.next().and_then(|a| a.parse().ok()).unwrap(), a.next().and_then(|a| a.parse().ok()).unwrap())
    }

    let mut waypoints = reader.records().map(|res| {
        let res = res.unwrap();
        (res.get(0).unwrap().into(), parse_coords(res.get(1).unwrap()), vec![])
    }).collect::<Vec<_>>();

    let mut airways = vec![];
    for (name, coords, _) in &waypoints {
        for nw in nearest_waypoints(&waypoints, *coords) {
            airways.push((name.to_owned(), nw.to_owned()))
        }
    }
    for (name, _, conns) in &mut waypoints {
        *conns = airways.iter().filter_map(|(a, b)| {
            if *a == *name {Some(b)} else if *b == *name {Some(a)} else { None }
        }).sorted().dedup().collect()
    }
    world_data.waypoints = waypoints.into_iter().map(|(name, coords, conns)| Arc::new(Waypoint {
        name: name.into(),
        pos: coords,
        connections: conns.into_iter().map(Into::into).collect()
    })).collect();
    Ok(())
}