//! Flying a drone and driving a car in one phiecs world, from a keyboard
//! and then from an AirSim client — the whole control layer, end to end.
//!
//! ```text
//! cargo run -p simkit --example fly_with_a_stick
//! ```
//!
//! There is no window here, so the keys are pressed in code: a host with a
//! window calls the same [`Keys::press`] and [`Keys::release`] from its own
//! key events and nothing else changes. Swap [`Keys`] for a gamepad
//! backend — anything implementing [`Device`] — and nothing else changes
//! either, which is the point of the device being a seam.

use simkit::ecs::{build, link_of, Act, Driver, Plan};
use simkit::input::{CarDemand, ControlState, Device, Gear, Key, Keys, Pilot};

/// Fifty steps a second.
const DT: f32 = 0.02;

fn main() {
    println!("simkit control layer v{}\n", simkit::VERSION);
    fly_a_drone();
    println!();
    drive_a_car();
    println!();
    fly_it_from_a_client();
}

/// A drone flown from the keyboard: space arms, `M` steps the mode switch,
/// `W` and `S` work the collective, the arrows are the right stick.
fn fly_a_drone() {
    println!("── a drone, from a keyboard ──");
    let mut world = build(Plan::new().with_drone("drone"));
    let link = link_of(&world);
    let mut driver = Driver::new(&world, "drone", Keys::flight(), Pilot::gamepad());

    // Space arms it, and one press of `M` steps the mode switch from
    // Angle to AltitudeHold, where the collective is a climb rate about
    // its centre.
    driver.device.press(Key::Space);
    driver.device.press(Key::Char('m'));
    run(&mut world, &mut driver, 1);
    driver.device.release(Key::Char('m'));
    println!("  mode: {:?}, armed", driver.pilot.mode());

    // `W` raises the collective. It is a lever, so it stays where it is
    // left and the drone keeps climbing after the key is let go.
    driver.device.press(Key::Char('w'));
    run(&mut world, &mut driver, 60);
    driver.device.release(Key::Char('w'));
    run(&mut world, &mut driver, 90);
    say(&link, "drone", "climbing on a raised collective");

    // Centring the collective holds the height it reached.
    driver.device.press(Key::Char('s'));
    run(&mut world, &mut driver, 60);
    driver.device.release(Key::Char('s'));
    run(&mut world, &mut driver, 150);
    say(&link, "drone", "holding on a centred collective");

    // The up arrow is pitch, which in AltitudeHold is a lean, so it
    // flies forward.
    driver.device.press(Key::Up);
    run(&mut world, &mut driver, 150);
    driver.device.release(Key::Up);
    run(&mut world, &mut driver, 50);
    say(&link, "drone", "flying forward on the pitch stick");

    // `L` lands it: the controller flies the descent and disarms on the
    // ground, so the stick is not holding it down.
    driver.device.press(Key::Char('l'));
    run(&mut world, &mut driver, 1);
    driver.device.release(Key::Char('l'));
    driver.device.release(Key::Space);
    run(&mut world, &mut driver, 400);
    say(&link, "drone", "landed");
}

/// A car driven from the keyboard: the arrows steer and work the pedals,
/// `E` and `Q` shift, space is the handbrake.
fn drive_a_car() {
    println!("── a car, from a keyboard ──");
    let mut world = build(Plan::new().with_car("taxi"));
    let link = link_of(&world);
    let mut driver = Driver::new(&world, "taxi", Keys::driving(), Pilot::wheel());

    // The wheel preset reads steering off its own axis, which is where
    // the driving key layout puts the arrows.
    driver.device.press(Key::Up);
    run(&mut world, &mut driver, 200);
    say(&link, "taxi", "accelerating in first");

    driver.device.press(Key::Char('e'));
    run(&mut world, &mut driver, 1);
    driver.device.release(Key::Char('e'));
    run(&mut world, &mut driver, 200);
    println!("  gear: {:?}", driver.pilot.gear());
    say(&link, "taxi", "in second");

    driver.device.press(Key::Right);
    run(&mut world, &mut driver, 200);
    driver.device.release(Key::Right);
    say(&link, "taxi", "through a right-hander");

    driver.device.release(Key::Up);
    driver.device.press(Key::Space);
    run(&mut world, &mut driver, 300);
    say(&link, "taxi", "stopped on the handbrake");
}

/// The same world, flown by an AirSim client over the same link.
///
/// The orders below are what the msgpack-rpc server builds from
/// `enableApiControl`, `armDisarm` and `moveToPosition` — there is no
/// second path to the vehicle, so a client and a stick take turns rather
/// than fighting.
fn fly_it_from_a_client() {
    println!("── the same drone, from a client ──");
    let mut world = build(Plan::new().with_drone("drone"));
    let link = link_of(&world);

    link.api("drone", Act::ApiControl(true));
    link.api("drone", Act::Arm(true));
    link.api("drone", Act::Takeoff { altitude: 6.0 });
    step(&mut world, 700);
    say(&link, "drone", "after a commanded takeoff");

    link.api("drone", Act::MoveToPosition { to: glam::Vec3::new(15.0, -8.0, 6.0), speed: 6.0 });
    step(&mut world, 1200);
    say(&link, "drone", "after a commanded move");

    // A stick cannot interrupt while the client holds it. The order is
    // counted as refused rather than dropped quietly, so a pilot
    // wondering why nothing moved can be told.
    link.device("drone", Act::Helm(ControlState::Rotor(Default::default())));
    step(&mut world, 1);
    let refused = link.report("drone").expect("a report").refused;
    println!("  a stick frame while the client holds it: refused ({refused} so far)");

    link.api("drone", Act::Land);
    step(&mut world, 800);
    say(&link, "drone", "landed");

    // And the car's own controls come off the wire the same way.
    let mut city = build(Plan::new().with_car("taxi"));
    let roads = link_of(&city);
    roads.api("taxi", Act::ApiControl(true));
    roads.api(
        "taxi",
        Act::Helm(ControlState::Car(CarDemand {
            throttle: 0.8,
            steer: 0.2,
            gear: Gear::Forward(1),
            ..CarDemand::idle()
        })),
    );
    step(&mut city, 400);
    say(&roads, "taxi", "driven from a client");
}

/// Polls the device and steps the world `steps` times, which is a host's
/// whole loop.
fn run<D: Device>(world: &mut phiecs::prelude::World, driver: &mut Driver<D>, steps: usize) {
    for _ in 0..steps {
        driver.poll(DT);
        world.step(DT);
    }
}

/// Steps a world nobody is holding a stick over.
fn step(world: &mut phiecs::prelude::World, steps: usize) {
    for _ in 0..steps {
        world.step(DT);
    }
}

/// Where a vehicle got to.
fn say(link: &simkit::ecs::ControlLink, vehicle: &str, what: &str) {
    let Some(report) = link.report(vehicle) else {
        println!("  {what}: nothing reported");
        return;
    };
    let at = report.pose().position;
    println!(
        "  {what}: at ({:.1}, {:.1}, {:.1}) m, {:+.2} m/s climb, {:.1} m/s over the ground, {:?}{}",
        at.x,
        at.y,
        at.z,
        report.climb(),
        report.ground_speed(),
        report.mode,
        match report.armed {
            true => ", armed",
            false => ", disarmed",
        }
    );
}
