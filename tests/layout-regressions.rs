use sekiro_deflect_observer::{
    config::{AnchorMode, Config},
    layout,
};

#[test]
fn default_1080p_layout_matches_the_reference_top_anchor() {
    let config = Config::default();
    let bounds =
        layout::bounds([1920.0, 1080.0], 16.0 / 9.0, &config).expect("default layout should fit");

    assert_eq!(bounds.center[0], 960.0);
    assert!((bounds.center[1] - 172.8).abs() < 0.01);
    assert_eq!(bounds.lane.width(), 480.0);
    assert!(bounds.full.top >= 1080.0 * 0.095);
    assert!(bounds.full.bottom <= 900.0);
    assert!(bounds.full.left >= 24.0);
    assert!(bounds.full.right <= 1896.0);
}

#[test]
fn supported_surfaces_keep_every_effect_inside_the_playable_viewport() {
    let config = Config::default();
    for (display, aspect) in [
        ([1280.0, 720.0], 16.0 / 9.0),
        ([1920.0, 1080.0], 16.0 / 9.0),
        ([2560.0, 1440.0], 16.0 / 9.0),
        ([3840.0, 2160.0], 16.0 / 9.0),
        ([3440.0, 1440.0], 16.0 / 9.0),
        ([5120.0, 1440.0], 16.0 / 9.0),
    ] {
        let bounds = layout::bounds(display, aspect, &config).expect("layout should fit");
        assert!(bounds.full.left >= bounds.viewport.left + config.safe_margin * bounds.scale);
        assert!(bounds.full.right <= bounds.viewport.right - config.safe_margin * bounds.scale);
        assert!(bounds.full.top >= bounds.viewport.top + config.safe_margin * bounds.scale);
        assert!(bounds.full.bottom <= bounds.posture_limit);
    }
}

#[test]
fn inset_and_small_viewports_clamp_or_suppress_without_clipping() {
    let config = Config::default();
    let inset = layout::bounds_in(
        layout::Rect::from_xywh(240.0, 120.0, 960.0, 540.0),
        16.0 / 9.0,
        &config,
        None,
    )
    .expect("inset viewport should fit");
    assert!(inset.full.left >= 252.0);
    assert!(inset.full.right <= 1188.0);
    assert!(inset.full.bottom <= 600.0);

    assert!(layout::bounds([180.0, 90.0], 16.0 / 9.0, &config).is_none());
    assert!(layout::bounds([720.0, 1280.0], 16.0 / 9.0, &config).is_none());
}

#[test]
fn maximum_label_scale_and_glow_are_included_in_full_bounds() {
    let config = Config {
        scale: 1.5,
        width: 360.0,
        label_size: 30.0,
        safe_margin: 8.0,
        posture_gap: 8.0,
        glow_intensity: 1.0,
        pulse_intensity: 1.0,
        ..Config::default()
    };
    let bounds = layout::bounds([3840.0, 2160.0], 16.0 / 9.0, &config)
        .expect("largest valid cue should fit at 4K");

    assert!(bounds.full.contains(bounds.lane));
    assert!(bounds.full.contains(bounds.label));
    assert!(bounds.full.bottom <= bounds.posture_limit);
    assert!(bounds.label.height() >= 90.0);

    let minimum = layout::bounds(
        [1280.0, 720.0],
        16.0 / 9.0,
        &Config {
            scale: 0.5,
            ..Config::default()
        },
    )
    .expect("minimum configured scale should remain readable at 720p");
    assert!(minimum.label.height() >= 14.0);
    assert!(minimum.full.contains(minimum.label));
}

#[test]
fn offsets_are_scaled_and_clamped_with_the_complete_cue() {
    let config = Config {
        anchor: AnchorMode::Posture,
        offset_x: 480.0,
        offset_y: 160.0,
        scale: 1.5,
        width: 360.0,
        ..Config::default()
    };
    let bounds = layout::bounds([1920.0, 1080.0], 16.0 / 9.0, &config)
        .expect("valid offset should be clamped into the viewport");

    assert_eq!(bounds.full.right, 1884.0);
    assert_eq!(bounds.full.bottom, 900.0);
}

#[test]
fn recorded_080_posture_bar_has_clearance_and_readable_warning_geometry() {
    // Video 22-14-54: 2560x720 recording of a 5120x1440 surface.
    // Wolf's actual posture decoration starts near video y=618, not y=648
    // assumed by the former 0.90 band. Use the earlier y=612 as a safe bound.
    let config = Config {
        anchor: AnchorMode::Posture,
        ..Config::default()
    };
    let b = layout::bounds([5120.0, 1440.0], 16.0 / 9.0, &config).unwrap();
    assert!(
        b.full.bottom / 2.0 <= 600.0,
        "cue overlaps recorded posture region: {:?}",
        b.full
    );
    assert!(b.lane.width() / 2.0 >= 210.0);
    assert!(b.lane.height() / 2.0 >= 9.0);
    assert!(b.label.height() / 2.0 >= 20.0);
}
