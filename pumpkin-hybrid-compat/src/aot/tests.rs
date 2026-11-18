use super::execute;
use std::path::PathBuf;

#[test]
#[ignore = "set PUMPKIN_AOT_TEST_JAR to the original Teleport Commands 1.3.5 JAR"]
fn original_jar_matches_java_reference() {
    let jar = PathBuf::from(std::env::var_os("PUMPKIN_AOT_TEST_JAR").unwrap());
    let cache = tempfile::tempdir().unwrap();
    for (input, expected) in [
        ([12.5, 49.0, 12.5], [12, 49, 12]),
        ([-0.01, -2.5, 44.999], [-1, -2, 44]),
        ([-0.0, 2.5, 0.0], [0, 3, 0]),
        ([1.9, 2_147_483_648.0, -1.01], [1, i32::MIN, -2]),
        ([f64::NAN; 3], [0; 3]),
        (
            [f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY],
            [i32::MAX, -1, i32::MIN],
        ),
        (
            [f64::NEG_INFINITY, f64::NEG_INFINITY, f64::INFINITY],
            [i32::MIN, 0, i32::MAX],
        ),
    ] {
        assert_eq!(
            execute(&jar, cache.path(), Some(input)).unwrap().value,
            Some(expected)
        );
    }
    assert_eq!(execute(&jar, cache.path(), None).unwrap().value, None);
}
