//! This module's stages on the GPU, for the DirectX renderer. `gpui_windows`
//! compiles this file and `glass.hlsl` in, since it needs them and appkit's
//! own crate is not on the workspace's stack yet.
//!
//! Each [`Backdrop`] copies what has been drawn under it, shrinks the copy,
//! blurs it both ways, and draws the shape from the result: the rim bends the
//! sample inward, then saturation, tint and edge light go on. The textures are
//! made by the first backdrop drawn, so a window without glass pays nothing.

use std::slice;

use anyhow::{Context, Result};
use gpui::{Backdrop, Rgba};
use windows::Win32::Graphics::{Direct3D::*, Direct3D11::*, Dxgi::Common::*};

use crate::directx_renderer::shader_resources::{RawShaderBytes, ShaderModule, ShaderTarget};
use crate::directx_renderer::{
    RENDER_TARGET_FORMAT, create_fragment_shader, create_vertex_shader, update_buffer,
};

/// How far in from the edge the rim bends light and catches it, in logical px.
const RIM_BAND: f32 = 38.0;
/// A Gaussian is flat past three sigmas, and the sigma is half the blur.
const BLUR_REACH: f32 = 1.5;

const PASS_SHRINK: u32 = 0;
const PASS_BLUR: u32 = 1;
const PASS_GLASS: u32 = 2;

#[derive(Clone, Copy, Default)]
#[repr(C)]
struct BackdropParams {
    rect: [f32; 4],
    target_size: [f32; 2],
    pixel_to_uv: [f32; 2],
    uv_limit: [f32; 4],
    step: [f32; 2],
    sigma: f32,
    pass: u32,
    bounds: [f32; 4],
    radii: [f32; 4],
    tint: [f32; 4],
    edge: [f32; 4],
    saturation: f32,
    lens: f32,
    opacity: f32,
    rim_band: f32,
}

/// A texture the pass reads, and may draw into.
struct Layer {
    texture: ID3D11Texture2D,
    read: Option<ID3D11ShaderResourceView>,
    write: Option<ID3D11RenderTargetView>,
}

struct Layers {
    width: u32,
    height: u32,
    capture: Layer,
    /// Half the window each, the most any shrink needs.
    blurred: [Layer; 2],
}

pub(crate) struct BackdropPass {
    vertex: ID3D11VertexShader,
    fragment: ID3D11PixelShader,
    params: Option<ID3D11Buffer>,
    sampler: Option<ID3D11SamplerState>,
    blend: ID3D11BlendState,
    layers: Option<Layers>,
}

impl BackdropPass {
    pub(crate) fn new(device: &ID3D11Device) -> Result<Self> {
        let vertex = RawShaderBytes::new(ShaderModule::Backdrop, ShaderTarget::Vertex)?;
        let fragment = RawShaderBytes::new(ShaderModule::Backdrop, ShaderTarget::Fragment)?;
        let params_desc = D3D11_BUFFER_DESC {
            ByteWidth: size_of::<BackdropParams>() as u32,
            Usage: D3D11_USAGE_DYNAMIC,
            BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as u32,
            CPUAccessFlags: D3D11_CPU_ACCESS_WRITE.0 as u32,
            ..Default::default()
        };
        let sampler_desc = D3D11_SAMPLER_DESC {
            Filter: D3D11_FILTER_MIN_MAG_MIP_LINEAR,
            AddressU: D3D11_TEXTURE_ADDRESS_CLAMP,
            AddressV: D3D11_TEXTURE_ADDRESS_CLAMP,
            AddressW: D3D11_TEXTURE_ADDRESS_CLAMP,
            MaxAnisotropy: 1,
            ComparisonFunc: D3D11_COMPARISON_ALWAYS,
            MaxLOD: D3D11_FLOAT32_MAX,
            ..Default::default()
        };
        // Colour blends by coverage. Alpha stays as it was: glass over a
        // translucent window must not make the window opaque.
        let mut blend_desc = D3D11_BLEND_DESC::default();
        blend_desc.RenderTarget[0].BlendEnable = true.into();
        blend_desc.RenderTarget[0].BlendOp = D3D11_BLEND_OP_ADD;
        blend_desc.RenderTarget[0].BlendOpAlpha = D3D11_BLEND_OP_ADD;
        blend_desc.RenderTarget[0].SrcBlend = D3D11_BLEND_SRC_ALPHA;
        blend_desc.RenderTarget[0].DestBlend = D3D11_BLEND_INV_SRC_ALPHA;
        blend_desc.RenderTarget[0].SrcBlendAlpha = D3D11_BLEND_ZERO;
        blend_desc.RenderTarget[0].DestBlendAlpha = D3D11_BLEND_ONE;
        blend_desc.RenderTarget[0].RenderTargetWriteMask = D3D11_COLOR_WRITE_ENABLE_ALL.0 as u8;

        let (mut params, mut sampler, mut blend) = (None, None, None);
        unsafe {
            device.CreateBuffer(&params_desc, None, Some(&mut params))?;
            device.CreateSamplerState(&sampler_desc, Some(&mut sampler))?;
            device.CreateBlendState(&blend_desc, Some(&mut blend))?;
        }
        Ok(Self {
            vertex: create_vertex_shader(device, vertex.as_bytes())?,
            fragment: create_fragment_shader(device, fragment.as_bytes())?,
            params,
            sampler,
            blend: blend.context("backdrop blend state")?,
            layers: None,
        })
    }

