//! Exercise the production DX11 backend on an offscreen WARP device. No game,
//! window, injection, or desktop capture is involved.
#![cfg(windows)]

use hudhook::imgui::{Context, DrawData};
use hudhook::windows::core::Result;
use hudhook::windows::Win32::Foundation::HMODULE;
use hudhook::windows::Win32::Graphics::Direct3D::*;
use hudhook::windows::Win32::Graphics::Direct3D11::*;
use hudhook::windows::Win32::Graphics::Dxgi::Common::*;
pub use hudhook::{util, RenderContext};

// The backend's internal trait is deliberately small. Include the exact file
// used by the dependency so these checks exercise its real GPU commands.
mod renderer {
    use super::*;
    pub trait RenderEngine: RenderContext {
        type RenderTarget;
        fn render(&mut self, data: &DrawData, target: Self::RenderTarget) -> Result<()>;
        fn setup_fonts(&mut self, ctx: &mut Context) -> Result<()>;
    }
}
#[allow(clippy::all)]
mod backend {
    use hudhook::{imgui, tracing, windows};
    include!("../vendor/hudhook/src/renderer/backend/dx11.rs");
}
use renderer::RenderEngine;

const WIDTH: u32 = 128;
const HEIGHT: u32 = 96;

unsafe fn texture(device: &ID3D11Device, format: DXGI_FORMAT) -> Result<ID3D11Texture2D> {
    util::try_out_ptr(|out| {
        device.CreateTexture2D(
            &D3D11_TEXTURE2D_DESC {
                Width: WIDTH,
                Height: HEIGHT,
                MipLevels: 1,
                ArraySize: 1,
                Format: format,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                Usage: D3D11_USAGE_DEFAULT,
                BindFlags: (D3D11_BIND_RENDER_TARGET | D3D11_BIND_SHADER_RESOURCE).0 as u32,
                ..Default::default()
            },
            None,
            Some(out),
        )
    })
}

unsafe fn pixels(
    device: &ID3D11Device,
    context: &ID3D11DeviceContext,
    source: &ID3D11Texture2D,
) -> Result<Vec<[u8; 4]>> {
    let mut desc = D3D11_TEXTURE2D_DESC::default();
    source.GetDesc(&mut desc);
    desc.Usage = D3D11_USAGE_STAGING;
    desc.BindFlags = 0;
    desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ.0 as u32;
    let staging: ID3D11Texture2D =
        util::try_out_ptr(|out| device.CreateTexture2D(&desc, None, Some(out)))?;
    context.CopyResource(&staging, source);
    let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
    context.Map(&staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped))?;
    let mut result = Vec::new();
    for y in 0..HEIGHT {
        let row = std::slice::from_raw_parts(
            mapped
                .pData
                .cast::<u8>()
                .add((y * mapped.RowPitch) as usize),
            (WIDTH * 4) as usize,
        );
        result.extend(row.chunks_exact(4).map(|p| [p[0], p[1], p[2], p[3]]));
    }
    context.Unmap(&staging, 0);
    Ok(result)
}

