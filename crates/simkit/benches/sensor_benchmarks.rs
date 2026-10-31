//! Sensor update benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use simkit::prelude::*;

fn benchmark_imu_update(c: &mut Criterion) {
    let mut imu = ImuSensor::new("bench_imu");
    let kinematics = KinematicsState::default();
    let environment = EnvironmentState::default();

    imu.initialize(&kinematics, &environment);

    c.bench_function("imu_update", |b| {
        b.iter(|| {
            imu.update(black_box(0.016));
        });
    });
}

fn benchmark_sensor_collection_update(c: &mut Criterion) {
    let mut collection = SensorCollection::new();

    // Add multiple sensors
    for i in 0..10 {
        collection.add(Box::new(ImuSensor::new(format!("imu_{}", i))));
        collection.add(Box::new(GpsSensor::new(format!("gps_{}", i))));
        collection.add(Box::new(BarometerSensor::new(format!("baro_{}", i))));
    }

    c.bench_function("sensor_collection_30_sensors", |b| {
        b.iter(|| {
            collection.update(black_box(0.016));
        });
    });
}

fn benchmark_pose_operations(c: &mut Criterion) {
    let pose1 = Pose::new(
        glam::Vec3::new(1.0, 2.0, 3.0),
        glam::Quat::from_rotation_x(0.5),
    );

    let pose2 = Pose::new(
        glam::Vec3::new(4.0, 5.0, 6.0),
        glam::Quat::from_rotation_y(0.5),
    );

    c.bench_function("pose_clone", |b| {
        b.iter(|| {
            black_box(pose1);
            black_box(pose2);
        });
    });

    c.bench_function("pose_identity", |b| {
        b.iter(|| {
            black_box(Pose::identity());
        });
    });
}

fn benchmark_kinematics_access(c: &mut Criterion) {
    let kinematics = KinematicsState {
        pose: Pose::new(glam::Vec3::new(1.0, 2.0, 3.0), glam::Quat::IDENTITY),
        twist: Twist {
            linear: glam::Vec3::new(1.0, 0.0, 0.0),
            angular: glam::Vec3::new(0.0, 0.0, 0.5),
        },
        accelerations: Accelerations {
            linear: glam::Vec3::new(0.1, 0.0, 0.0),
            angular: glam::Vec3::ZERO,
        },
        timestamp: 0,
    };

    c.bench_function("kinematics_position_access", |b| {
        b.iter(|| {
            black_box(kinematics.position());
        });
    });

    c.bench_function("kinematics_velocity_access", |b| {
        b.iter(|| {
            black_box(kinematics.linear_velocity());
        });
    });
}

fn benchmark_recording_frame_creation(c: &mut Criterion) {
    c.bench_function("recorded_frame_creation", |b| {
        b.iter(|| {
            let mut frame = RecordedFrame::new(0, 0);
            frame.vehicle_poses.insert("drone".to_string(), Pose::identity());
            black_box(frame);
        });
    });
}

fn benchmark_clock_advance(c: &mut Criterion) {
    let mut clock = SimulationClock::new_steppable(0.016);

    c.bench_function("clock_advance_16ms", |b| {
        b.iter(|| {
            clock.step_by(black_box(0.016));
        });
    });
}

fn benchmark_image_request_creation(c: &mut Criterion) {
    c.bench_function("image_request_creation", |b| {
        b.iter(|| {
            let request = ImageRequest::new("camera1", ImageType::Scene);
            black_box(request);
        });
    });
}

criterion_group!(
    benches,
    benchmark_imu_update,
    benchmark_sensor_collection_update,
    benchmark_pose_operations,
    benchmark_kinematics_access,
    benchmark_recording_frame_creation,
    benchmark_clock_advance,
    benchmark_image_request_creation,
);
criterion_main!(benches);