    /// Draw `backdrops` onto `target`, leaving `target_view` bound.
    pub(crate) fn draw(
        &mut self,
        device: &ID3D11Device,
        context: &ID3D11DeviceContext,
        target: &ID3D11Texture2D,
        target_view: &Option<ID3D11RenderTargetView>,
        viewport: &D3D11_VIEWPORT,
        backdrops: &[Backdrop],
    ) -> Result<()> {
        let (width, height) = (viewport.Width as u32, viewport.Height as u32);
        if !self.layers.as_ref().is_some_and(|l| (l.width, l.height) == (width, height)) {
            self.layers = Some(Layers::new(device, width, height)?);
        }
        unsafe {
            context.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP);
            context.VSSetShader(&self.vertex, None);
            context.PSSetShader(&self.fragment, None);
            context.VSSetConstantBuffers(1, Some(slice::from_ref(&self.params)));
            context.PSSetConstantBuffers(1, Some(slice::from_ref(&self.params)));
            context.PSSetSamplers(0, Some(slice::from_ref(&self.sampler)));
        }
        for backdrop in backdrops {
            self.draw_one(context, target, target_view, viewport, backdrop)?;
        }
        Ok(())
    }

    fn draw_one(
        &self,
        context: &ID3D11DeviceContext,
        target: &ID3D11Texture2D,
        target_view: &Option<ID3D11RenderTargetView>,
        viewport: &D3D11_VIEWPORT,
        backdrop: &Backdrop,
    ) -> Result<()> {
        let layers = self.layers.as_ref().context("backdrop layers missing")?;
        let visible = backdrop.bounds.intersect(&backdrop.content_mask.bounds);
        if visible.is_empty() || backdrop.opacity <= 0.0 {
            return Ok(());
        }
        let style = &backdrop.style;
        let (x, y) = (backdrop.bounds.origin.x.0, backdrop.bounds.origin.y.0);
        let (w, h) = (backdrop.bounds.size.width.0, backdrop.bounds.size.height.0);
        let half_side = w.min(h) * 0.5;
        let blur = style.blur_radius * backdrop.scale_factor;
        let lens = (style.lens * backdrop.scale_factor).min(half_side);
        let shrink = shrink_for(blur);

        let shown = [
            visible.origin.x.0,
            visible.origin.y.0,
            visible.origin.x.0 + visible.size.width.0,
            visible.origin.y.0 + visible.size.height.0,
        ];
        let margin = blur * BLUR_REACH + lens;
        let [l, t, r, b] = capture_region(shown, margin, shrink, layers.width, layers.height);
        let source = D3D11_BOX { left: l, top: t, front: 0, right: r, bottom: b, back: 1 };
        unsafe {
            context.CopySubresourceRegion(&layers.capture.texture, 0, l, t, 0, target, 0, Some(&source));
        }

        let (full_w, full_h) = (layers.width as f32, layers.height as f32);
        let (small_w, small_h) = (half_up(layers.width), half_up(layers.height));
        let scale = shrink as f32;
        let small_rect = [l as f32 / scale, t as f32 / scale, (r - l) as f32 / scale, (b - t) as f32 / scale];
        let small_size = [small_w, small_h];
        let small_limit = limit(small_rect, small_w, small_h);
        let small_viewport = D3D11_VIEWPORT { Width: small_w, Height: small_h, MaxDepth: 1.0, ..Default::default() };
        let base = BackdropParams { rect: small_rect, target_size: small_size, ..Default::default() };

        // Shrink: four taps a quarter of the shrink out each cover a quarter of it.
        self.stage(context, &layers.capture, &layers.blurred[0].write, &small_viewport, &BackdropParams {
            pass: PASS_SHRINK,
            pixel_to_uv: [scale / full_w, scale / full_h],
            uv_limit: limit([l as f32, t as f32, (r - l) as f32, (b - t) as f32], full_w, full_h),
            step: [scale * 0.25 / full_w, scale * 0.25 / full_h],
            ..base
        })?;
        // Blur, across and then down.
        let blurring = BackdropParams {
            pass: PASS_BLUR,
            pixel_to_uv: [1.0 / small_w, 1.0 / small_h],
            uv_limit: small_limit,
            sigma: blur * 0.5 / scale,
            ..base
        };
        self.stage(context, &layers.blurred[0], &layers.blurred[1].write, &small_viewport, &BackdropParams {
            step: [1.0 / small_w, 0.0],
            ..blurring
        })?;
        self.stage(context, &layers.blurred[1], &layers.blurred[0].write, &small_viewport, &BackdropParams {
            step: [0.0, 1.0 / small_h],
            ..blurring
        })?;

        // The shape, back on the window.
        let tint = Rgba::from(style.tint);
        let edge = Rgba::from(style.edge);
        let radii = backdrop.corner_radii;
        unsafe { context.OMSetBlendState(&self.blend, None, 0xFFFFFFFF) };
        let result = self.stage(context, &layers.blurred[0], target_view, viewport, &BackdropParams {
            pass: PASS_GLASS,
            rect: [shown[0], shown[1], shown[2] - shown[0], shown[3] - shown[1]],
            target_size: [full_w, full_h],
            pixel_to_uv: [1.0 / (scale * small_w), 1.0 / (scale * small_h)],
            uv_limit: small_limit,
            bounds: [x, y, w, h],
            radii: [radii.top_left.0, radii.top_right.0, radii.bottom_right.0, radii.bottom_left.0],
            tint: [tint.r, tint.g, tint.b, tint.a * style.tint_amount],
            edge: [edge.r, edge.g, edge.b, edge.a * style.edge_strength],
            saturation: style.saturation,
            lens,
            opacity: backdrop.opacity,
            rim_band: (RIM_BAND * backdrop.scale_factor).min(half_side).max(1.0),
            ..Default::default()
        });
        unsafe { context.PSSetShaderResources(0, Some(&[None])) };
        result
    }

    /// One draw: `source` read, `into` written.
    fn stage(
        &self,
        context: &ID3D11DeviceContext,
        source: &Layer,
        into: &Option<ID3D11RenderTargetView>,
        viewport: &D3D11_VIEWPORT,
        params: &BackdropParams,
    ) -> Result<()> {
        update_buffer(context, self.params.as_ref().context("backdrop params missing")?, slice::from_ref(params))?;
        unsafe {
            // A texture can't be read and written at once, so let go of the
            // last read before binding what it may become.
            context.PSSetShaderResources(0, Some(&[None]));
            if params.pass != PASS_GLASS {
                context.OMSetBlendState(None, None, 0xFFFFFFFF);
            }
            context.OMSetRenderTargets(Some(slice::from_ref(into)), None);
            context.RSSetViewports(Some(slice::from_ref(viewport)));
            context.PSSetShaderResources(0, Some(slice::from_ref(&source.read)));
            context.Draw(4, 0);
        }
        Ok(())
    }
}

