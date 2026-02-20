#[cfg(feature = "shurjopay")]
#[test]
fn shurjopay_feature_reexports_client() {
    let _ = std::any::TypeId::of::<bd_payment_gateway::shurjopay::ShurjopayClient>();
}

#[cfg(feature = "portwallet")]
#[test]
fn portwallet_feature_reexports_client() {
    let _ = std::any::TypeId::of::<bd_payment_gateway::portwallet::PortwalletClient>();
}

#[cfg(feature = "aamarpay")]
#[test]
fn aamarpay_feature_reexports_client() {
    let _ = std::any::TypeId::of::<bd_payment_gateway::aamarpay::AamarpayClient>();
}

#[cfg(feature = "sslcommerz")]
#[test]
fn sslcommerz_feature_reexports_client() {
    let _ = std::any::TypeId::of::<bd_payment_gateway::sslcommerz::SslcommerzClient>();
}
