use thiserror::Error;

#[derive(Error, Debug)]
pub enum VError {
    #[error("Not logged in to e-register!")]
    NotLoggedIn,

    #[error("Website element not found: {0} on {1}!")]
    ElementNotFound(String, String),

    #[error("Website element attribute not found: {0} on {1} on {2}!")]
    ElementAttributeNotFound(String, String, String),

    #[error("API format on endpoint: {0} changed. Field {1} not found!")]
    ApiFormatChanged(String, String),
}