#[test]
fn overlay_preserves_host_bindings_and_pixels_outside_the_cue() -> Result<()> {
    unsafe {
        let mut device = None;
        let mut context = None;
        D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_WARP,
            HMODULE(std::ptr::null_mut()),
            D3D11_CREATE_DEVICE_FLAG(0),
            Some(&[D3D_FEATURE_LEVEL_11_0]),
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            Some(&mut context),
        )?;
        let device = device.unwrap();
        let context = context.unwrap();
        let mut imgui = Context::create();
        imgui.set_ini_filename(None);
        imgui.io_mut().display_size = [WIDTH as f32, HEIGHT as f32];
        imgui.io_mut().delta_time = 1.0 / 60.0;
        let mut renderer = backend::D3D11RenderEngine::new(&device, &mut imgui)?;
        renderer.setup_fonts(&mut imgui)?;

        for format in [DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_FORMAT_R8G8B8A8_UNORM_SRGB] {
            let target = texture(&device, format)?;
            let target_view: ID3D11RenderTargetView =
                util::try_out_ptr(|out| device.CreateRenderTargetView(&target, None, Some(out)))?;
            let host = texture(&device, format)?;
            let host_view: ID3D11RenderTargetView =
                util::try_out_ptr(|out| device.CreateRenderTargetView(&host, None, Some(out)))?;
            let alias: ID3D11ShaderResourceView =
                util::try_out_ptr(|out| device.CreateShaderResourceView(&target, None, Some(out)))?;
            context.ClearRenderTargetView(&target_view, &[0.2, 0.4, 0.7, 1.0]);
            let before = pixels(&device, &context, &target)?;

            // Include bindings implicitly unbound when the backbuffer becomes
            // an output, plus deliberately NULL bindings the old backup skipped.
            context.ClearState();
            context.OMSetRenderTargets(Some(&[Some(host_view.clone())]), None);
            context.PSSetShaderResources(7, Some(&[Some(alias.clone())]));
            context.VSSetShaderResources(5, Some(&[Some(alias.clone())]));
            context.RSSetViewports(Some(&[D3D11_VIEWPORT {
                TopLeftX: 3.0,
                TopLeftY: 4.0,
                Width: 80.0,
                Height: 60.0,
                MinDepth: 0.2,
                MaxDepth: 0.8,
            }]));

            for visible in [true, true, false] {
                let frame_before = pixels(&device, &context, &target)?;
                let ui = imgui.frame();
                if visible {
                    ui.get_background_draw_list()
                        .add_rect([40.0, 30.0], [80.0, 50.0], [0.0, 1.0, 0.0, 1.0])
                        .filled(true)
                        .build();
                }
                renderer.render(imgui.render(), target.clone())?;

                let mut restored_target = [None];
                context.OMGetRenderTargets(Some(&mut restored_target), None);
                assert_eq!(
                    restored_target[0].as_ref(),
                    Some(&host_view),
                    "host render target leaked"
                );
                let mut resource = [None];
                context.PSGetShaderResources(7, Some(&mut resource));
                assert_eq!(resource[0].as_ref(), Some(&alias), "PS alias was unbound");
                context.VSGetShaderResources(5, Some(&mut resource));
                assert_eq!(resource[0].as_ref(), Some(&alias), "VS alias was unbound");
                context.PSGetShaderResources(0, Some(&mut resource));
                assert!(resource[0].is_none(), "font texture leaked into game state");
                let mut sampler = [None];
                context.PSGetSamplers(0, Some(&mut sampler));
                assert!(sampler[0].is_none(), "overlay sampler leaked");
                let mut constant = [None];
                context.VSGetConstantBuffers(0, Some(&mut constant));
                assert!(constant[0].is_none(), "overlay projection leaked");
                assert!(
                    context.IAGetInputLayout().is_err(),
                    "overlay input layout leaked"
                );
                let mut viewport = [D3D11_VIEWPORT::default()];
                let mut count = 1;
                context.RSGetViewports(&mut count, Some(viewport.as_mut_ptr()));
                assert_eq!(viewport[0].TopLeftX, 3.0);
                assert_eq!(viewport[0].MinDepth, 0.2);

                let after = pixels(&device, &context, &target)?;
                if visible {
                    assert_ne!(
                        after[(40 * WIDTH + 60) as usize],
                        before[(40 * WIDTH + 60) as usize],
                        "cue was not rendered"
                    );
                    for y in 0..HEIGHT {
                        for x in 0..WIDTH {
                            if !(38..82).contains(&x) || !(28..52).contains(&y) {
                                let i = (y * WIDTH + x) as usize;
                                assert_eq!(
                                    after[i], before[i],
                                    "pixel changed outside cue at {x},{y}"
                                );
                            }
                        }
                    }
                } else {
                    assert_eq!(after, frame_before, "hidden frame changed the image");
                }
            }
        }
    }
    Ok(())
}
