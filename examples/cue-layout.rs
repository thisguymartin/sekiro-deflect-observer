//! Emit the actual ImGui draw mesh for a reproducible, synthetic visual check.
//! No game reads, hooks, DLL loading, or input generation.
use hudhook::imgui::{self, DrawCmd};
use sekiro_deflect_observer::{config, cue, layout, timing};
pub use sekiro_deflect_observer::{hud_art, practice};
use std::fs::{self, File};
use std::io::Write;
use std::time::Duration;

mod diagnostics {
    use super::timing;

    #[derive(Default)]
    pub(super) struct RenderSample {
        pub status: &'static str,
        pub position: Option<[f32; 2]>,
        pub decision: timing::Decision,
        pub layout_mode: &'static str,
        pub surface: [f32; 2],
        pub viewport: [f32; 4],
        pub bounds: Option<[f32; 4]>,
        pub practice: super::practice::Snapshot,
    }
}
#[path = "../src/windows/cue_draw.rs"]
mod cue_draw;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let output = args
        .get(1)
        .filter(|arg| !arg.starts_with("--"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| "dist/review-0.9.0/layout/default".into());
    fs::create_dir_all(&output)?;
    fs::write(output.join("strike-emblem.svg"), hud_art::strike_svg())?;
    let incoming_gallery = args.iter().any(|arg| arg == "--incoming-gallery");
    let gallery = incoming_gallery || args.iter().any(|arg| arg == "--gallery");
    let no_camera = args.iter().any(|arg| arg == "--no-camera");
    let state = option(&args, "--state").unwrap_or("parry");
    let indicator_only = args.iter().any(|arg| arg == "--indicator-only");
    let practice_speed = option(&args, "--speed")
        .map(str::parse)
        .transpose()?
        .unwrap_or(0.8_f32);
    if !practice_speed.is_finite() || !(0.5..=1.0).contains(&practice_speed) {
        return Err("--speed must be between 0.5 and 1.0".into());
    }
    let indicator = option(&args, "--indicator").or_else(|| {
        args.iter()
            .any(|arg| arg == "--practice")
            .then_some("active")
    });
    let display = parse_display(&args)?.unwrap_or([1920.0, 1080.0]);
    let scale = option(&args, "--scale")
        .map(str::parse)
        .transpose()?
        .unwrap_or(1.0_f32);
    let camera_aspect = option(&args, "--aspect")
        .map(str::parse)
        .transpose()?
        .unwrap_or(16.0_f32 / 9.0);

    let states: Vec<&str> = if incoming_gallery {
        vec![
            "incoming-parry",
            "active-parry",
            "incoming-dodge",
            "incoming-jump",
            "incoming-mikiri",
            "incoming-avoid",
            "incoming-unknown",
            "locked",
        ]
    } else if gallery {
        vec![
            "ready", "parry", "dodge", "jump", "expired", "watch", "locked",
        ]
    } else {
        vec![state]
    };
    let mut context = imgui::Context::create();
    context.set_ini_filename(None);
    let fonts = cue_draw::initialize_fonts(&mut context);
    let atlas = context.fonts().build_rgba32_texture();
    let (aw, ah) = (atlas.width, atlas.height);
    fs::write(output.join("atlas.rgba"), atlas.data)?;
    context.io_mut().display_size = display;
    context.io_mut().delta_time = 1.0 / 60.0;
    let ui = context.frame();
    let mut measurements = Vec::new();
    for (row, state) in states.iter().enumerate() {
        let (mut target, mut decision) = synthetic(state, camera_aspect)?;
        if no_camera {
            target.camera = None;
        }
        let expected = expected_state(state);
        if decision.state != expected {
            return Err(format!(
                "synthetic {state} produced {:?}, expected {:?}: {}",
                decision.state, expected, decision.reason
            )
            .into());
        }
        let mut config = config::Config {
            scale,
            practice_speed,
            ..config::Config::default()
        };
        if args.iter().any(|arg| arg == "--posture") {
            config.anchor = config::AnchorMode::Posture;
        }
        if args.iter().any(|arg| arg == "--reduced-flash") {
            config.reduced_flash = true;
        }
        if gallery {
            config.offset_y = row as f32 * 98.0;
        }
        if args.iter().any(|arg| arg == "--no-hints") {
            config.parry = false;
            config.dodge = false;
            config.jump = false;
            config.mikiri = false;
            let at = target.captured_at.unwrap_or_default();
            let mut engine = timing::Engine::default();
            engine.observe(at, Some(&target));
            decision = engine.incoming(at, [false; 3], false);
        }
        let mut submitted = diagnostics::RenderSample::default();
        if args.iter().any(|arg| arg == "--practice") {
            submitted.practice = practice::Snapshot {
                status: if practice::eligible(&target, target.captured_at.unwrap_or_default()) {
                    practice::Status::Active
                } else {
                    practice::Status::Ready
                },
                handle: target.handle,
                model: target.model,
                animation_module: target.animation_module,
                percent: (practice_speed * 100.0).round() as u32,
                ..Default::default()
            };
        }
        if !indicator_only {
            cue_draw::draw(ui, &target, &decision, &config, &fonts, &mut submitted);
            measurements.push((state.to_string(), submitted));
        }
    }
    if let Some(mode) = indicator {
        let (enabled, status) = match mode {
            "armed" => (true, practice::Status::Ready),
            "active" => (true, practice::Status::Active),
            "paused" => (true, practice::Status::Conflict),
            "unavailable" => (true, practice::Status::Unavailable),
            "unsupported" => (true, practice::Status::Unsupported),
            "pending" => (false, practice::Status::RestorePending),
            "off" => (false, practice::Status::Off),
            _ => return Err(format!("unknown indicator mode: {mode}").into()),
        };
        let config = config::Config {
            scale,
            practice_speed,
            visible: !args.iter().any(|arg| arg == "--hidden"),
            reduced_flash: args.iter().any(|arg| arg == "--reduced-flash"),
            ..Default::default()
        };
        // No Target is passed: this exercises the persistent, unlocked status.
        let bounds = cue_draw::draw_practice_indicator(
            ui,
            &config,
            &fonts,
            enabled,
            practice::Snapshot {
                status,
                percent: (practice_speed * 100.0).round() as u32,
                ..Default::default()
            },
            camera_aspect,
        );
        let expected_visible = config.visible;
        if bounds.is_some() != expected_visible {
            return Err(format!("unexpected indicator visibility for {mode}").into());
        }
        if let Some(bounds) = bounds {
            measurements.push((
                format!("indicator-{mode}"),
                diagnostics::RenderSample {
                    status: status.label(),
                    position: Some(bounds.center),
                    layout_mode: "practice_top_right",
                    viewport: [
                        bounds.viewport.left,
                        bounds.viewport.top,
                        bounds.viewport.right,
                        bounds.viewport.bottom,
                    ],
                    bounds: Some([
                        bounds.full.left,
                        bounds.full.top,
                        bounds.full.right,
                        bounds.full.bottom,
                    ]),
                    ..Default::default()
                },
            ));
        }
    }
    let data = context.render();
    if data.total_vtx_count == 0 && !measurements.is_empty() {
        return Err("synthetic renderer produced no cue geometry".into());
    }
    if measurements.is_empty() && data.total_vtx_count != 0 {
        return Err("hidden indicator still produced geometry".into());
    }
    // imgui 0.12 constructs a slice from a null pointer for an empty draw list.
    // An intentionally hidden indicator has no lists to inspect or export.
    let draw_lists: Vec<_> = if data.total_vtx_count == 0 {
        Vec::new()
    } else {
        data.draw_lists().collect()
    };
    for vertex in draw_lists.iter().flat_map(|list| list.vtx_buffer().iter()) {
        let contained = measurements.iter().any(|(_, sample)| {
            sample.bounds.is_some_and(|bounds| {
                vertex.pos[0] >= bounds[0] - 0.25
                    && vertex.pos[0] <= bounds[2] + 0.25
                    && vertex.pos[1] >= bounds[1] - 0.25
                    && vertex.pos[1] <= bounds[3] + 0.25
                    && vertex.pos[0] >= sample.viewport[0]
                    && vertex.pos[0] <= sample.viewport[2]
                    && vertex.pos[1] >= sample.viewport[1]
                    && vertex.pos[1] <= sample.viewport[3]
            })
        });
        if !contained {
            return Err(format!("draw vertex {:?} escaped measured cue bounds", vertex.pos).into());
        }
    }

    let mut file = File::create(output.join("mesh.json"))?;
    let label = if indicator_only {
        indicator.unwrap_or("off")
    } else if gallery {
        "gallery"
    } else {
        state
    };
    let version = env!("CARGO_PKG_VERSION");
    write!(
        file,
        "{{\"atlas\":[{aw},{ah}],\"display\":{display:?},\"viewport\":[0,0,{},{}],\"label\":\"{label}\",\"synthetic\":true,\"version\":\"{version}\",\"measurements\":[",
        display[0], display[1]
    )?;
    for (index, (name, sample)) in measurements.iter().enumerate() {
        if index > 0 {
            write!(file, ",")?;
        }
        write!(
            file,
            "{{\"state\":\"{}\",\"decision\":\"{}\",\"reason\":\"{}\",\"layout\":\"{}\",\"position\":{},\"playable_viewport\":{:?},\"bounds\":{}}}",
            name,
            sample.decision.state.label(),
            sample.status,
            sample.layout_mode,
            sample.position.map_or_else(
                || "null".to_string(),
                |value| format!("[{},{}]", value[0], value[1]),
            ),
            sample.viewport,
            sample.bounds.map_or_else(
                || "null".to_string(),
                |value| format!("[{},{},{},{}]", value[0], value[1], value[2], value[3]),
            )
        )?;
    }
    write!(file, "],\"lists\":[")?;
    for (n, list) in draw_lists.iter().enumerate() {
        if n > 0 {
            write!(file, ",")?;
        }
        write!(file, "{{\"vertices\":[")?;
        for (i, vertex) in list.vtx_buffer().iter().enumerate() {
            if i > 0 {
                write!(file, ",")?;
            }
            write!(
                file,
                "[{},{},{},{},{},{},{},{}]",
                vertex.pos[0],
                vertex.pos[1],
                vertex.uv[0],
                vertex.uv[1],
                vertex.col[0],
                vertex.col[1],
                vertex.col[2],
                vertex.col[3]
            )?;
        }
        write!(file, "],\"indices\":{:?},\"commands\":[", list.idx_buffer())?;
        let mut comma = false;
        for command in list.commands() {
            if let DrawCmd::Elements { count, cmd_params } = command {
                if comma {
                    write!(file, ",")?;
                }
                comma = true;
                write!(
                    file,
                    "[{}, {}, {}, {:?}]",
                    count, cmd_params.idx_offset, cmd_params.vtx_offset, cmd_params.clip_rect
                )?;
            }
        }
        write!(file, "]}}")?;
    }
    write!(file, "]}}")?;
    Ok(())
}

