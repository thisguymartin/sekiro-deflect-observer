//! Shared ImGui drawing for live DX11 frames and the offline visual check.
use super::diagnostics::RenderSample;
use crate::config::{AnchorMode, Config};
use crate::cue::{self, Response};
use crate::layout::{self, Bounds, Rect};
use crate::timing::{Decision, State};
use hudhook::imgui::{self, Context, Ui};

pub(super) fn initialize_fonts(context: &mut Context) -> Vec<(f32, usize)> {
    let mut fonts = Vec::new();
    // imgui 0.12 exposes an invalid empty-vector slice on recent Rust UB checks;
    // seed the atlas before asking for its length.
    context
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
    for size in [
        14.0, 18.0, 20.0, 24.0, 30.0, 36.0, 40.0, 48.0, 60.0, 72.0, 90.0,
    ] {
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
    decision: &Decision,
    config: &Config,
    fonts: &[(f32, usize)],
    submitted: &mut RenderSample,
) {
    submitted.decision = decision.clone();
    submitted.status = decision.reason;
    if decision.state == State::Hidden || !config.visible {
        return;
    }

    let display = ui.io().display_size;
    let (aspect, anchor, mode) = match config.anchor {
        AnchorMode::Top | AnchorMode::Posture => match target.camera {
            Some(camera) => (camera.aspect, None, "fixed_camera_viewport"),
            None => (16.0 / 9.0, None, "posture_configured_16_9"),
        },
        AnchorMode::Overhead => {
            let Some(camera) = target.camera else {
                submitted.status = "overhead_camera_missing";
                return;
            };
            let Ok(position) = camera.project_checked(target.anchor, display) else {
                submitted.status = "overhead_anchor_offscreen";
                return;
            };
            (camera.aspect, Some(position), "overhead")
        }
    };
    let Some(bounds) = layout::bounds_in(
        Rect::from_xywh(0.0, 0.0, display[0], display[1]),
        aspect,
        config,
        anchor,
    ) else {
        submitted.status = "layout_suppressed";
        return;
    };
    submitted.position = Some(bounds.center);
    submitted.layout_mode = if config.anchor == AnchorMode::Top {
        "top_reference"
    } else {
        mode
    };
    submitted.surface = display;
    submitted.viewport = rect_array(bounds.viewport);
    submitted.bounds = Some(rect_array(bounds.full));

    draw_cue(ui, &bounds, decision, config, fonts);
}

fn presentation(decision: &Decision, config: &Config) -> (&'static str, [f32; 4], bool) {
    if matches!(decision.state, State::Incoming | State::AttackActive) {
        let (label, color) = match decision.response {
            Response::Parry => ("PARRY", config.colors.parry),
            Response::Dodge => ("DODGE", config.colors.dodge),
            Response::Jump => ("JUMP", config.colors.jump),
            Response::Mikiri => ("MIKIRI", [1.0, 0.82, 0.2, 1.0]),
            Response::Avoid => ("NO PARRY", [1.0, 0.35, 0.25, 1.0]),
            Response::Unverified => ("UNKNOWN", config.colors.ready),
        };
        return (label, color, decision.state == State::AttackActive);
    }
    match decision.state {
        State::Preparation => ("READY", config.colors.ready, false),
        State::Actionable => match decision.response {
            Response::Parry => ("PARRY NOW", config.colors.parry, true),
            Response::Dodge => ("DODGE", config.colors.dodge, true),
            Response::Jump => ("JUMP", config.colors.jump, true),
            Response::Unverified | Response::Mikiri | Response::Avoid => {
                ("LOCKED", config.colors.ready, false)
            }
        },
        State::Expired => ("EXPIRED", config.colors.expired, true),
        State::Neutral if decision.reason == "unverified_attack" => {
            ("WATCH", config.colors.ready, false)
        }
        State::Neutral => ("LOCKED", config.colors.ready, false),
        State::Hidden | State::Incoming | State::AttackActive => ("", config.colors.ready, false),
    }
}

fn draw_cue(
    ui: &Ui,
    bounds: &Bounds,
    decision: &Decision,
    config: &Config,
    fonts: &[(f32, usize)],
) {
    let (label, mut hue, _) = presentation(decision, config);
    let draw = ui.get_background_draw_list();
    let s = bounds.scale;
    let [cx, cy] = bounds.center;
    let lane = bounds.lane;
    let active = decision.state == State::AttackActive;
    let striking = active && decision.response == Response::Parry && !config.reduced_flash;
    let windup = matches!(decision.state, State::Incoming | State::Preparation);
    let showing_attack = windup || active || decision.state == State::Actionable;
    if showing_attack && decision.response == Response::Unverified {
        hue = [0.37, 0.42, 0.46, 0.9];
    }
    hue[3] *= config.opacity;
    let fill = if striking {
        [0.98, 0.99, 1.0, config.opacity]
    } else {
        hue
    };

    // All contrast is confined to the slim rail and caption. No enclosing panel.
    for pad in [6.0, 3.0] {
        pointed_bar(
            &draw,
            inflate(lane, pad * s),
            [0.0, 0.0, 0.0, 0.13 * config.opacity],
        );
    }
    pointed_bar(&draw, lane, [0.025, 0.029, 0.032, 0.74 * config.opacity]);
    draw.add_line(
        [lane.left + 10.0 * s, lane.top],
        [lane.right - 10.0 * s, lane.top],
        [0.68, 0.73, 0.76, 0.18 * config.opacity],
    )
    .thickness(s)
    .build();

    if showing_attack {
        let segment = Rect {
            left: cx - 7.0 * s,
            right: lane.right - 8.0 * s,
            ..lane
        };
        if config.glow_intensity > 0.0 {
            for (pad, alpha) in [(5.0, 0.075), (2.5, 0.14)] {
                let glow = [
                    fill[0],
                    fill[1],
                    fill[2],
                    alpha * config.glow_intensity * config.opacity,
                ];
                pointed_bar(&draw, inflate(segment, pad * s), glow);
            }
        }
        pointed_bar(&draw, segment, fill);
        draw.add_line(
            [cx + 4.0 * s, lane.top + 2.0 * s],
            [segment.right - 9.0 * s, lane.top + 2.0 * s],
            [1.0, 1.0, 1.0, 0.27 * config.opacity],
        )
        .thickness(s)
        .build();
    }

    // Chevron endcaps and the fixed center gate match the reference silhouette.
    for (x, direction, alpha) in [
        (lane.left - 8.0 * s, -1.0, 0.38),
        (lane.right + 8.0 * s, 1.0, 0.86),
    ] {
        draw.add_polyline(
            vec![
                [x - direction * 4.0 * s, cy - 8.0 * s],
                [x + direction * 2.0 * s, cy],
                [x - direction * 4.0 * s, cy + 8.0 * s],
            ],
            [0.88, 0.94, 0.94, alpha * config.opacity],
        )
        .thickness(1.6 * s)
        .build();
    }
    draw.add_line(
        [cx, cy - 22.0 * s],
        [cx, cy + 21.0 * s],
        [0.84, 0.97, 0.93, 0.48 * config.opacity],
    )
    .thickness(s)
    .build();
    diamond(
        &draw,
        [cx, cy],
        6.5 * s,
        [0.83, 0.96, 0.88, 0.85 * config.opacity],
        false,
    );

    if striking {
        let red = [0.96, 0.17, 0.27, 0.96 * config.opacity];
        for stroke in crate::hud_art::strike_strokes() {
            let points: Vec<[f32; 2]> = stroke
                .points
                .iter()
                .map(|p| [cx + p[0] * s, cy + p[1] * s])
                .collect();
            if config.glow_intensity > 0.0 {
                draw.add_polyline(
                    points.clone(),
                    [red[0], red[1], red[2], 0.16 * config.glow_intensity],
                )
                .thickness((stroke.width + 5.0) * s)
                .build();
            }
            draw.add_polyline(points, red)
                .thickness(stroke.width * s)
                .build();
        }
        for points in crate::hud_art::strike_shards() {
            draw.add_triangle(
                [cx + points[0][0] * s, cy + points[0][1] * s],
                [cx + points[1][0] * s, cy + points[1][1] * s],
                [cx + points[2][0] * s, cy + points[2][1] * s],
                red,
            )
            .filled(true)
            .build();
        }
    }
    if showing_attack {
        let marker_x = if windup {
            cx + (lane.right - cx - 14.0 * s) * (1.0 - decision.progress.clamp(0.0, 1.0))
        } else {
            cx
        };
        diamond(&draw, [marker_x, cy], 8.0 * s, [0.0, 0.0, 0.0, 0.72], true);
        diamond(
            &draw,
            [marker_x, cy],
            6.0 * s,
            [1.0, 1.0, 1.0, config.opacity],
            true,
        );
    }

    let desired = (config.label_size * 0.68 * s).max(14.0);
    let font = fonts
        .iter()
        .filter(|(size, _)| *size <= desired)
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .or_else(|| fonts.first())
        .unwrap()
        .1;
    let _font = ui.push_font(ui.fonts().fonts()[font]);
    let text_size = ui.calc_text_size(label);
    let has_button = showing_attack && decision.response == Response::Parry;
    let hint = config.parry_button.label();
    let hint_size = ui.calc_text_size(hint);
    let badge_width = if has_button {
        hint_size[0] + 12.0 * s
    } else {
        0.0
    };
    let gap = if has_button { 9.0 * s } else { 0.0 };
    let x = cx - (text_size[0] + badge_width + gap) * 0.5;
    let y = bounds.label.top;
    if has_button {
        draw.add_rect(
            [x, y - s],
            [x + badge_width, y + hint_size[1] + s],
            [0.90, 0.94, 0.92, 0.93 * config.opacity],
        )
        .rounding(3.0 * s)
        .filled(true)
        .build();
        draw.add_text([x + 6.0 * s, y], [0.055, 0.075, 0.065, 1.0], hint);
    }
    let text = [x + badge_width + gap, y];
    // Tight text outline, rather than the previous large opaque backing.
    for (dx, dy) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
        draw.add_text(
            [text[0] + dx * s, text[1] + dy * s],
            [0.01, 0.015, 0.012, 0.9],
            label,
        );
    }
    draw.add_text(
        text,
        [
            hue[0].max(0.8),
            hue[1].max(0.8),
            hue[2].max(0.8),
            config.opacity,
        ],
        label,
    );
}

