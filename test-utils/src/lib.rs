use std::convert::TryFrom;

use reqwest::Url;

#[macro_export]
macro_rules! mock_server {
    ($status:expr $(, $func:tt ($($arg:expr),*))*) => {{
        let mock_server = wiremock::MockServer::start().await;
        let template = wiremock::ResponseTemplate::new(http::StatusCode::from($status));
        let template = template$(.$func($($arg),*))*;
        wiremock::Mock::given(wiremock::matchers::method("GET")).respond_with(template).mount(&mock_server).await;
        mock_server
    }};
}

pub async fn get_mock_client_response<T, E>(request: T) -> lychee_lib::Response
where
    lychee_lib::Request: TryFrom<T, Error = E>,
    lychee_lib::ErrorKind: From<E>,
{
    lychee_lib::ClientBuilder::default()
        .client()
        .unwrap()
        .check(request)
        .await
        .unwrap()
}

/// Helper method to convert a string into a URI
/// Note: This panics on error, so it should only be used for testing
pub fn website(url: &str) -> lychee_lib::Uri {
    lychee_lib::Uri::from(Url::parse(url).expect("Expected valid Website URI"))
}

pub fn mail(address: &str) -> lychee_lib::Uri {
    if address.starts_with("mailto:") {
        Url::parse(address)
    } else {
        Url::parse(&(String::from("mailto:") + address))
    }
    .expect("Expected valid Mail Address")
    .into()
}
