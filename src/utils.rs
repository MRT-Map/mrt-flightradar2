use std::cell::LazyCell;

use color_eyre::{Report, Result};
use glam::Vec2;

pub fn parse_coords(c: &str) -> Vec2 {
    let mut a = c.trim().split(' ');
    Vec2::new(
        a.next().and_then(|a| a.parse().ok()).unwrap(),
        a.next().and_then(|a| a.parse().ok()).unwrap(),
    )
}

pub static SURF_CLIENT: LazyCell<surf::Client> =
    LazyCell::new(|| surf::client().with(surf::middleware::Redirect::new(5)));
pub async fn get_url(url: &'static str) -> Result<String> {
    SURF_CLIENT
        .send(surf::get(url))
        .await
        .map_err(|a| Report::msg(a.to_string()))?
        .body_string()
        .await
        .map_err(|a| Report::msg(a.to_string()))
}
