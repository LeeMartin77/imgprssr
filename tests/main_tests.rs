/// Integration tests for server_config — generate_app_config()
use imgprssr::appconfig::ImgprssrConfigErr;
use temp_env::with_vars;

#[test]
fn invalid_address_returns_error() {
    with_vars(
        [("IMGPRSSR_ADDRESS", Some("not.an.ip"))],
        || {
            let result = imgprssr::server_config::generate_app_config();
            assert_eq!(
                result,
                Err(ImgprssrConfigErr::InvalidValues(vec!["address".to_owned()]))
            );
        },
    );
}

#[test]
fn invalid_address_with_four_parts_panics() {
    // Regression: "1.2.3.999" has 4 parts (passes count check)
    // but 999 > 255 (fails parse). Current code skips error branch
    // and panics on unwrap(). This test expects an error, not a panic.
    with_vars(
        [("IMGPRSSR_ADDRESS", Some("1.2.3.999"))],
        || {
            let result = imgprssr::server_config::generate_app_config();
            assert_eq!(
                result,
                Err(ImgprssrConfigErr::InvalidValues(vec!["address".to_owned()]))
            );
        },
    );
}

#[test]
fn invalid_port_returns_error() {
    with_vars(
        [("IMGPRSSR_PORT", Some("not_a_number"))],
        || {
            let result = imgprssr::server_config::generate_app_config();
            assert_eq!(
                result,
                Err(ImgprssrConfigErr::InvalidValues(vec!["port".to_owned()]))
            );
        },
    );
}

#[test]
fn valid_address_and_port_parsed() {
    with_vars(
        [
            ("IMGPRSSR_ADDRESS", Some("10.0.0.1")),
            ("IMGPRSSR_PORT", Some("8080")),
        ],
        || {
            let result = imgprssr::server_config::generate_app_config();
            assert!(result.is_ok());
            let ((address, port), _config) = result.unwrap();
            assert_eq!(address, [10, 0, 0, 1]);
            assert_eq!(port, 8080);
        },
    );
}

#[test]
fn default_address_is_zeroed() {
    let kvs: [(&str, Option<&str>); 0] = [];
    with_vars(kvs, || {
        let result = imgprssr::server_config::generate_app_config();
        assert!(result.is_ok());
        let ((address, port), _config) = result.unwrap();
        assert_eq!(address, [0, 0, 0, 0]);
        assert_eq!(port, 3000);
    });
}
