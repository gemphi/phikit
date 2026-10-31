//! The two doors into a world, in one test: a stick in the room and
//! an AirSim client over msgpack-rpc.
//!
//! What is asserted is that neither can bypass the other. Both put an
//! order on the same `ControlLink`, the same `Intake` drains it, and
//! which one is obeyed is the one flag AirSim uses for it —
//! `enableApiControl`. The loser's orders are counted as refused
//! rather than dropped quietly.

use simkit::ecs::{build, link_of, Act, ControlLink, Driver, Plan};
use simkit::input::{slot, Pilot, Reading, RotorMode, Scripted};

/// Fifty steps a second.
const DT: f32 = 0.02;

/// A frame with the arm switch on, on the gamepad layout.
fn armed() -> Reading {
    Reading::resting(slot::AXES).with_held(slot::BUTTON_ARM)
}

#[test]
fn a_client_over_the_wire_and_a_stick_in_the_room_cannot_both_fly_it() {
    // The local path and the remote path, in one world, on one link.
    let mut world = build(Plan::new().with_drone("drone"));
    let link = link_of(&world);
    let script: Vec<Reading> =
        (0..2000).map(|_| armed().with_axis(slot::AXIS_THROTTLE, 1.0)).collect();
    let mut driver = Driver::new(&world, "drone", Scripted::new("stick", script), Pilot::gamepad());
    driver.pilot.set_mode(RotorMode::AltitudeHold);

    // The stick flies it up.
    for _ in 0..200 {
        driver.poll(DT);
        world.step(DT);
    }
    let by_stick = link.report("drone").expect("a report").altitude();
    assert!(by_stick > 2.0, "the stick flew it: {by_stick} m");

    // A client takes the vehicle, the way an AirSim client does, and now
    // the stick is refused — it is still being polled every step.
    wire("enableApiControl", &[Param::Flag(true), Param::Text("drone")], &link);
    for _ in 0..100 {
        driver.poll(DT);
        world.step(DT);
    }
    let after_taking = link.report("drone").expect("a report");
    assert!(after_taking.refused > 50, "the stick's frames were refused, and counted");

    // And the client's own commands land, by the same route.
    wire("armDisarm", &[Param::Flag(true), Param::Text("drone")], &link);
    wire(
        "moveToPosition",
        &[
            Param::Num(0.0),
            Param::Num(0.0),
            // AirSim's z is down, so this is twelve metres up.
            Param::Num(-12.0),
            Param::Num(4.0),
            Param::Text("drone"),
        ],
        &link,
    );
    for _ in 0..1200 {
        driver.poll(DT);
        world.step(DT);
    }
    let by_client = link.report("drone").expect("a report");
    assert!(
        (by_client.altitude() - 12.0).abs() < 1.5,
        "the client flew it to the height it asked for: {} m",
        by_client.altitude()
    );
    assert!(by_client.api_control, "and the client still holds it");

    // Handing it back puts the stick in charge again, with no second
    // code path involved either way.
    let refused_before = by_client.refused;
    wire("enableApiControl", &[Param::Flag(false), Param::Text("drone")], &link);
    for _ in 0..50 {
        driver.poll(DT);
        world.step(DT);
    }
    assert_eq!(
        link.report("drone").expect("a report").refused,
        refused_before,
        "nothing is being refused once the stick has it back"
    );
}

#[test]
fn an_rpc_server_with_a_world_attached_answers_from_it() {
    let mut world = build(Plan::new().with_drone("drone").with_car("taxi"));
    let link = link_of(&world);

    let mut server = simkit::server::Simulation::new();
    server.attach(link.clone());
    assert!(server.report("drone").is_none(), "nothing has stepped yet");

    link.api("drone", Act::ApiControl(true));
    link.api("drone", Act::Takeoff { altitude: 4.0 });
    for _ in 0..1000 {
        world.step(DT);
    }

    let report = server.report("drone").expect("the server reads the world's report");
    assert!(report.altitude() > 3.0, "the takeoff the client asked for: {} m", report.altitude());
    assert!(report.api_control);

    // And the answers a client gets are AirSim's: NED, so up is negative.
    let pose = simkit::ecs::wire::pose_reply(&report);
    assert!(pose[2] < -3.0, "a client reads the climb as negative down: {}", pose[2]);

    // With two vehicles there is no "the only one", so a call that names
    // none has to be refused rather than guessed at.
    assert_eq!(server.only_vehicle(), "", "two vehicles is a choice, not a default");
    assert!(server.report("").is_none());
}

/// A parameter of an AirSim call, for the wire tests.
enum Param {
    Num(f64),
    Flag(bool),
    Text(&'static str),
}

/// Parameters as AirSim sends them: a list.
struct Call<'a>(&'a [Param]);

impl simkit::ecs::wire::Params for Call<'_> {
    fn number(&self, index: usize) -> Option<f64> {
        match self.0.get(index) {
            Some(Param::Num(n)) => Some(*n),
            Some(Param::Flag(b)) => Some(f64::from(u8::from(*b))),
            _ => None,
        }
    }

    fn flag(&self, index: usize) -> Option<bool> {
        match self.0.get(index) {
            Some(Param::Flag(b)) => Some(*b),
            Some(Param::Num(n)) => Some(*n != 0.0),
            _ => None,
        }
    }

    fn text(&self, index: usize) -> Option<String> {
        match self.0.get(index) {
            Some(Param::Text(s)) => Some((*s).to_string()),
            _ => None,
        }
    }

    fn count(&self) -> usize {
        self.0.len()
    }
}

/// An AirSim call put on the link, exactly as the RPC server's own
/// dispatch does it.
fn wire(method: &str, params: &[Param], link: &ControlLink) {
    let call = simkit::ecs::wire::read(method, &Call(params))
        .unwrap_or_else(|| panic!("{method} should be a command"));
    assert!(simkit::ecs::wire::send(link, call, "drone"), "{method} should reach a vehicle");
}
