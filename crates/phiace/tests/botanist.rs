//! The botanist played through on phiecs, as a person plays Apple's
//! BOT-anist: make the robot, choose its look, go planting, walk to each
//! planter with the keys, celebrate, replay and exit. Where Apple's
//! scenes are not on this disk, the parts that read them say so and stop.

use phiace::bot::botanist::machine::{CELEBRATE, IDLE, STOP, WALK};
use phiace::bot::botanist::{Botanist, Data, Phase};
use phiecs::comp::tree::matrix;
use phiecs::usd::Kind;

#[test]
fn the_robot_begins_as_the_tables_say_and_takes_each_choice() {
    let mut bot = Botanist::new(Data::read());
    let data = bot.data().clone();
    let look = bot.look();
    let shape: Vec<&str> = look.part.iter().map(|(_, c)| c.shape.as_str()).collect();
    assert_eq!(shape, ["head2", "body2", "backpack1"]);
    assert_eq!((look.light_name(&data), look.face_name(&data), look.colour("body", &data)), ("white", "circle", [59, 152, 167]));
    // A metal body in its third colour, blue lights, heart eyes.
    bot.change(|look, _| {
        let body = look.choice("body").unwrap();
        (body.material, body.colour) = ("metal".to_string(), 2);
        look.choice("head").unwrap().shape = "head3".to_string();
        (look.light, look.face) = (3, 2);
    });
    let look = bot.look();
    assert_eq!((look.colour("body", &data), look.light_name(&data), look.face_name(&data)), ([150, 194, 70], "blue", "heart"));
    // Reset is the first robot again.
    bot.reset();
    assert_eq!(bot.look(), phiace::bot::botanist::look::Look::first(&data));
}

#[test]
fn a_part_is_its_usd_scene_in_the_world() {
    let bot = Botanist::new(Data::read());
    let head = bot.world.lexicon.get("robot/model/head").expect("the head");
    let prim = phiecs::comp::children(&bot.world, head);
    if prim.is_empty() {
        eprintln!("Apple's scenes are not on this disk: {}", bot.data().asset().display());
        return;
    }
    // The head's scene is head2.usda: its root holds head2, which takes
    // its mesh from head2.usdz, kept for whoever draws it.
    let mesh = bot.world.lexicon.get("robot/model/head/Root/head2").expect("the head's prim");
    let asset = bot.world.read::<Kind>(mesh).and_then(|k| k.get(phiecs::usd::stage::ASSET)).map(|v| v.text().to_string()).unwrap_or_default();
    assert!(asset.ends_with("head2.usdz"), "{asset}");
}

/// Steer with the keys toward a place on the ground, as a person does,
/// until the robot is there or the time runs out.
fn walk_to(bot: &mut Botanist, to: [f32; 2], seconds: f32) {
    let dt = 1.0 / 60.0;
    let mut held: Vec<&str> = Vec::new();
    for _ in 0..(seconds / dt) as usize {
        let at = matrix(&bot.world, bot.robot).origin();
        let (dx, dz) = (to[0] - at[0], to[1] - at[2]);
        let want: Vec<&str> = [(dx > 0.004, "d"), (dx < -0.004, "a"), (dz > 0.004, "s"), (dz < -0.004, "w")].into_iter().filter(|(on, _)| *on).map(|(_, key)| key).collect();
        held.iter().filter(|key| !want.contains(key)).for_each(|key| bot.lift(key));
        want.iter().filter(|key| !held.contains(key)).for_each(|key| bot.press(key));
        held = want;
        bot.step(dt);
        let garden = bot.garden().cloned().unwrap_or_default();
        if held.is_empty() || garden.done() {
            break;
        }
    }
    held.iter().for_each(|key| bot.lift(key));
}

