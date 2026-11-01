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
