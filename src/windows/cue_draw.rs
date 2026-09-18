//! Shared ImGui drawing for live DX11 frames and the offline visual check.
use super::diagnostics::RenderSample;
use crate::config::{AnchorMode, Config};
use crate::cue::{self, Response};
use crate::layout::{self, Bounds, Rect};
use crate::timing::{Decision, State};
use hudhook::imgui::{self, Context, Ui};

/// Adjacent moon triangles share edges. Anti-alias only the exterior contour,
/// so internal seams stay invisible. Restore the flag even on unwind.
struct FillAntialiasGuard {
    list: *mut imgui::sys::ImDrawList,
    flags: imgui::sys::ImDrawListFlags,
}
impl FillAntialiasGuard {
    fn without_fill_antialiasing() -> Self {
        // The render thread holds a background DrawListMut in a live frame;
        // this is that same context-owned list, not global style or game state.
        unsafe {
            let list = imgui::sys::igGetBackgroundDrawList();
            let flags = (*list).Flags;
            (*list).Flags &= !(imgui::sys::ImDrawListFlags_AntiAliasedFill as i32);
            Self { list, flags }
        }
    }
}
impl Drop for FillAntialiasGuard {
    fn drop(&mut self) {
        // The guard cannot outlive this frame's drawing call.
        unsafe {
            (*self.list).Flags = self.flags;
        }
    }
}

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

/// Session status, independent of attack hints and whether a target is locked.
/// The caller supplies a current controller snapshot; this never requests speed.
pub(super) fn draw_practice_indicator(
    ui: &Ui,
    config: &Config,
    fonts: &[(f32, usize)],
    enabled: bool,
    practice: crate::practice::Snapshot,
    aspect: f32,
) -> Option<layout::PracticeIndicatorBounds> {
    use crate::practice::Status;
    if !config.visible {
        return None;
    }
    let bounds = layout::practice_indicator_bounds(ui.io().display_size, aspect, config)?;
    let percent = (config.practice_speed * 100.0).round() as u32;
    let active = enabled && practice.status == Status::Active && practice.percent == percent;
    let warning = practice.status == Status::RestorePending
        || enabled
            && matches!(
                practice.status,
                Status::Unavailable | Status::Conflict | Status::Unsupported
            );
    let label = format!(
        "{} {percent}%{}",
        if enabled { "ON" } else { "OFF" },
        if warning { "!" } else { "" }
    );
    let hue = if warning {
        [0.96, 0.58, 0.26, config.opacity]
    } else if active {
        [0.48, 0.91, 0.76, config.opacity]
    } else if enabled {
        [0.78, 0.70, 0.49, config.opacity]
    } else {
        [0.58, 0.62, 0.62, config.opacity]
    };
    let s = bounds.scale;
    let [cx, cy] = bounds.center;
    let point = |x: f32, y: f32| [cx + x * s, cy + y * s];
    let draw = ui.get_background_draw_list();
    // A local ink wash makes the fine blade readable against snow and foliage.
    draw.add_circle(
        [cx, cy],
        26.5 * s,
        [0.025, 0.03, 0.027, 0.58 * config.opacity],
    )
    .filled(true)
    .num_segments(48)
    .build();
    if active && !config.reduced_flash && config.glow_intensity > 0.0 {
        let rim: Vec<_> = (0..=32)
            .map(|i| {
                let a = (-60.0 - 240.0 * i as f32 / 32.0).to_radians();
                point(22.0 * a.cos(), 22.0 * a.sin())
            })
            .collect();
        draw.add_polyline(
            rim,
            [
                hue[0],
                hue[1],
                hue[2],
                0.18 * config.opacity * config.glow_intensity,
            ],
        )
        .thickness(5.0 * s)
        .build();
    }
    let moon = crate::hud_art::practice_moon_triangles(config.practice_speed);
    {
        let _fill_guard = FillAntialiasGuard::without_fill_antialiasing();
        for &[a, b, c] in &moon {
            draw.add_triangle(point(a[0], a[1]), point(b[0], b[1]), point(c[0], c[1]), hue)
                .filled(true)
                .build();
        }
    }
    let contour: Vec<_> = moon
        .chunks_exact(2)
        .map(|pair| pair[0][0])
        .chain(moon.chunks_exact(2).rev().map(|pair| pair[1][2]))
        .map(|p| point(p[0], p[1]))
        .collect();
    draw.add_polyline(contour, hue).thickness(0.8 * s).build();
    let blade = if enabled {
        [0.95, 0.93, 0.83, config.opacity]
    } else {
        [0.61, 0.64, 0.62, config.opacity]
    };
    draw.add_polyline(
        vec![point(-4.0, 6.0), point(3.0, -5.0), point(8.0, -17.0)],
        blade,
    )
    .thickness(2.8 * s)
    .build();
    draw.add_triangle(
        point(6.7, -17.0),
        point(10.0, -23.0),
        point(9.4, -15.0),
        blade,
    )
    .filled(true)
    .build();
    draw.add_line(
        point(-5.0, 9.0),
        point(-11.0, 19.0),
        [0.51, 0.32, 0.24, config.opacity],
    )
    .thickness(4.0 * s)
    .build();
    for y in [11.0, 15.0, 19.0] {
        let x = -5.0 - (y - 9.0) * 0.6;
        draw.add_line(point(x - 1.8, y - 1.0), point(x + 1.8, y + 1.0), hue)
            .thickness(s)
            .build();
    }
    draw.add_line(point(-10.0, 4.0), point(1.0, 11.0), hue)
        .thickness(2.5 * s)
        .build();
    let font = fonts
        .iter()
        .filter(|(size, _)| *size <= (14.0 * s).max(14.0))
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .or_else(|| fonts.first())?
        .1;
    let _font = ui.push_font(ui.fonts().fonts()[font]);
    let size = ui.calc_text_size(&label);
    let text = [cx - size[0] * 0.5, cy + 32.0 * s];
    for (dx, dy) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
        draw.add_text(
            [text[0] + dx, text[1] + dy],
            [0.01, 0.015, 0.012, config.opacity],
            &label,
        );
    }
    draw.add_text(text, hue, &label);
    Some(bounds)
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

    let practice = submitted.practice;
    // Report the controller's applied speed, even when this attack's hint is
    // disabled. Rendering never decides which attacks practice may slow.
    let percent = practice.active_for(target).then_some(practice.percent);
    let armed = practice.status == crate::practice::Status::Ready;
    draw_cue(ui, &bounds, decision, config, fonts, percent, armed);
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

#[allow(clippy::too_many_arguments)]
fn draw_cue(
    ui: &Ui,
    bounds: &Bounds,
    decision: &Decision,
    config: &Config,
    fonts: &[(f32, usize)],
    practice_percent: Option<u32>,
    practice_armed: bool,
) {
    let (label, mut hue, _) = presentation(decision, config);
    let label = if practice_percent.is_some() && decision.state == State::Neutral {
        "PRACTICE"
    } else {
        label
    };
    let label = if let Some(percent) = practice_percent {
        format!("{label} {percent}%")
    } else if practice_armed && decision.state == State::Neutral {
        "PRACTICE".to_string()
    } else {
        label.to_string()
    };
    let label = label.as_str();
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
