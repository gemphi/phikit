//! Glass in the DirectX renderer: appkit's `src/glass` stages on the GPU.
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
        let (small_w, small_h) = (full_w.div_ceil_half(), full_h.div_ceil_half());
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

trait HalfUp {
    fn div_ceil_half(self) -> f32;
}

impl HalfUp for f32 {
    /// The blurred layers' side for a window side of this.
    fn div_ceil_half(self) -> f32 {
        (self as u32).div_ceil(2).max(1) as f32
    }
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
mod tests {
    use super::*;
    use gpui::{BackdropStyle, Bounds, ContentMask, Corners, Hsla, ScaledPixels, point, size};
    use windows::Win32::Foundation::HMODULE;

    #[test]
    fn the_capture_reaches_past_the_shape_and_stays_in_the_window() {
        assert_eq!(capture_region([100.0, 100.0, 200.0, 150.0], 10.0, 4, 1000, 1000), [88, 88, 212, 160]);
        assert_eq!(capture_region([0.0, 0.0, 998.0, 999.0], 10.0, 4, 1000, 1000), [0, 0, 1000, 1000]);
    }

    #[test]
    fn a_wide_blur_shrinks_further() {
        assert_eq!(shrink_for(8.0), 2);
        assert_eq!(shrink_for(24.0), 4);
    }

    const SIDE: (u32, u32) = (200, 100);
    const RED: [u8; 4] = [0, 0, 255, 255];
    const BLUE: [u8; 4] = [255, 0, 0, 255];

    /// Glass over the seam of a window that is red on the left and blue on the
    /// right, drawn by a real device, and the window's pixels back.
    fn drawn(style: BackdropStyle) -> Vec<[u8; 4]> {
        let (mut device, mut context) = (None, None);
        unsafe {
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_WARP,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                Some(&mut context),
            )
            .unwrap();
        }
        let (device, context): (ID3D11Device, ID3D11DeviceContext) = (device.unwrap(), context.unwrap());
        let window = Layer::new(&device, SIDE.0, SIDE.1, true).unwrap();
        let pixels: Vec<[u8; 4]> = (0..SIDE.0 * SIDE.1)
            .map(|i| if i % SIDE.0 < SIDE.0 / 2 { RED } else { BLUE })
            .collect();
        unsafe {
            context.UpdateSubresource(&window.texture, 0, None, pixels.as_ptr() as *const _, SIDE.0 * 4, 0);
        }
        let viewport = D3D11_VIEWPORT { Width: SIDE.0 as f32, Height: SIDE.1 as f32, MaxDepth: 1.0, ..Default::default() };
        let bounds = Bounds::new(
            point(ScaledPixels(60.0), ScaledPixels(20.0)),
            size(ScaledPixels(80.0), ScaledPixels(60.0)),
        );
        let backdrop = Backdrop {
            order: 0,
            bounds,
            content_mask: ContentMask { bounds },
            corner_radii: Corners::all(ScaledPixels(8.0)),
            style,
            opacity: 1.0,
            scale_factor: 1.0,
        };
        let mut pass = BackdropPass::new(&device).unwrap();
        pass.draw(&device, &context, &window.texture, &window.write, &viewport, &[backdrop]).unwrap();

        let desc = D3D11_TEXTURE2D_DESC {
            Width: SIDE.0,
            Height: SIDE.1,
            MipLevels: 1,
            ArraySize: 1,
            Format: RENDER_TARGET_FORMAT,
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            Usage: D3D11_USAGE_STAGING,
            BindFlags: 0,
            CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
            MiscFlags: 0,
        };
        let mut staging = None;
        unsafe {
            device.CreateTexture2D(&desc, None, Some(&mut staging)).unwrap();
            let staging = staging.unwrap();
            context.CopyResource(&staging, &window.texture);
            let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
            context.Map(&staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped)).unwrap();
            let read = (0..SIDE.1)
                .flat_map(|y| {
                    let row = (mapped.pData as *const u8).add((y * mapped.RowPitch) as usize) as *const [u8; 4];
                    std::slice::from_raw_parts(row, SIDE.0 as usize).to_vec()
                })
                .collect();
            context.Unmap(&staging, 0);
            read
        }
    }

    fn at(pixels: &[[u8; 4]], x: u32, y: u32) -> [u8; 4] {
        pixels[(y * SIDE.0 + x) as usize]
    }

    fn plain() -> BackdropStyle {
        BackdropStyle { blur_radius: 24.0, saturation: 1.0, ..Default::default() }
    }

    #[test]
    fn what_is_under_the_glass_is_blurred_and_what_is_outside_is_not() {
        let pixels = drawn(plain());
        let [blue, _, red, alpha] = at(&pixels, 100, 50);
        assert!(red > 60 && blue > 60, "the seam under the glass mixes both sides: red {red}, blue {blue}");
        assert_eq!(alpha, 255, "glass keeps the window's alpha");
        assert_eq!(at(&pixels, 20, 50), RED, "left of the glass is untouched");
        assert_eq!(at(&pixels, 180, 50), BLUE, "right of the glass is untouched");
        assert_eq!(at(&pixels, 100, 5), BLUE, "above the glass is untouched");
    }

    #[test]
    fn the_tint_covers_what_shows_through() {
        let tint = Hsla { h: 1.0 / 3.0, s: 1.0, l: 0.5, a: 1.0 };
        let pixels = drawn(BackdropStyle { tint, tint_amount: 1.0, ..plain() });
        let [blue, green, red, _] = at(&pixels, 100, 50);
        assert!(green > 240 && red < 10 && blue < 10, "all tint: {red} {green} {blue}");
    }

    #[test]
    fn the_rim_catches_the_light_where_it_faces_it() {
        let edge = Hsla { h: 0.0, s: 0.0, l: 1.0, a: 1.0 };
        let lit = drawn(BackdropStyle { edge, edge_strength: 1.0, ..plain() });
        let unlit = drawn(plain());
        // Just inside the top edge, on the left half where blue starts near zero.
        let (top_lit, top_unlit) = (at(&lit, 80, 21), at(&unlit, 80, 21));
        assert!(top_lit[1] > top_unlit[1] + 40, "the top rim lights: {top_lit:?} against {top_unlit:?}");
    }
}