fn option<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
}

fn parse_display(args: &[String]) -> Result<Option<[f32; 2]>, &'static str> {
    let Some(index) = args.iter().position(|arg| arg == "--display") else {
        return Ok(None);
    };
    let width = args
        .get(index + 1)
        .ok_or("missing display width")?
        .parse::<f32>()
        .map_err(|_| "invalid display width")?;
    let height = args
        .get(index + 2)
        .ok_or("missing display height")?
        .parse::<f32>()
        .map_err(|_| "invalid display height")?;
    if [width, height]
        .iter()
        .all(|value| value.is_finite() && *value > 0.0)
    {
        Ok(Some([width, height]))
    } else {
        Err("display dimensions must be finite and positive")
    }
}

fn expected_state(name: &str) -> timing::State {
    match name {
        "incoming-parry" | "incoming-dodge" | "incoming-jump" | "incoming-mikiri"
        | "incoming-avoid" | "incoming-unknown" => timing::State::Incoming,
        "active-parry" => timing::State::AttackActive,
        "ready" => timing::State::Preparation,
        "parry" | "dodge" | "jump" => timing::State::Actionable,
        "expired" => timing::State::Expired,
        "watch" | "locked" => timing::State::Neutral,
        _ => timing::State::Hidden,
    }
}

fn synthetic(
    state: &str,
    camera_aspect: f32,
) -> Result<(cue::Target, timing::Decision), Box<dyn std::error::Error>> {
    let (model, animation, time) = match state {
        "incoming-parry" => (1020, 3003, 0.300),
        "active-parry" => (1020, 3003, 0.960),
        "incoming-dodge" => (5020, 100003005, 0.300),
        "incoming-jump" => (5100, 100003009, 0.800),
        "incoming-mikiri" => (1550, 3003, 0.450),
        "incoming-avoid" => (5000, 3005, 0.600),
        "incoming-unknown" => (1020, 3004, 0.500),
        "ready" => (1010, 3000, 0.300),
        "parry" => (1010, 3000, 0.600),
        "dodge" => (5020, 100003005, 0.450),
        "jump" => (5100, 100003009, 1.050),
        "expired" => (1010, 3000, 0.720),
        "watch" => (1550, 3003, 0.450),
        "locked" => (1010, -1, 0.300),
        _ => {
            return Err("state must be ready, parry, dodge, jump, expired, watch, or locked".into())
        }
    };
    let camera = cue::Camera {
        right: [1.0, 0.0, 0.0],
        up: [0.0, 1.0, 0.0],
        forward: [0.0, 0.0, 1.0],
        position: [0.0; 3],
        fov: 1.0,
        aspect: camera_aspect,
        near: 0.08,
        far: 1000.0,
    };
    let mut target = cue::Target {
        metadata: Default::default(),
        captured_at: Some(Duration::ZERO),
        player_instance: 1,
        animation_module: 2,
        handle: 3,
        model,
        npc_param: None,
        animation: cue::Animation {
            id: animation,
            previous: time - 0.048,
            time: time - 0.032,
            sequence: 1,
        },
        position: [0.0, 0.0, 2.0],
        anchor: [0.0, 1.0, 5.0],
        facing: [0.0, 0.0, -1.0],
        player_position: [0.0; 3],
        body_radius: 0.0,
        camera: Some(camera),
        animation_error: None,
    };
    let mut engine = timing::Engine::default();
    for (step, milliseconds) in [0_u64, 16, 32].into_iter().enumerate() {
        let at = Duration::from_millis(milliseconds);
        target.captured_at = Some(at);
        target.animation.previous = target.animation.time;
        target.animation.time = time - 0.032 + step as f32 * 0.016;
        target.animation.sequence = step as i32 + 1;
        engine.observe(at, Some(&target));
    }
    let decision = if state.starts_with("incoming-") || state.starts_with("active-") {
        engine.incoming(Duration::from_millis(32), [true; 3], true)
    } else {
        engine.decide(Duration::from_millis(32))
    };
    Ok((target, decision))
}
