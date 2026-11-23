# Math — Vectors and Matrices

## Overview

`math` module provides 2D/3D vector and matrix types for graphics and animation. Essential for transforms, positioning, and calculations. Mirrors SceneKit/RealityKit math types.

## Types

| Type | Description | Dimensions |
|------|-------------|------------|
| `SCNVector3` | 3D vector | (x, y, z) |
| `SCNVector4` | 4D vector | (x, y, z, w) |
| `SCNMatrix4` | 4x4 matrix | Transform matrix |
| `SCNQuaternion` | Quaternion | Rotation |

## Vectors

### SCNVector3
```rust
use rust_appkit::math::SCNVector3;

// Create
let v = SCNVector3::new(1.0, 2.0, 3.0);

// Zero and one
let zero = SCNVector3::zero();
let one = SCNVector3::one();

// Unit axes
let x = SCNVector3::x_axis();
let y = SCNVector3::y_axis();
let z = SCNVector3::z_axis();

// Operations
let a = SCNVector3::new(1.0, 0.0, 0.0);
let b = SCNVector3::new(0.0, 1.0, 0.0);

let sum = a + b;           // (1, 1, 0)
let diff = a - b;          // (1, -1, 0)
let scaled = a * 2.0;      // (2, 0, 0)

let dot = a.dot(b);        // 0.0 (perpendicular)
let cross = a.cross(b);    // (0, 0, 1)
let len = a.length();      // 1.0
let norm = a.normalized(); // (1, 0, 0)
```

### SCNVector4
```rust
use rust_appkit::math::SCNVector4;

let v4 = SCNVector4::new(1.0, 2.0, 3.0, 1.0);
// Often used as homogeneous coordinates
```

## Matrices

### SCNMatrix4
```rust
use rust_appkit::math::SCNMatrix4;

// Identity
let identity = SCNMatrix4::identity();

// Translation
let translate = SCNMatrix4::translation(10.0, 20.0, 30.0);

// Scale
let scale = SCNMatrix4::scale(2.0, 2.0, 2.0);

// Rotation (axis + angle in radians)
let rotate_x = SCNMatrix4::rotation(SCNVector3::x_axis(), std::f32::consts::PI / 2.0);

// Composition (multiply in reverse order)
let transform = translate * rotate_x * scale;

// Transform a point
let point = SCNVector3::new(1.0, 0.0, 0.0);
let transformed = transform.transform_vector3(point);
```

### Matrix Decomposition
```rust
let matrix = SCNMatrix4::identity();

let (position, rotation, scale) = matrix.decompose();

println!("Position: {:?}", position);
println!("Rotation: {:?}", rotation);
println!("Scale: {:?}", scale);
```

## Quaternions

### SCNQuaternion
```rust
use rust_appkit::math::SCNQuaternion;

// From axis-angle
let q = SCNQuaternion::from_axis_angle(SCNVector3::y_axis(), std::f32::consts::PI);
// 180° rotation around Y

// Identity (no rotation)
let identity = SCNQuaternion::identity();

// Composition
let combined = q * identity; // Still just q

// To/from matrix
let matrix = q.to_matrix4();
let back_to_q = SCNQuaternion::from_matrix4(&matrix);

// Spherical interpolation (slerp)
let start = SCNQuaternion::identity();
let end = SCNQuaternion::from_axis_angle(SCNVector3::y_axis(), PI);
let halfway = SCNQuaternion::slerp(start, end, 0.5);
```

## Connection to Other Modules

```
Math Types
├── Used by: Transform component (ECS)
├── Used by: Camera (position, rotation)
├── Used by: Timeline (animation values)
└── Used by: 3D viewport (projection matrices)
```

## ECS Integration

```rust
// In reality-foundation ECS
#[derive(Component)]
struct Transform {
    position: SCNVector3,
    rotation: SCNQuaternion,
    scale: SCNVector3,
}

impl Transform {
    fn to_matrix(&self) -> SCNMatrix4 {
        SCNMatrix4::translation(self.position.x, self.position.y, self.position.z)
            * self.rotation.to_matrix4()
            * SCNMatrix4::scale(self.scale.x, self.scale.y, self.scale.z)
    }
}
```

## File Structure

```
src/math/
├── mod.rs          # SCNVector3, SCNVector4
├── matrix.rs       # SCNMatrix4
└── quaternion.rs   # SCNQuaternion
```

## Testing & Verification

```rust
// Vector tests
let v = SCNVector3::new(3.0, 4.0, 0.0);
assert_eq!(v.length(), 5.0);

let normalized = v.normalized();
assert!((normalized.length() - 1.0).abs() < 0.0001);

// Cross product
let x = SCNVector3::x_axis();
let y = SCNVector3::y_axis();
let z = x.cross(y);
assert_eq!(z, SCNVector3::z_axis());

// Matrix tests
let translate = SCNMatrix4::translation(10.0, 20.0, 30.0);
let point = SCNVector3::zero();
let moved = translate.transform_vector3(point);
assert_eq!(moved, SCNVector3::new(10.0, 20.0, 30.0));
```

## Related

- **reality-foundation**: ECS uses these types for transforms
- **reality-kit**: 3D rendering with these math types
- **timeline**: Animates `SCNVector3` and `SCNQuaternion`
