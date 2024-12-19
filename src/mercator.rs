use std::f64::consts::PI;

use crate::models::HighwayNode;

// Earth's radius in meters (mean radius)
const EARTH_RADIUS: f64 = 6378137.0;

/// Transforms latitude and longitude to Mercator projection
pub fn highway_lat_lon_mercator(highway_node: &HighwayNode) -> (f64, f64) {
    // Convert latitude and longitude from degrees to radians
    let lat_rad = highway_node.latitude.degrees().to_radians();
    let lon_rad = highway_node.longitude.degrees().to_radians();

    // Apply Mercator projection formulas
    let x = EARTH_RADIUS * lon_rad;
    let y = EARTH_RADIUS * ((PI / 4.0) + (lat_rad / 2f64)).tan().ln();

    (x, y)
}
