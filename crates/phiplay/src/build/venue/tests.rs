use super::*;
use crate::sport::BSport;
use crate::venue::BVenue;

#[test]
fn every_venue_seats_a_crowd_round_every_sport_and_lights_its_show() {
    for venue_kind in BVenue::ALL {
        for sport in BSport::ALL {
            let choice = BChoice { venue: venue_kind, sport, ..BChoice::default() };
            let built = venue(&choice);
            let label = format!("{} round {}", venue_kind.spec().label, sport.spec().label);
            assert!(!built.parts.is_empty(), "{label}");
            // Seats on every tier, every one outside the floor, facing in.
            let [fx, fy] = sport.spec().field.floor_half();
            // Room for the full house, and every fan in a seat.
            assert!(built.places.len() >= crate::choice::CROWD as usize, "{label}: {} seats", built.places.len());
            assert_eq!(built.crowd.iter().map(|f| f.at.len()).sum::<usize>(), crate::choice::CROWD as usize, "{label}");
            for p in &built.places {
                assert!(p.at.x.abs() > fx || p.at.y.abs() > fy, "{label}: a seat on the floor at {:?}", p.at);
                let facing = glam::Vec2::new(p.heading.sin(), -p.heading.cos());
                assert!(facing.dot(-p.at.truncate()) > 0.0, "{label}: a seat facing out at {:?}", p.at);
            }
            let seated: usize = built.seats.iter().map(|s| s.at.len()).sum();
            assert_eq!(seated, built.places.len(), "{label}: a seat for every place");
            assert_eq!(built.groups.iter().filter(|g| g.material.starts_with("Facade")).count(), usize::from(venue_kind.spec().panels));
            assert!(!built.lamps.is_empty(), "{label}: lit");
        }
    }
}
