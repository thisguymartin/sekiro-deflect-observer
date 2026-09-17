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
        pub at: std::time::Duration,
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
        let (mut target, decision) = synthetic(state, camera_aspect)?;
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
        let mut submitted = diagnostics::RenderSample {
            at: target.captured_at.unwrap_or_default(),
            ..Default::default()
        };
        if args.iter().any(|arg| arg == "--practice") {
            submitted.practice = practice::Snapshot {
                status: practice::Status::Active,
                handle: target.handle,
                model: target.model,
                animation_module: target.animation_module,
                percent: 80,
                ..Default::default()
            };
        }
        cue_draw::draw(ui, &target, &decision, &config, &fonts, &mut submitted);
        measurements.push((state.to_string(), submitted));
    }
    let data = context.render();
    if data.total_vtx_count == 0 {
        return Err("synthetic renderer produced no cue geometry".into());
    }
    for vertex in data.draw_lists().flat_map(|list| list.vtx_buffer().iter()) {
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
    let label = if gallery { "gallery" } else { state };
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
    for (n, list) in data.draw_lists().enumerate() {
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
