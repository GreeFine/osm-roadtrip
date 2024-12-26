use geo_svg::{Color, ToSvg};
use geo_types::{Coord, LineString};

use crate::{mercator::highway_lat_lon_mercator, models::Highway};

pub fn draw_nodes(highways: Vec<&Highway>) -> String {
    // let points: Vec<_> = highways_nodes
    //     .iter()
    //     .flat_map(|highway_nodes| {
    //         highway_nodes
    //             .clone()
    //             .into_iter()
    //             .map(highway_lat_lon_mercator)
    //             .map(|(x, y)| Point::new(x, y))
    //     })
    //     .collect();

    // let texts: Vec<_> = highways_nodes
    //     .iter()
    //     .flat_map(|highway_nodes| {
    //         highway_nodes
    //             .iter()
    //             .map(|node| highway_lat_lon_mercator(node))
    //             .map(|(x, y)| Coord { x, y })
    //             .enumerate()
    //             .map(|(idx, coord)| Text::new(idx.to_string(), coord))
    //     })
    //     .collect();

    let lines: Vec<_> = highways
        .iter()
        .map(|highway| {
            let line_coords = highway
                .nodes
                .iter()
                .map(|node| highway_lat_lon_mercator(node))
                .map(|(x, y)| Coord { x, y })
                .collect();
            LineString::new(line_coords)
        })
        .collect();

    let svg =
    //  points
    //     .to_svg()
    //     .with_radius(1.0)
    //     .with_fill_color(Color::Named("red"))
    //     .with_fill_opacity(0.7)
        // .and(texts.to_svg())
        // .and(
            lines
                .to_svg()
                .with_fill_opacity(0f32)
                .with_stroke_opacity(1f32)
                .with_stroke_color(Color::Named("green"));
    // );

    svg.to_string()
}
