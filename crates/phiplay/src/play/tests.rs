use super::*;
use crate::sport::BSport;

#[test]
fn every_sport_plays_a_minute_with_scores_and_everyone_on_the_field() {
    for sport in BSport::ALL {
        let choice = BChoice { sport, ..BChoice::default() };
        let tape = play(&choice, 60.0);
        let spec = sport.spec();
        assert_eq!(tape.frames(), 1440, "{}", spec.label);
        assert_eq!(tape.players.len(), 2 * usize::from(spec.side));
        let [fx, fy] = spec.field.floor_half();
        for track in &tape.players {
            assert_eq!(track.steps.len(), 1440);
            for s in &track.steps {
                assert!(s.at.x.abs() <= fx + 1.5 && s.at.y.abs() <= fy + 0.5, "{}: a player at {:?}", spec.label, s.at);
            }
        }
        let last = tape.score.last().unwrap();
        assert!(last[0] + last[1] > 0, "{}: a minute without a score", spec.label);
        let moves: std::collections::HashSet<_> = tape.players.iter().flat_map(|t| t.steps.iter().map(|s| s.mv)).collect();
        assert!(moves.len() >= 4, "{}: only {:?}", spec.label, moves);
        eprintln!("{}: {:?} after a minute, moves {:?}", spec.label, last, moves);
    }
}
