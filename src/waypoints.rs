use std::{
    hash::{DefaultHasher, Hash, Hasher},
    iter::Iterator,
    sync::{Arc, LazyLock},
};

use air_traffic_simulator::{engine::world_data::Waypoint, WorldData};
use color_eyre::Result;
use gatelogue_types::GatelogueData;
use glam::{DVec2, Vec2};
use itertools::Itertools;
use rand::prelude::*;
use smol_str::SmolStr;

struct WaypointNameGenerator(Vec<SmolStr>);
impl WaypointNameGenerator {
    fn new() -> Self {
        WaypointNameGenerator(vec![])
    }
}

pub const CONSONANTS: &str = "BBCCCDDDFFGGHHJKKLLLMMMNNNPPQRRRSSSTTTVVWWXYZZ";
pub const VOWELS: &str = "AAEEIIOOUUY";
pub const DIPHTHONGS1: [&str; 127] = [
    "BH", "BL", "BR", "BW", "BY", "CH", "CL", "CR", "CY", "CZ", "DH", "DJ", "DR", "DS", "DW", "DY",
    "DZ", "FL", "FR", "FT", "FW", "FY", "GH", "GL", "GN", "GR", "GW", "GY", "HJ", "HM", "HN", "HR",
    "HW", "HY", "JH", "JR", "JW", "JY", "KH", "KJ", "KL", "KM", "KN", "KR", "KS", "KV", "KW", "KY",
    "LH", "LR", "LW", "LY", "MB", "MH", "ML", "MN", "MP", "MR", "MS", "MW", "MY", "ND", "NG", "NH",
    "NR", "NW", "NY", "PF", "PH", "PL", "PR", "PS", "PT", "PW", "PY", "QH", "QL", "QM", "QN", "QR",
    "QU", "QV", "QW", "QY", "RH", "RW", "RY", "SB", "SC", "SD", "SG", "SH", "SJ", "SK", "SL", "SM",
    "SN", "SP", "SQ", "SR", "ST", "SV", "SW", "SY", "SZ", "TH", "TR", "TS", "TW", "TY", "TZ", "VH",
    "VL", "VR", "VW", "VY", "WH", "WR", "XR", "XW", "XY", "ZH", "ZL", "ZR", "ZS", "ZW", "ZY",
];
pub static DIPHTHONGS2: LazyLock<Vec<String>> = LazyLock::new(|| {
    CONSONANTS
        .chars()
        .flat_map(|c1| {
            "HSTZ"
                .chars()
                .map(move |c2| format!("{c1}{c2}"))
                .chain([format!("{c1}{c1}")])
        })
        .chain(
            [
                "CK", "RN", "LN", "GN", "NG", "MB", "MP", "ND", "NT", "NK", "NQ",
            ]
            .into_iter()
            .map(Into::into),
        )
        .collect()
});
pub static DIPHTHONGS3: LazyLock<Vec<String>> = LazyLock::new(|| {
    CONSONANTS
        .chars()
        .flat_map(|c1| "NLR".chars().map(move |c2| format!("{c1}{c2}")))
        .collect()
});
pub static WAYPOINT_NAMINGS: [&str; 18] = [
    "CVCVC", "CVVCV", "CVCVV", "VVCVC", "VCVVC", "VVCVV", "DVCV", "VCVE", "VCCVV", "VVCCV",
    "VCCVC", "CVCCV", "VVVF", "DVF", "CVVF", "CVCF", "VVCF", "VEF",
];

impl WaypointNameGenerator {
    fn gen(&mut self, rng: &mut StdRng) -> SmolStr {
        let mut new = SmolStr::default();
        while new.is_empty() || self.0.contains(&new) {
            let naming = WAYPOINT_NAMINGS.choose(rng).unwrap();
            let mut new2 = String::new();
            for char in naming.chars() {
                match char {
                    'C' => new2.push(CONSONANTS.chars().choose(rng).unwrap()),
                    'V' => new2.push(VOWELS.chars().choose(rng).unwrap()),
                    'D' => new2.push_str(DIPHTHONGS1.choose(rng).unwrap()),
                    'E' => new2.push_str(DIPHTHONGS2.choose(rng).unwrap()),
                    'F' => new2.push_str(DIPHTHONGS3.choose(rng).unwrap()),
                    _ => unreachable!(),
                }
            }
            if new2.contains("YY") {
                new2 = "".into();
            }
            new = new2.into();
        }
        new
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

pub async fn waypoints(world_data: &mut WorldData, gatelogue_data: &GatelogueData) -> Result<()> {
    let mut gen = WaypointNameGenerator::new();
    let mut waypoints: Vec<(SmolStr, Vec2, Vec<SmolStr>)> = gatelogue_data
        .air_airports()
        .filter_map(|a| a.common.coordinates.to_owned())
        .chain(
            gatelogue_data
                .towns()
                .filter_map(|a| a.common.coordinates.to_owned()),
        )
        .map(|c| {
            (
                {
                    let mut s = DefaultHasher::new();
                    c.0.to_le_bytes().hash(&mut s);
                    c.1.to_le_bytes().hash(&mut s);
                    gen.gen(&mut StdRng::seed_from_u64(s.finish()))
                },
                { DVec2::from(*c).as_vec2() },
                vec![],
            )
        })
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
