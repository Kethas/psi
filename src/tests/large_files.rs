use super::*;

const JSON_LARGE_FILE: &str = include_str!("large_files/large_file.json");
const JSON_HUGE_FILE: &str = include_str!("large_files/huge_file.json");
const JSON_1MB: &str = include_str!("large_files/1MB.json");

#[test]
fn json_large_file() {
    init();

    let parsed = rules::JsonRules
        .parse_entire("start", JSON_LARGE_FILE)
        .map(|result| result.downcast::<rules::json::Json>());

    println!("Parsed: {parsed:#?}");

    assert!(parsed.is_ok() && parsed.unwrap().is_ok());
}

#[test]
#[ignore] // it's just too big...
fn json_huge_file() {
    init();

    let parsed = rules::JsonRules
        .parse_entire("start", JSON_HUGE_FILE)
        .map(|result| result.downcast::<rules::json::Json>());

    println!("Parsed: {parsed:#?}");

    assert!(parsed.is_ok() && parsed.unwrap().is_ok());
}

#[test]
fn json_1mb() {
    init();

    let parsed = rules::JsonRules
        .parse_entire("start", JSON_1MB)
        .map(|result| result.downcast::<rules::json::Json>());

    println!("Parsed: {parsed:#?}");

    assert!(parsed.is_ok() && parsed.unwrap().is_ok());
}
