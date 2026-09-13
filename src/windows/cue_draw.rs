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
    let height = 16.0 * scale;
    let left = position[0] - width / 2.0;
    let top = position[1] - height / 2.0;
    let right = left + width;
    let bottom = top + height;
    // Thin translucent track and end chevrons from the user's reference.
    draw.add_rect(
        [left - 5.0 * scale, top - 4.0 * scale],
        [right + 5.0 * scale, bottom + 4.0 * scale],
        [0.0, 0.0, 0.0, 0.22],
    )
    .filled(true)
    .build();
    draw.add_rect([left, top], [right, bottom], [0.025, 0.03, 0.025, 0.72])
        .filled(true)
        .build();
    for (edge, direction) in [(left, -1.0_f32), (right, 1.0_f32)] {
        draw.add_polyline(
            vec![
                [edge - direction * 3.0 * scale, position[1] - 5.0 * scale],
                [edge + direction * 2.0 * scale, position[1]],
                [edge - direction * 3.0 * scale, position[1] + 5.0 * scale],
            ],
            [0.92, 0.95, 0.9, 0.8],
        )
        .thickness(1.4 * scale)
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
        draw.add_rect([green_left, top], [green_right, bottom], zone)
            .filled(true)
            .build();
        // Diamond travels over the actual timing interval: no invented width.
        let marker = left + width * phase.progress;
        let radius = 10.0 * scale;
        let diamond = vec![
            [marker, position[1] - radius],
            [marker + radius, position[1]],
            [marker, position[1] + radius],
            [marker - radius, position[1]],
            [marker, position[1] - radius],
        ];
        draw.add_polyline(diamond.clone(), [0.0, 0.0, 0.0, 0.9])
            .thickness(6.0 * scale)
            .build();
        draw.add_polyline(diamond, [0.98, 1.0, 0.96, 1.0])
            .thickness(2.5 * scale)
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
    let text = [position[0] - size[0] / 2.0, top - size[1] - 10.0 * scale];
    draw.add_rect(
        [text[0] - 8.0 * scale, text[1] - 3.0 * scale],
        [
            text[0] + size[0] + 8.0 * scale,
            text[1] + size[1] + 3.0 * scale,
        ],
        [0.015, 0.025, 0.02, 0.88],
    )
    .filled(true)
    .rounding(3.0 * scale)
    .build();
    draw.add_text(text, label_color, cue_label);
}
