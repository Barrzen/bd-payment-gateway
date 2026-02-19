#![cfg(feature = "all-providers")]

#[tokio::test]
#[ignore = "Live tests are disabled by default. Set BD_PAYMENT_GATEWAY_RUN_LIVE_TESTS=1 and provider credentials."]
async fn live_smoke_tests_guard() {
    assert_eq!(
        std::env::var("BD_PAYMENT_GATEWAY_RUN_LIVE_TESTS")
            .ok()
            .as_deref(),
        Some("1"),
        "Set BD_PAYMENT_GATEWAY_RUN_LIVE_TESTS=1 to explicitly opt in to live integration tests."
    );
}
