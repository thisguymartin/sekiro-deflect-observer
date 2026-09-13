//! Emit the actual ImGui draw mesh for a reproducible, offline visual check.
//! No game reads, hooks, DLL loading, or input generation.
use hudhook::imgui::{self, DrawCmd};
use sekiro_deflect_observer::cue;
use std::fs::{self, File};
use std::io::Write;

mod diagnostics {
    use super::cue;
    #[derive(Default)]
    pub(super) struct RenderSample {
        pub status: &'static str,
        pub position: Option<[f32; 2]>,
        pub phase: Option<cue::Timeline>,
    }
}
#[path = "../src/windows/cue_draw.rs"]
mod cue_draw;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args_os()
        .nth(1)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| "dist/review-0.6/layout".into());
    fs::create_dir_all(&output)?;
    let gallery = std::env::args().any(|arg| arg == "--gallery");
    let display = if gallery {
        [960.0, 540.0]
    } else {
        [1920.0, 1080.0]
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
    let camera = cue::Camera {
        right: [1.0, 0.0, 0.0],
        up: [0.0, 1.0, 0.0],
        forward: [0.0, 0.0, 1.0],
        position: [0.0; 3],
        fov: 1.0,
        aspect: display[0] / display[1],
        near: 0.08,
        far: 1000.0,
    };
    for (row, (model, animation, time)) in [
        (1010, 3000, 0.3),
        (1010, 3000, 0.60),
        (5020, 100003005, 0.45),
        (5100, 100003009, 1.05),
    ]
    .into_iter()
    .enumerate()
    {
        let y = if gallery {
            105.0 + row as f32 * 110.0
        } else {
            180.0 + row as f32 * 230.0
        };
        let anchor_y = (1.0 - y / (display[1] / 2.0)) * 5.0 * (0.5_f32).tan();
        let target = cue::Target {
            handle: 1,
            model,
            animation: cue::Animation {
                id: animation,
                previous: time - 0.016,
                time,
                sequence: 1,
            },
            position: [0.0, 0.0, 2.0],
            anchor: [0.0, anchor_y, 5.0],
            facing: [0.0, 0.0, -1.0],
            player_position: [0.0; 3],
            body_radius: 0.0,
            camera,
            animation_error: None,
        };
        cue_draw::draw(
            ui,
            &target,
            true,
            0,
            &fonts,
            &mut diagnostics::RenderSample::default(),
        );
    }
    let data = context.render();
    if data.total_vtx_count == 0 {
        return Err("The offline camera produced no drawable cue geometry".into());
    }
    let mut file = File::create(output.join("mesh.json"))?;
    write!(
        file,
        "{{\"atlas\":[{aw},{ah}],\"display\":{display:?},\"lists\":["
    )?;
    for (n, list) in data.draw_lists().enumerate() {
        if n > 0 {
            write!(file, ",")?;
        }
        write!(file, "{{\"vertices\":[")?;
        for (i, v) in list.vtx_buffer().iter().enumerate() {
            if i > 0 {
                write!(file, ",")?;
            }
            write!(
                file,
                "[{},{},{},{},{},{},{},{}]",
                v.pos[0], v.pos[1], v.uv[0], v.uv[1], v.col[0], v.col[1], v.col[2], v.col[3]
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
