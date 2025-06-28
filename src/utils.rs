use std::sync::LazyLock;

use color_eyre::{Report, Result, eyre::eyre};
use glam::Vec2;
use serde::{Deserialize, Deserializer};

pub fn parse_coords(c: &str) -> Result<Vec2> {
    let a = c.trim().split(' ').collect::<Vec<_>>();
    if a.len() == 2 {
        Ok(Vec2::new(a[0].parse()?, a[1].parse()?))
    } else if a.len() == 3 {
        Ok(Vec2::new(a[0].parse()?, a[2].parse()?))
    } else {
        Err(eyre!("{c} is not a valid coordinate"))
    }
}

pub fn deserialize_coords<'de, D: Deserializer<'de>>(de: D) -> Result<Option<Vec2>, D::Error> {
    let s = <&str>::deserialize(de)?;
    if s.trim().is_empty() {
        return Ok(None);
    }
    match parse_coords(s) {
        Ok(v) => Ok(Some(v)),
        Err(e) => Err(serde::de::Error::custom(e.to_string())),
    }
}

pub static SURF_CLIENT: LazyLock<surf::Client> =
    LazyLock::new(|| surf::client().with(surf::middleware::Redirect::new(5)));
pub async fn get_url(url: &'static str) -> Result<String> {
    SURF_CLIENT
        .send(surf::get(url))
        .await
        .map_err(|a| Report::msg(a.to_string()))?
        .body_string()
        .await
        .map_err(|a| Report::msg(a.to_string()))
}
