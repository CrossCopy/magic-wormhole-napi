use color_eyre::{eyre, eyre::Context};
use eyre::ErrReport;
use napi::Error;

pub struct NapiError(napi::Error);

impl NapiError {
  pub fn into_inner(self) -> napi::Error {
    self.0
  }

  pub fn new(message: String) -> Self {
    NapiError(napi::Error::new(napi::Status::GenericFailure, message))
  }
}

impl From<ErrReport> for NapiError {
  fn from(err: ErrReport) -> Self {
    NapiError(napi::Error::new(
      napi::Status::GenericFailure,
      format!("Error: {}", err),
    ))
  }
}

impl From<magic_wormhole::forwarding::ForwardingError> for NapiError {
  fn from(err: magic_wormhole::forwarding::ForwardingError) -> Self {
    NapiError(napi::Error::new(
      napi::Status::GenericFailure,
      format!("Error: {}", err),
    ))
  }
}

pub fn convert_to_napi_error(err: impl ToString) -> napi::Error {
  napi::Error::new(napi::Status::GenericFailure, err.to_string())
}

pub fn generic_napi_err(errmsg: &str) -> napi::Error {
  napi::Error::new(napi::Status::GenericFailure, errmsg)
}