fn pointed_bar(draw: &imgui::DrawListMut<'_>, rect: Rect, color: [f32; 4]) {
    let tip = (rect.height() * 0.5).min(rect.width() * 0.25);
    let y = (rect.top + rect.bottom) * 0.5;
    draw.add_polyline(
        vec![
            [rect.left, y],
            [rect.left + tip, rect.top],
            [rect.right - tip, rect.top],
            [rect.right, y],
            [rect.right - tip, rect.bottom],
            [rect.left + tip, rect.bottom],
        ],
        color,
    )
    .filled(true)
    .build();
}

fn diamond(
    draw: &imgui::DrawListMut<'_>,
    center: [f32; 2],
    radius: f32,
    color: [f32; 4],
    filled: bool,
) {
    let [x, y] = center;
    draw.add_polyline(
        vec![
            [x, y - radius],
            [x + radius * 0.7, y],
            [x, y + radius],
            [x - radius * 0.7, y],
            [x, y - radius],
        ],
        color,
    )
    .thickness((radius * 0.16).max(1.0))
    .filled(filled)
    .build();
}

fn inflate(rect: Rect, amount: f32) -> Rect {
    Rect {
        left: rect.left - amount,
        top: rect.top - amount,
        right: rect.right + amount,
        bottom: rect.bottom + amount,
    }
}

fn rect_array(rect: Rect) -> [f32; 4] {
    [rect.left, rect.top, rect.right, rect.bottom]
}
