//! Glass's stages on the GPU, for the DirectX renderer. The shader is the
//! glass crate's (`../gpui_glass/src/glass.hlsl`); this pass is the
//! renderer's, so it lives with the renderer.
//!
//! Each [`Backdrop`] copies what has been drawn under it, shrinks the copy,
//! blurs it both ways, and draws the shape from the result: the rim bends the
//! sample inward, then saturation, tint and edge light go on. What to copy and
//! the numbers each draw reads are the glass crate's (`GlassPlan`), the same
//! for every renderer; this file is the DirectX of it. The textures are made
//! by the first backdrop drawn, so a window without glass pays nothing.

use std::slice;

use anyhow::{Context, Result};
use gpui::{Backdrop, Rgba};
use gpui_glass::plan::{GlassParam, GlassPlan, GlassRequest, STAGE_GLASS};
use windows::Win32::Graphics::{Direct3D::*, Direct3D11::*, Dxgi::Common::*};

use crate::directx_renderer::shader_resources::{RawShaderBytes, ShaderModule, ShaderTarget};
use crate::directx_renderer::{
    RENDER_TARGET_FORMAT, create_fragment_shader, create_vertex_shader, update_buffer,
};

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
            ByteWidth: size_of::<GlassParam>() as u32,
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
        let (style, bounds, radii) = (&backdrop.style, &backdrop.bounds, backdrop.corner_radii);
        let (tint, edge) = (Rgba::from(style.tint), Rgba::from(style.edge));
        let request = GlassRequest {
            bounds: [bounds.origin.x.0, bounds.origin.y.0, bounds.size.width.0, bounds.size.height.0],
            visible: [
                visible.origin.x.0,
                visible.origin.y.0,
                visible.origin.x.0 + visible.size.width.0,
                visible.origin.y.0 + visible.size.height.0,
            ],
            radii: [radii.top_left.0, radii.top_right.0, radii.bottom_right.0, radii.bottom_left.0],
            blur_radius: style.blur_radius,
            saturation: style.saturation,
            lens: style.lens,
            tint: [tint.r, tint.g, tint.b, tint.a * style.tint_amount],
            edge: [edge.r, edge.g, edge.b, edge.a * style.edge_strength],
            opacity: backdrop.opacity,
            scale_factor: backdrop.scale_factor,
        };
        // Nothing of it shows: nothing is copied and nothing drawn.
        let Some(plan) = GlassPlan::of(&request, layers.width, layers.height) else {
            return Ok(());
        };
        let [l, t, r, b] = plan.capture;
        let source = D3D11_BOX { left: l, top: t, front: 0, right: r, bottom: b, back: 1 };
        unsafe {
            context.CopySubresourceRegion(&layers.capture.texture, 0, l, t, 0, target, 0, Some(&source));
        }
        let [small_w, small_h] = plan.small;
        let small_viewport = D3D11_VIEWPORT { Width: small_w, Height: small_h, MaxDepth: 1.0, ..Default::default() };
        let [shrink, across, down, shape] = &plan.stage;
        // Shrink, then blur across and down, between the two small layers.
        self.stage(context, &layers.capture, &layers.blurred[0].write, &small_viewport, shrink)?;
        self.stage(context, &layers.blurred[0], &layers.blurred[1].write, &small_viewport, across)?;
        self.stage(context, &layers.blurred[1], &layers.blurred[0].write, &small_viewport, down)?;
        // The shape, back on the window.
        unsafe { context.OMSetBlendState(&self.blend, None, 0xFFFFFFFF) };
        let result = self.stage(context, &layers.blurred[0], target_view, viewport, shape);
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
        params: &GlassParam,
    ) -> Result<()> {
        update_buffer(context, self.params.as_ref().context("backdrop params missing")?, slice::from_ref(params))?;
        unsafe {
            // A texture can't be read and written at once, so let go of the
            // last read before binding what it may become.
            context.PSSetShaderResources(0, Some(&[None]));
            if params.stage != STAGE_GLASS {
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

#[cfg(test)]
#[path = "directx_backdrop_tests.rs"]
mod tests;
