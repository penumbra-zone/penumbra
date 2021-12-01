use decaf377_fmd as fmd;
use rand_core::OsRng;

#[test]
fn detection_distribution_matches_expectation() {
    let alice_dk = fmd::DetectionKey::new(OsRng);
    let alice_clue_key = alice_dk.clue_key().expand().unwrap();
    // alice's friend bobce, whose name has the same number of letters
    let bobce_dk = fmd::DetectionKey::new(OsRng);

    const NUM_CLUES: usize = 1024;
    const PRECISION_BITS: usize = 4; // p = 1/16

    let clues = (0..NUM_CLUES)
        .map(|_| alice_clue_key.create_clue(PRECISION_BITS, OsRng).unwrap())
        .collect::<Vec<_>>();

    let alice_detections = clues
        .iter()
        .filter(|clue| alice_dk.examine(clue).is_ok())
        .count();
    let bobce_detections = clues
        .iter()
        .filter(|clue| bobce_dk.examine(clue).is_ok())
        .count();

    let bobce_detection_rate = (bobce_detections as f64) / (NUM_CLUES as f64);
    let expected_rate = 0.5f64.powi(PRECISION_BITS as i32);

    dbg!(alice_detections);
    dbg!(bobce_detections);
    dbg!(bobce_detection_rate);
    dbg!(expected_rate);

    assert_eq!(alice_detections, NUM_CLUES);
    assert!((expected_rate - bobce_detection_rate).abs() < 0.04);
}
