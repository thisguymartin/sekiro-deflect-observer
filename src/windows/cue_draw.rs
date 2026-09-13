//! Shared ImGui drawing for live DX11 frames and the offline visual check.
use super::diagnostics::RenderSample;
use crate::cue;
use hudhook::imgui::{self, Context, Ui};

pub(super) fn initialize_fonts(context: &mut Context) -> Vec<(f32, usize)> {
    let mut fonts = Vec::new();
    context
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
    for size in [18.0, 22.0, 26.0, 34.0, 42.0, 52.0] {
        let index = context.fonts().fonts().len();
        context
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData {
                config: Some(imgui::FontConfig {
                    size_pixels: size,
                    oversample_h: 2,
                    oversample_v: 2,
                    ..Default::default()
                }),
            }]);
        fonts.push((size, index));
    }
    fonts
}

pub(super) fn draw(
    ui: &Ui,
    target: &cue::Target,
    advancing: bool,
    vertical_adjust: i32,
    fonts: &[(f32, usize)],
    submitted: &mut RenderSample,
) {
    submitted.status = "projection_rejected";
    let Some(mut position) = target.camera.project(target.anchor, ui.io().display_size) else {
        return;
    };
    let draw = ui.get_background_draw_list();
    let scale = (ui.io().display_size[1] / 1080.0).clamp(0.75, 2.0);
    position[1] += vertical_adjust as f32 * scale;
    let width = 360.0 * scale;
    let height = 12.0 * scale;
    let left = position[0] - width / 2.0;
    let top = position[1] - height / 2.0;
    let right = left + width;
    let bottom = top + height;
    // Decorative ribbon wings sit outside the functional timing lane.
    // They do not change the marker's mapping or the estimated press interval.
    draw.add_polyline(
        vec![
            [left - 32.0 * scale, position[1]],
            [left - 22.0 * scale, top - 3.0 * scale],
            [right + 22.0 * scale, top - 3.0 * scale],
            [right + 32.0 * scale, position[1]],
            [right + 22.0 * scale, bottom + 3.0 * scale],
            [left - 22.0 * scale, bottom + 3.0 * scale],
        ],
        [0.025, 0.03, 0.025, 0.38],
    )
    .filled(true)
    .build();
    draw.add_rect([left, top], [right, bottom], [0.06, 0.07, 0.065, 0.65])
        .filled(true)
        .build();
    for (edge, direction) in [(left, -1.0_f32), (right, 1.0_f32)] {
        draw.add_polyline(
            vec![
                [edge - direction * 4.0 * scale, position[1] - 7.0 * scale],
                [edge + direction * 3.0 * scale, position[1]],
                [edge - direction * 4.0 * scale, position[1] + 7.0 * scale],
            ],
            [0.97, 0.98, 0.92, 0.9],
        )
        .thickness(2.0 * scale)
        .build();
    }
    submitted.position = Some(position);
    submitted.status = "locked";
    let mut cue_label = if crate::cue::special_attack(target) {
        "SPECIAL / UNVERIFIED"
    } else {
        "LOCKED"
    };
    let mut label_color = [0.8, 0.85, 0.9, 1.0];
    if target.animation_error.is_some() {
        submitted.status = "animation_unavailable";
    } else if crate::cue::attack_mapped(target) && !advancing {
        submitted.status = "clock_stopped";
    }
    if let Some(phase) = crate::cue::timeline(target).filter(|_| advancing) {
        submitted.phase = Some(phase);
        submitted.status = if phase.response_now {
            phase.response.label()
        } else {
            "timeline"
        };
        let green_left = left + width * phase.green_start;
        let green_right = left + width * phase.green_end;
        let response_color = match phase.response {
            cue::Response::Parry => [0.18, 0.95, 0.27, 0.95],
            cue::Response::Dodge => [1.0, 0.40, 0.12, 1.0],
            cue::Response::Jump => [0.18, 0.8, 1.0, 1.0],
            cue::Response::Unverified => [0.35, 0.38, 0.35, 0.65],
        };
        let zone = if phase.in_reach {
            response_color
        } else {
            [0.35, 0.38, 0.35, 0.65]
        };
        if phase.response_now {
            draw.add_rect(
                [green_left, top - 3.0 * scale],
                [green_right, bottom + 3.0 * scale],
                [
                    response_color[0],
                    response_color[1],
                    response_color[2],
                    0.25,
                ],
            )
            .filled(true)
            .build();
        }
        let bevel = (4.0 * scale).min((green_right - green_left) / 2.0);
        // Tapered colored segment stays inside the exact estimated interval.
        draw.add_polyline(
            vec![
                [green_left, position[1]],
                [green_left + bevel, top],
                [green_right - bevel, top],
                [green_right, position[1]],
                [green_right - bevel, bottom],
                [green_left + bevel, bottom],
            ],
            zone,
        )
        .filled(true)
        .build();
        // Filled diamond and thin needle inspired by the supplied video thumbnail.
        // The center crosses the same interval; glow never extends cue duration.
        let marker = left + width * phase.progress;
        let diamond = |radius: f32| {
            vec![
                [marker, position[1] - radius * 1.2 * scale],
                [marker + radius * scale, position[1]],
                [marker, position[1] + radius * 1.2 * scale],
                [marker - radius * scale, position[1]],
            ]
        };
        let glow = if phase.response_now {
            response_color
        } else {
            [0.9, 0.94, 1.0, 1.0]
        };
        for (radius, alpha) in [(22.0, 0.05), (17.0, 0.10), (13.0, 0.16)] {
            draw.add_polyline(diamond(radius), [glow[0], glow[1], glow[2], alpha])
                .filled(true)
                .build();
        }
        let needle_top = [marker, position[1] - 29.0 * scale];
        let needle_bottom = [marker, position[1] + 29.0 * scale];
        draw.add_line(needle_top, needle_bottom, [0.0, 0.0, 0.0, 0.55])
            .thickness(3.0 * scale)
            .build();
        draw.add_line(needle_top, needle_bottom, [1.0, 0.97, 0.86, 0.8])
            .thickness(1.0 * scale)
            .build();
        // A small downward pointer mirrors the reference and follows the cursor.
        // Magenta identifies the cursor only; the zone and English text give the response.
        let pointer_color = if phase.response != cue::Response::Unverified && phase.in_reach {
            [1.0, 0.08, 0.43, 1.0]
        } else {
            [0.65, 0.68, 0.7, 0.85]
        };
        let pointer = [
            [marker - 6.0 * scale, position[1] - 41.0 * scale],
            [marker, position[1] - 37.0 * scale],
            [marker + 6.0 * scale, position[1] - 41.0 * scale],
            [marker, position[1] - 28.0 * scale],
        ];
        // Two convex triangles form the notched arrow without a concave fill.
        for indices in [[0, 1, 3], [1, 2, 3]] {
            draw.add_polyline(indices.map(|i| pointer[i]).to_vec(), pointer_color)
                .filled(true)
                .build();
        }
        draw.add_polyline(diamond(11.5), [0.03, 0.04, 0.035, 0.85])
            .filled(true)
            .build();
        draw.add_polyline(diamond(10.0), [1.0, 0.98, 0.87, 1.0])
            .filled(true)
            .build();
        draw.add_polyline(diamond(4.0), [1.0, 1.0, 1.0, 1.0])
            .filled(true)
            .build();
        cue_label = if phase.response == cue::Response::Dodge {
            if !phase.in_reach {
                "GRAB / OUT OF REACH"
            } else if phase.response_now {
                "DODGE"
            } else if phase.remaining > 0.0 {
                "GRAB / GET READY"
            } else {
                "GRAB ACTIVE"
            }
        } else if phase.response == cue::Response::Jump {
            if !phase.in_reach {
                "SWEEP / OUT OF REACH"
            } else if phase.response_now {
                "JUMP"
            } else if phase.remaining > 0.0 {
                "SWEEP / GET READY"
            } else {
                "SWEEP ACTIVE"
            }
        } else if crate::cue::special_attack(target) {
            "SPECIAL / UNVERIFIED"
        } else if !phase.classified {
            "TIMING UNVERIFIED"
        } else if !phase.in_reach {
            "OUT OF REACH"
        } else if phase.press_now {
            "PARRY"
        } else if phase.remaining > 0.150 && phase.remaining <= 0.65 {
            "READY"
        } else {
            "LOCKED"
        };
        if matches!(phase.response, cue::Response::Dodge | cue::Response::Jump) {
            label_color = response_color;
        } else if phase.press_now {
            label_color = [0.7, 1.0, 0.75, 1.0];
        } else if !phase.classified {
            label_color = [1.0, 0.8, 0.4, 1.0];
        }
    }
    // Fixed, large label above the track: its brief PARRY state
    // is not stretched past the estimate to make it more visible.
    let desired = if matches!(cue_label, "PARRY" | "JUMP" | "DODGE") {
        26.0
    } else {
        18.0
    } * scale;
    let font = fonts
        .iter()
        .min_by(|a, b| (a.0 - desired).abs().total_cmp(&(b.0 - desired).abs()))
        .unwrap()
        .1;
    let _font = ui.push_font(ui.fonts().fonts()[font]);
    let size = ui.calc_text_size(cue_label);
    let text = [position[0] - size[0] / 2.0, top - size[1] - 42.0 * scale];
    // Outlined lettering stays legible over bright scenery without a boxed badge.
    for (x, y) in [
        (-1.0, -1.0),
        (0.0, -1.0),
        (1.0, -1.0),
        (-1.0, 0.0),
        (1.0, 0.0),
        (-1.0, 1.0),
        (0.0, 1.0),
        (1.0, 1.0),
    ] {
        draw.add_text(
            [text[0] + x * 1.5 * scale, text[1] + y * 1.5 * scale],
            [0.015, 0.02, 0.015, 0.95],
            cue_label,
        );
    }
    draw.add_text(text, label_color, cue_label);
}
