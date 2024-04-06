use std::path::PathBuf;

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
#[ignore]
fn json_1mb() {
    init();

    let parsed = rules::JsonRules
        .parse_entire("start", JSON_1MB)
        .map(|result| result.downcast::<rules::json::Json>());

    assert!(parsed.is_ok() && parsed.unwrap().is_ok());
}

// This test assumes that the working directory is the crate root
#[cfg(feature = "file_input")]
#[test]
#[ignore]
fn read_json_1mb() {
    init();

    let path = PathBuf::from("./src/tests/large_files/1MB.json");

    let parsed = rules::JsonRules
        .parse_entire("start", &path)
        .map(|result| result.downcast::<rules::json::Json>());

    println!("Parsed: {parsed:#?}");

    assert!(parsed.is_ok() && parsed.unwrap().is_ok());
}

#[cfg(feature = "tcp_input")]
#[test]
#[ignore]
fn local_tcp_stream_json_1mb() {
    use std::net::{TcpListener, TcpStream};
    use std::time::Duration;

    init();

    // spawn server that listens to 1 request
    std::thread::spawn(|| {
        let listener = TcpListener::bind("127.0.0.1:7878").expect("Could not start server");

        let (mut stream, _) = listener.accept().expect("Could not accept connection");

        let half_len = JSON_1MB.len() / 2;

        let (first_half, second_half) = JSON_1MB.as_bytes().split_at(half_len);

        // send first half of file
        stream
            .write_all(first_half)
            .expect("Could not send first half of file");

        // wait 5 secodnds
        std::thread::sleep(Duration::from_secs(5));

        // send second half of file
        stream
            .write_all(second_half)
            .expect("Could not send second half of file");
    });

    // wait to ensure that the listener is running
    std::thread::sleep(Duration::from_millis(200));

    let stream = TcpStream::connect("127.0.0.1:7878").expect("Could not connect to server");

    let parsed = rules::JsonRules
        .parse_entire("start", stream)
        .map(|result| result.downcast::<rules::json::Json>());

    println!("Parsed: {parsed:#?}");

    assert!(parsed.is_ok() && parsed.unwrap().is_ok());
}
