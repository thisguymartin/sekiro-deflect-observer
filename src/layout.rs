//! Portable HUD geometry shared by the live overlay and synthetic previews.
use crate::config::{AnchorMode, Config};

const MIN_REFERENCE_SCALE: f32 = 0.5;
const FIXED_X: f32 = 0.5;
// Caption now sits below the rail, so legacy posture mode needs more clearance.
const FIXED_Y: f32 = 0.78;
const LANE_HEIGHT: f32 = 18.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Rect {
    pub fn from_xywh(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        }
    }

    pub fn width(self) -> f32 {
        self.right - self.left
    }

    pub fn height(self) -> f32 {
        self.bottom - self.top
    }

    pub fn contains(self, other: Self) -> bool {
        other.left >= self.left
            && other.top >= self.top
            && other.right <= self.right
            && other.bottom <= self.bottom
    }

    fn translated(self, x: f32, y: f32) -> Self {
        Self {
            left: self.left + x,
            top: self.top + y,
            right: self.right + x,
            bottom: self.bottom + y,
        }
    }

    fn union(self, other: Self) -> Self {
        Self {
            left: self.left.min(other.left),
            top: self.top.min(other.top),
            right: self.right.max(other.right),
            bottom: self.bottom.max(other.bottom),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub viewport: Rect,
    pub full: Rect,
    pub lane: Rect,
    pub label: Rect,
    pub center: [f32; 2],
    pub scale: f32,
    pub posture_limit: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct PracticeIndicatorBounds {
    pub viewport: Rect,
    pub full: Rect,
    pub center: [f32; 2],
    pub scale: f32,
}

/// A small, independent status crest in the playable viewport's upper right.
/// It does not need a locked target or inherit the attack rail's placement.
pub fn practice_indicator_bounds(
    display: [f32; 2],
    aspect: f32,
    config: &Config,
) -> Option<PracticeIndicatorBounds> {
    if ![
        display[0],
        display[1],
        aspect,
        config.scale,
        config.safe_margin,
    ]
    .iter()
    .all(|v| v.is_finite() && *v >= 0.0)
        || aspect == 0.0
        || config.scale == 0.0
    {
        return None;
    }
    let width = display[0].min(display[1] * aspect);
    let height = width / aspect;
    if height < 540.0 {
        return None;
    }
    let viewport = Rect::from_xywh(
        (display[0] - width) * 0.5,
        (display[1] - height) * 0.5,
        width,
        height,
    );
    // Keep this tiny and legible even when the attack rail is heavily scaled.
    let scale = (height / 1080.0 * config.scale).clamp(0.75, 2.0);
    let margin = config.safe_margin.max(18.0) * scale;
    let full = Rect::from_xywh(
        viewport.right - margin - 88.0 * scale,
        viewport.top + margin,
        88.0 * scale,
        84.0 * scale,
    );
    viewport.contains(full).then_some(PracticeIndicatorBounds {
        viewport,
        full,
        center: [full.left + 44.0 * scale, full.top + 30.0 * scale],
        scale,
    })
}

/// Layout the fixed posture cue in a display surface.
pub fn bounds(display: [f32; 2], aspect: f32, config: &Config) -> Option<Bounds> {
    bounds_in(
        Rect::from_xywh(0.0, 0.0, display[0], display[1]),
        aspect,
        config,
        None,
    )
}

/// Layout in an inset surface. `anchor` is an absolute screen position used by
/// overhead mode; `None` selects the normalized fixed posture anchor.
pub fn bounds_in(
    surface: Rect,
    aspect: f32,
    config: &Config,
    anchor: Option<[f32; 2]>,
) -> Option<Bounds> {
    if ![
        surface.left,
        surface.top,
        surface.right,
        surface.bottom,
        aspect,
    ]
    .iter()
    .all(|value| value.is_finite())
        || surface.width() <= 0.0
        || surface.height() <= 0.0
        || aspect <= 0.0
    {
        return None;
    }

    let fitted_width = surface.width().min(surface.height() * aspect);
    let fitted_height = fitted_width / aspect;
    let viewport = Rect::from_xywh(
        surface.left + (surface.width() - fitted_width) * 0.5,
        surface.top + (surface.height() - fitted_height) * 0.5,
        fitted_width,
        fitted_height,
    );
    let reference_scale = viewport.height() / 1080.0;
    if reference_scale < MIN_REFERENCE_SCALE {
        return None;
    }
    let scale = reference_scale * config.scale;
    if !scale.is_finite() || scale <= 0.0 {
        return None;
    }

    let margin = config.safe_margin * scale;
    let posture_limit = (viewport.top + viewport.height() * config.posture_band_top
        - config.posture_gap * scale)
        .min(viewport.bottom - margin);
    let available = Rect {
        left: viewport.left + margin,
        top: if config.anchor == AnchorMode::Top {
            (viewport.top + viewport.height() * 0.095).max(viewport.top + margin)
        } else {
            viewport.top + margin
        },
        right: viewport.right - margin,
        bottom: posture_limit,
    };
    if available.width() <= 0.0 || available.height() <= 0.0 {
        return None;
    }

    let base = anchor.unwrap_or([
        viewport.left + viewport.width() * FIXED_X,
        viewport.top
            + viewport.height()
                * if config.anchor == AnchorMode::Top {
                    0.16
                } else {
                    FIXED_Y
                },
    ]);
    if !base.iter().all(|value| value.is_finite()) {
        return None;
    }
    let desired = [
        base[0] + config.offset_x * scale,
        base[1] + config.offset_y * scale,
    ];
    let lane_half_width = config.width * scale * 0.5;
    let lane_half_height = LANE_HEIGHT * scale * 0.5;
    // Two additional reference pixels contain ImGui's anti-aliased edge.
    let effect_pad = 16.0 * scale;
    let lane_relative = Rect {
        left: -lane_half_width,
        top: -lane_half_height,
        right: lane_half_width,
        bottom: lane_half_height,
    };
    let effects_relative = Rect {
        left: lane_relative.left - effect_pad,
        top: lane_relative.top - effect_pad,
        right: lane_relative.right + effect_pad,
        bottom: lane_relative.bottom + effect_pad,
    };
    let label_height = (config.label_size * scale).max(14.0);
    // Include the response caption and optional controller badge below the rail.
    let label_width = 12.0 * label_height * 0.72 + 12.0 * scale;
    let label_relative = Rect {
        left: -label_width * 0.5,
        top: 23.0 * scale,
        right: label_width * 0.5,
        bottom: 23.0 * scale + label_height + 2.0 * scale,
    };
    // The red strike emblem is drawn at the fixed center gate and stays local.
    let emblem = Rect::from_xywh(-46.0 * scale, -46.0 * scale, 92.0 * scale, 92.0 * scale);
    let full_relative = effects_relative.union(label_relative).union(emblem);
    if full_relative.width() > available.width() || full_relative.height() > available.height() {
        return None;
    }

    let center = [
        desired[0].clamp(
            available.left - full_relative.left,
            available.right - full_relative.right,
        ),
        desired[1].clamp(
            available.top - full_relative.top,
            available.bottom - full_relative.bottom,
        ),
    ];
    let lane = lane_relative.translated(center[0], center[1]);
    let label = label_relative.translated(center[0], center[1]);
    let full = full_relative.translated(center[0], center[1]);
    Some(Bounds {
        viewport,
        full,
        lane,
        label,
        center,
        scale,
        posture_limit,
    })
}
