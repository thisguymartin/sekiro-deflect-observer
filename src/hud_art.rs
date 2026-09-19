//! Original vector strike emblem, shared by live drawing and the SVG export.
//! Coordinates are centered on the gate; the art fits inside +/- 40 units.
use std::fmt::Write;

pub struct Stroke {
    pub points: Vec<[f32; 2]>,
    pub width: f32,
}

/// A true crescent, triangulated so the open side never fills in. Stronger
/// slowdown gives a wider moon: 90% is slim, 80% wider, and 70% or lower fullest.
pub fn practice_moon_triangles(factor: f32) -> Vec<[[f32; 2]; 3]> {
    let offset = ((1.0 - factor) * 45.0).clamp(3.0, 12.0);
    let tip_y = 22.0 * 60.0_f32.to_radians().sin();
    let inner_radius = ((11.0 - offset).powi(2) + tip_y.powi(2)).sqrt();
    let inner_start = (-tip_y).atan2(11.0 - offset);
    let edge = |i: usize| {
        let t = i as f32 / 32.0;
        let outer = (-60.0 - 240.0 * t).to_radians();
        let inner = inner_start + (-std::f32::consts::TAU - 2.0 * inner_start) * t;
        (
            [22.0 * outer.cos(), 22.0 * outer.sin()],
            [
                offset + inner_radius * inner.cos(),
                inner_radius * inner.sin(),
            ],
        )
    };
    (0..32)
        .flat_map(|i| {
            let (a, d) = edge(i);
            let (b, c) = edge(i + 1);
            // ImGui expects clockwise winding in screen coordinates for its
            // outward anti-aliased fringe.
            [[a, c, b], [a, d, c]]
        })
        .collect()
}

pub fn strike_strokes() -> Vec<Stroke> {
    [
        (29.0, -75.0_f32, 65.0_f32, 2.8),
        (29.0, 105.0, 245.0, 2.8),
        (19.0, -28.0, 145.0, 3.4),
        (19.0, 160.0, 305.0, 2.4),
        (35.0, 145.0, 224.0, 1.3),
    ]
    .into_iter()
    .map(|(radius, start, end, width)| Stroke {
        points: (0..=28)
            .map(|i| {
                let angle = (start + (end - start) * i as f32 / 28.0).to_radians();
                [radius * angle.cos(), radius * angle.sin()]
            })
            .collect(),
        width,
    })
    .collect()
}

pub fn strike_shards() -> [[[f32; 2]; 3]; 4] {
    [
        [[-34.0, 39.0], [-17.0, 8.0], [-9.0, 10.0]],
        [[34.0, -39.0], [17.0, -8.0], [9.0, -10.0]],
        [[-32.0, -25.0], [-9.0, -10.0], [-15.0, -6.0]],
        [[32.0, 25.0], [9.0, 10.0], [15.0, 6.0]],
    ]
}

pub fn strike_svg() -> String {
    let mut svg = String::from("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"-48 -48 96 96\" fill=\"none\"><title>Attack phase strike emblem</title><g stroke=\"#F52B46\" stroke-linejoin=\"round\" stroke-linecap=\"round\">");
    for stroke in strike_strokes() {
        write!(svg, "<polyline stroke-width=\"{}\" points=\"", stroke.width).unwrap();
        for [x, y] in stroke.points {
            write!(svg, "{x:.2},{y:.2} ").unwrap();
        }
        svg.push_str("\"/>");
    }
    svg.push_str("</g><g fill=\"#F52B46\">");
    for points in strike_shards() {
        svg.push_str("<polygon points=\"");
        for [x, y] in points {
            write!(svg, "{x},{y} ").unwrap();
        }
        svg.push_str("\"/>");
    }
    svg.push_str("</g></svg>\n");
    svg
}