impl Layers {
    fn new(device: &ID3D11Device, width: u32, height: u32) -> Result<Self> {
        let (small_w, small_h) = (width.div_ceil(2).max(1), height.div_ceil(2).max(1));
        Ok(Self {
            width,
            height,
            capture: Layer::new(device, width, height, false)?,
            blurred: [Layer::new(device, small_w, small_h, true)?, Layer::new(device, small_w, small_h, true)?],
        })
    }
}

impl Layer {
    fn new(device: &ID3D11Device, width: u32, height: u32, writable: bool) -> Result<Self> {
        let write_flag = if writable { D3D11_BIND_RENDER_TARGET.0 as u32 } else { 0 };
        let desc = D3D11_TEXTURE2D_DESC {
            Width: width,
            Height: height,
            MipLevels: 1,
            ArraySize: 1,
            Format: RENDER_TARGET_FORMAT,
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32 | write_flag,
            CPUAccessFlags: 0,
            MiscFlags: 0,
        };
        let (mut texture, mut read, mut write) = (None, None, None);
        unsafe {
            device.CreateTexture2D(&desc, None, Some(&mut texture))?;
            let made = texture.as_ref().context("backdrop texture")?;
            device.CreateShaderResourceView(made, None, Some(&mut read))?;
            if writable {
                device.CreateRenderTargetView(made, None, Some(&mut write))?;
            }
        }
        Ok(Self { texture: texture.context("backdrop texture")?, read, write })
    }
}

/// The blurred layers' side for a window side of this.
fn half_up(side: u32) -> f32 {
    side.div_ceil(2).max(1) as f32
}

/// How much smaller the copy is. A wide blur loses nothing at a quarter size
/// and costs a sixteenth as much.
fn shrink_for(blur: f32) -> u32 {
    match blur >= 12.0 {
        true => 4,
        false => 2,
    }
}

/// What to copy: `shown` and `margin` around it, inside the window, on whole
/// texels of the shrunk copy.
fn capture_region(shown: [f32; 4], margin: f32, shrink: u32, width: u32, height: u32) -> [u32; 4] {
    let down = |v: f32| (v - margin).max(0.0) as u32 / shrink * shrink;
    let up = |v: f32, side: u32| ((v + margin).ceil() as u32).div_ceil(shrink).saturating_mul(shrink).min(side);
    [down(shown[0]), down(shown[1]), up(shown[2], width), up(shown[3], height)]
}

/// The first and last texel centres of `rect`, as texture coordinates.
fn limit(rect: [f32; 4], width: f32, height: f32) -> [f32; 4] {
    [
        (rect[0] + 0.5) / width,
        (rect[1] + 0.5) / height,
        (rect[0] + rect[2] - 0.5) / width,
        (rect[1] + rect[3] - 0.5) / height,
    ]
}

#[cfg(test)]
#[path = "directx_tests.rs"]
mod tests;
