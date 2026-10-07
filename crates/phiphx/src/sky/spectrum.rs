//! The spectral tables the atmosphere is integrated over, from Blender's
//! `intern/sky/source/sky_single_scattering.cpp`: 21 wavelengths from 380 nm
//! to 780 nm, the sun's irradiance at the top of the atmosphere, Rayleigh
//! and ozone coefficients at each, and the CIE XYZ colour matching
//! functions that turn a spectrum into a colour.

/// Wavelengths sampled (`NUM_WAVELENGTHS`).
pub const BANDS: usize = 21;
/// nm between samples (`STEP_LAMBDA`): 380 nm to 780 nm over 20 steps.
pub const STEP_LAMBDA: f32 = 20.0;

/// Sun irradiance on top of the atmosphere, W·m⁻²·nm⁻¹ (`IRRADIANCE`).
pub const IRRADIANCE: [f32; BANDS] = [
    1.457_568_3, 1.565_963_1, 1.651_484_5, 1.714_962_4, 1.757_979_9, 1.782_564_1, 1.790_951_1,
    1.785_415_5, 1.768_155_5, 1.741_220_7, 1.706_471_3, 1.665_560_9, 1.619_934_4, 1.570_836,
    1.519_323_4, 1.466_285, 1.412_458_5, 1.358_449_6, 1.304_749_1, 1.251_749_6, 1.199_760,
];

/// Rayleigh scattering coefficient, m⁻¹ (`RAYLEIGH_COEFF`).
pub const RAYLEIGH_COEFF: [f32; BANDS] = [
    5.424_820_1e-5, 4.418_549_9e-5, 3.635_151_9e-5, 3.017_929e-5, 2.526_320_2e-5, 2.130_859_3e-5,
    1.809_838e-5, 1.547_057_1e-5, 1.330_285e-5, 1.150_184_8e-5, 9.995_574_3e-6, 8.727_999_7e-6,
    7.655_137e-6, 6.742_172e-6, 5.961_341_3e-6, 5.290_346e-6, 4.711_156_9e-6, 4.209_104_8e-6,
    3.772_183_8e-6, 3.390_512_6e-6, 3.055_915_3e-6,
];

/// Ozone absorption coefficient, m⁻¹ (`OZONE_COEFF`).
pub const OZONE_COEFF: [f32; BANDS] = [
    3.251_268_5e-9, 5.853_953_7e-9, 1.977_191_2e-8, 7.309_568_8e-8, 2.008_456_2e-7,
    4.038_395_8e-7, 6.355_133_6e-7, 9.670_704_1e-7, 1.547_974e-6, 2.090_386_5e-6, 2.461_280_6e-6,
    2.735_513e-6, 2.151_258_6e-6, 1.590_518_4e-6, 1.123_562e-6, 7.352_755_1e-7, 4.645_013e-7,
    3.309_608e-7, 2.251_261_2e-7, 1.487_912_9e-7, 1.682_862_3e-7,
];

/// CIE XYZ colour matching functions at each wavelength (`CMF_XYZ`).
pub const CMF_XYZ: [[f32; 3]; BANDS] = [
    [0.001_368, 0.000_039, 0.006_450_001],
    [0.014_31, 0.000_396, 0.067_850_01],
    [0.134_38, 0.004, 0.645_6],
    [0.348_28, 0.023, 1.747_06],
    [0.290_8, 0.06, 1.669_2],
    [0.095_64, 0.139_02, 0.812_950_1],
    [0.004_9, 0.323, 0.272],
    [0.063_27, 0.71, 0.078_249_99],
    [0.290_4, 0.954, 0.020_3],
    [0.594_5, 0.995, 0.003_9],
    [0.916_3, 0.87, 0.001_650_001],
    [1.062_2, 0.631, 0.000_8],
    [0.854_449_9, 0.381, 0.000_19],
    [0.447_9, 0.175, 0.000_02],
    [0.164_9, 0.061, 0.0],
    [0.046_77, 0.017, 0.0],
    [0.011_359_16, 0.004_102, 0.0],
    [0.002_899_327, 0.001_047, 0.0],
    [0.000_690_078_6, 0.000_249_2, 0.0],
    [0.000_166_150_5, 0.000_06, 0.0],
    [0.000_041_509_94, 0.000_014_99, 0.0],
];

/// Gauss–Laguerre quadrature over the optical depth along a ray: the
/// standard n = 8 nodes with `exp(x)` folded into the weights, scaled so
/// the last node is still inside the atmosphere (`QUADRATURE_NODES`,
/// `QUADRATURE_WEIGHTS`). Blender's own comment gives the sympy that
/// produced them.
pub const QUADRATURE: [(f32, f32); 8] = [
    (0.006_811_185, 0.017_508_937),
    (0.036_148_07, 0.041_354_775),
    (0.090_043_47, 0.066_788_39),
    (0.170_668_01, 0.095_076_99),
    (0.281_836_2, 0.128_341_64),
    (0.430_340_64, 0.170_743_02),
    (0.629_627_15, 0.232_723_33),
    (0.914_525_3, 0.356_249_05),
];

/// A spectrum as a CIE XYZ colour (`spec_to_xyz`).
pub fn spec_to_xyz(spectrum: &[f32; BANDS]) -> [f32; 3] {
    let mut xyz = [0.0f32; 3];
    for (band, cmf) in spectrum.iter().zip(CMF_XYZ.iter()) {
        for k in 0..3 {
            xyz[k] += cmf[k] * band;
        }
    }
    xyz.map(|v| v * STEP_LAMBDA)
}

/// CIE XYZ to linear Rec.709 / sRGB, which is what `xyz_to_rgb_clamped`
/// comes to for the default working space (Blender derives the matrix from
/// the scene's, and phiray works in linear sRGB throughout). Negatives are
/// clamped, as Cycles clamps them.
pub fn xyz_to_rgb(xyz: [f32; 3]) -> [f32; 3] {
    const M: [[f32; 3]; 3] = [
        [3.240_454_2, -1.537_138_5, -0.498_531_4],
        [-0.969_266, 1.876_010_8, 0.041_556],
        [0.055_643_4, -0.204_025_9, 1.057_225_2],
    ];
    M.map(|row| (row[0] * xyz[0] + row[1] * xyz[1] + row[2] * xyz[2]).max(0.0))
}
