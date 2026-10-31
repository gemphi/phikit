//! What the wire promises: a getter is not a command, every vector
//! turns out of NED exactly once, and a `moveByRC` arrives as the
//! same helm command a gamepad sends.

use super::*;
use crate::input::RotorMode;

/// Parameters as a list, which is what every AirSim call is.
struct List(Vec<Value>);

#[derive(Clone)]
enum Value {
    Num(f64),
    Flag(bool),
    Text(String),
}

impl Params for List {
    fn number(&self, index: usize) -> Option<f64> {
        match self.0.get(index) {
            Some(Value::Num(n)) => Some(*n),
            Some(Value::Flag(b)) => Some(f64::from(u8::from(*b))),
            _ => None,
        }
    }

    fn flag(&self, index: usize) -> Option<bool> {
        match self.0.get(index) {
            Some(Value::Flag(b)) => Some(*b),
            Some(Value::Num(n)) => Some(*n != 0.0),
            _ => None,
        }
    }

    fn text(&self, index: usize) -> Option<String> {
        match self.0.get(index) {
            Some(Value::Text(s)) => Some(s.clone()),
            _ => None,
        }
    }

    fn count(&self) -> usize {
        self.0.len()
    }
}

fn list(values: Vec<Value>) -> List {
    List(values)
}

#[test]
fn a_getter_is_not_a_command() {
    assert!(read("ping", &list(vec![])).is_none());
    assert!(read("simGetImages", &list(vec![])).is_none());
    assert!(read("getImuData", &list(vec![])).is_none());
}

#[test]
fn taking_control_and_arming_carry_their_flag_and_their_vehicle() {
    let call = read("enableApiControl", &list(vec![Value::Flag(true), Value::Text("drone".into())]))
        .expect("a command");
    assert_eq!(call.vehicle, "drone");
    assert_eq!(call.act, Act::ApiControl(true));

    let disarm =
        read("armDisarm", &list(vec![Value::Flag(false)])).expect("a command");
    assert_eq!(disarm.act, Act::Arm(false));
    assert_eq!(disarm.vehicle, ANY_VEHICLE, "a client that says nothing means the only one");
}

#[test]
fn a_commanded_move_turns_out_of_ned_into_the_world() {
    let call = read(
        "moveToPosition",
        &list(vec![
            Value::Num(5.0),
            Value::Num(2.0),
            // AirSim's z is down, so -10 is ten metres up.
            Value::Num(-10.0),
            Value::Num(3.0),
            Value::Text("drone".into()),
        ]),
    )
    .expect("a command");
    assert_eq!(call.vehicle, "drone");
    match call.act {
        Act::MoveToPosition { to, speed } => {
            assert_eq!(to, Vec3::new(5.0, 2.0, 10.0), "ten metres up in a z-up world");
            assert_eq!(speed, 3.0);
        }
        other => panic!("wanted a move, got {other:?}"),
    }
}

#[test]
fn a_commanded_velocity_turns_too() {
    let call = read(
        "moveByVelocity",
        &list(vec![
            Value::Num(1.0),
            Value::Num(0.0),
            Value::Num(-2.0),
            Value::Num(5.0),
            Value::Text("drone".into()),
        ]),
    )
    .expect("a command");
    match call.act {
        Act::MoveByVelocity { velocity, .. } => {
            assert_eq!(velocity, Vec3::new(1.0, 0.0, 2.0), "descending down the wire is climbing")
        }
        other => panic!("wanted a velocity, got {other:?}"),
    }
}

#[test]
fn a_move_by_rc_arrives_as_the_same_helm_a_gamepad_sends() {
    // Sticks centred, throttle open, arm switch on, mode switch on
    // altitude hold: switches = 1 | (2 << 1) = 5.
    let call = read(
        "moveByRC",
        &list(vec![
            Value::Num(0.25),
            Value::Num(-0.5),
            Value::Num(1.0),
            Value::Num(0.0),
            Value::Num(5.0),
            Value::Text("drone".into()),
        ]),
    )
    .expect("a command");
    let Act::Helm(ControlState::Rotor(demand)) = call.act else {
        panic!("a moveByRC is a helm command, like a stick's")
    };
    assert_eq!(demand.roll, 0.25);
    assert_eq!(demand.pitch, -0.5);
    assert_eq!(demand.throttle, 1.0, "a centred-on-the-wire throttle at its top is open");
    assert!(demand.armed);
    assert_eq!(demand.mode, RotorMode::AltitudeHold);
}

#[test]
fn car_controls_arrive_as_a_car_helm() {
    let call = read(
        "setCarControls",
        &list(vec![
            Value::Num(0.7),
            Value::Num(-0.3),
            Value::Num(0.0),
            Value::Flag(false),
            Value::Flag(true),
            Value::Num(-1.0),
            Value::Text("taxi".into()),
        ]),
    )
    .expect("a command");
    assert_eq!(call.vehicle, "taxi");
    let Act::Helm(ControlState::Car(demand)) = call.act else { panic!("a car helm") };
    assert_eq!(demand.throttle, 0.7);
    assert_eq!(demand.steer, -0.3);
    assert_eq!(demand.gear, crate::input::Gear::Reverse);
}

#[test]
fn a_pose_goes_back_out_in_ned() {
    let mut report = Report::default();
    report.kinematics.pose.position = Vec3::new(1.0, 2.0, 30.0);
    let reply = pose_reply(&report);
    assert_eq!(reply[2], -30.0, "thirty metres up is thirty metres of negative down");
    assert_eq!(reply[3], 1.0, "and the orientation leads with w, as AirSim's does");
}

#[test]
fn kinematics_go_back_out_in_ned_too() {
    let mut report = Report::default();
    report.kinematics.pose.position = Vec3::new(0.0, 0.0, 5.0);
    report.kinematics.twist.linear = Vec3::new(0.0, 0.0, 2.0);
    let [at, vel, _] = kinematics_reply(&report);
    assert_eq!(at[2], -5.0);
    assert_eq!(vel[2], -2.0, "a climb is a negative descent");
}
