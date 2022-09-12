use decaf377_fmd as fmd;
use fmd::ClueKey;
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

    let alice_detections = clues.iter().filter(|clue| alice_dk.examine(clue)).count();
    let bobce_detections = clues.iter().filter(|clue| bobce_dk.examine(clue)).count();

    let bobce_detection_rate = (bobce_detections as f64) / (NUM_CLUES as f64);
    let expected_rate = 0.5f64.powi(PRECISION_BITS as i32);

    dbg!(alice_detections);
    dbg!(bobce_detections);
    dbg!(bobce_detection_rate);
    dbg!(expected_rate);

    assert_eq!(alice_detections, NUM_CLUES);
    assert!((expected_rate - bobce_detection_rate).abs() < 0.04);
}

#[test]
fn fails_to_expand_clue_key() {
    let clue_key = ClueKey([1; 32]);

    clue_key
        .expand()
        .err()
        .expect("fails to generate an expanded clue key with invalid encoding");
}

#[test]
fn fails_to_generate_clue() {
    let detection_key = fmd::DetectionKey::new(OsRng);
    let expanded_clue_key = detection_key.clue_key().expand().unwrap();

    expanded_clue_key
        .create_clue(fmd::MAX_PRECISION + 1, OsRng)
        .expect_err("fails to generate clue with excessive precision");
}
