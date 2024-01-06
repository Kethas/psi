use super::*;

#[test]
fn integer() {
    init();

    let inputs = [
        ("0", 0),
        ("12850", 12850),
        ("-12850", -12850),
        ("+11100000", 11100000),
        ("-9", -9),
        ("-0", -0),
    ];

    for (input, expected_result) in inputs {
        log::debug!("input = \"{input}\"");

        assert_eq!(
            Some(&expected_result),
            rules::Integer
                .parse_entire("start", input)
                .expect("Should be parsed")
                .downcast_ref::<isize>()
        )
    }
}