#[test]
fn the_robot_walks_the_greenhouse_propagates_every_plant_and_celebrates() {
    let mut bot = Botanist::new(Data::read());
    if !bot.start_planting() {
        eprintln!("Apple's greenhouse is not on this disk: {}", bot.data().asset().display());
        return;
    }
    assert_eq!(bot.phase, Phase::Exploration);
    let garden = bot.garden().cloned().expect("the game");
    let mut plant: Vec<&str> = garden.planter.iter().map(|p| p.plant.as_str()).collect();
    plant.sort();
    assert_eq!((plant, garden.total, garden.found()), (vec!["coffeeBerry", "poppy", "yucca"], 3, 0));
    // A key walks it; letting go ends the walk, then it stands.
    bot.press("d");
    bot.step(0.1);
    assert_eq!(bot.state(), WALK);
    bot.lift("d");
    (0..12).for_each(|_| bot.step(0.02));
    assert_eq!(bot.state(), STOP);
    (0..80).for_each(|_| bot.step(0.02));
    assert_eq!(bot.state(), IDLE);
    // To each planter in turn: each is propagated where the robot comes.
    for planter in garden.planter.clone() {
        let there = matrix(&bot.world, planter.word).origin();
        walk_to(&mut bot, [there[0], there[2]], 12.0);
        let grown = bot.garden().map(|g| g.planter.iter().any(|p| p.word == planter.word && p.grown));
        assert_eq!(grown, Some(true), "{} at {there:?}", planter.plant);
    }
    bot.step(0.02);
    assert_eq!((bot.garden().map(|g| g.found()), bot.state()), (Some(3), CELEBRATE.to_string()));
    // Held long one way, it stops at the floor's edge, as walls stop it.
    let reach = bot.garden().map(|g| g.reach).unwrap_or_default();
    bot.press("d");
    (0..600).for_each(|_| bot.step(1.0 / 60.0));
    bot.lift("d");
    let at = matrix(&bot.world, bot.robot).origin();
    assert!(at[0].hypot(at[2]) <= reach + 1e-4, "{at:?} past {reach}");
    // Replay: nothing propagated, the robot standing where it began.
    bot.replay();
    assert_eq!((bot.garden().map(|g| g.found()), bot.state()), (Some(0), IDLE.to_string()));
    // Exit: back to making it.
    bot.exit();
    assert_eq!(bot.phase, Phase::Playing);
}

#[test]
fn the_robot_is_its_meshes_in_3d_with_its_head_on_its_body() {
    let bot = Botanist::new(Data::read());
    let mut library = phiusd::draw::Library::default();
    let started = std::time::Instant::now();
    let piece = phiace::bot::botanist::figure::figure(&bot.world, &bot.look(), bot.data(), &mut library);
    let count: usize = piece.iter().map(|p| p.triangle.len()).sum();
    if count == 0 {
        eprintln!("Apple's meshes are not on this disk");
        return;
    }
    // About 125 thousand triangles, read in well under a second.
    println!("{count} triangles in {} ms", started.elapsed().as_millis());
    assert!(count > 50_000, "{count}");
    // The head stands over the body: its lowest point is higher than the body's middle.
    let low = |p: &phiace::bot::botanist::figure::Piece| p.triangle.iter().flatten().map(|v| v[1]).fold(f32::MAX, f32::min);
    let high = |p: &phiace::bot::botanist::figure::Piece| p.triangle.iter().flatten().map(|v| v[1]).fold(f32::MIN, f32::max);
    let (all_low, all_high) = (piece.iter().map(low).fold(f32::MAX, f32::min), piece.iter().map(high).fold(f32::MIN, f32::max));
    println!("robot from {all_low} to {all_high}");
    assert!(all_high > all_low);
    // The parts are of one rig: the head on the body with no gap, so
    // the robot is no taller than a body and a head (about 9.4 cm).
    assert!(all_high - all_low < 0.1, "{all_low} to {all_high}");
    // Seen small, far fewer triangles do: 96 cells across it.
    let few: usize = phiace::bot::botanist::figure::coarse(&piece, 96.0).iter().map(|p| p.triangle.len()).sum();
    println!("{few} triangles in 96 cells");
    assert!(few * 3 < count, "{few} of {count}");
}
