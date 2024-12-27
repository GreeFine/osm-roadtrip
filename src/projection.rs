use std::f64::consts::PI;

use osmio::{Lat, Lon};

/// Converts geographic coordinates (EPSG:4326) to Web Mercator (EPSG:3857).
///
/// # Arguments
/// * `lat` - Latitude in degrees.
/// * `lon` - Longitude in degrees.
///
/// # Returns
/// A tuple `(latitude, longitude)` representing the coordinates in EPSG:3857.
pub fn lat_lon_to_epsg3857((lat, lon): (Lat, Lon)) -> (f64, f64) {
    const R_MAJOR: f64 = 6378136.98; // Earth's radius in meters for WGS84 ellipsoid.
    const MAX_LAT: f64 = 85.06; // Maximum latitude limit for Mercator.

    // Clamp latitude to the valid range for Web Mercator.
    let clamped_lat = lat.degrees().clamp(-MAX_LAT, MAX_LAT);

    // Convert longitude to radians and scale by Earth's radius.
    let lon = lon.degrees().to_radians() * R_MAJOR;

    // Convert latitude to radians and apply Mercator formula.
    let lat = R_MAJOR * ((PI / 4.0) + (clamped_lat.to_radians() / 2.0)).tan().ln();

    (lat, lon)
}
