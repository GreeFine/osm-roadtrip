use geo::{Coord, LineString};
use geo_svg::{Color, ToSvg};

use crate::models::Highway;

pub fn draw_nodes(highways: Vec<&Highway>) -> String {
    let lines: Vec<_> = highways
        .iter()
        .map(|highway| {
            let line_coords = highway
                .nodes
                .iter()
                .map(|node| Coord {
                    x: node.latitude,
                    y: node.longitude,
                })
                .collect();
            LineString::new(line_coords)
        })
        .collect();

    let svg = lines
        .to_svg()
        .with_fill_opacity(0f32)
        .with_stroke_opacity(1f32)
        .with_stroke_width(5.)
        .with_stroke_color(Color::Named("green"));

    svg.to_string()
}
