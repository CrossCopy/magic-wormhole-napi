#![deny(clippy::all)]
#![allow(clippy::too_many_arguments)]

#[macro_use]
extern crate napi_derive;

use color_eyre::eyre::{self, Context};
use futures::FutureExt;
use indicatif::ProgressBar;
use magic_wormhole::{transfer, transit, MailboxConnection, Wormhole};
use napi::{
  bindgen_prelude::*,
  threadsafe_function::{
    ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode, UnknownReturnValue,
  },
  JsString,
};
use std::{path::PathBuf, sync::Arc};
use std::{thread, time::Duration};

mod error;
mod util;

use error::generic_napi_err;

use crate::error::convert_to_napi_error;

#[napi]
fn hello_world() -> String {
  "Hello, world!".to_string()
}

// fn parse_transit_args(args: &CommonArgs) -> transit::Abilities {
// 	match (args.force_direct, args.force_relay) {
// 			(false, false) => transit::Abilities::ALL_ABILITIES,
// 			(true, false) => transit::Abilities::FORCE_DIRECT,
// 			(false, true) => transit::Abilities::FORCE_RELAY,
// 			(true, true) => unreachable!("These flags are mutually exclusive"),
// 	}
// }

#[napi(object)]
pub struct ProgressHandlerPayload {
  pub sent: BigInt,
  pub total: BigInt,
}

#[napi]
async fn send(
  filepath: String,
  code_callback: ThreadsafeFunction<String>,
  start_callback: ThreadsafeFunction<BigInt>,
  progress_callback: ThreadsafeFunction<ProgressHandlerPayload>,
) -> Result<()> {
  let ctrl_c = util::install_ctrlc_handler().map_err(convert_to_napi_error)?;
  let path = PathBuf::from(filepath);
  let filesize = std::fs::metadata(path.clone()).unwrap().len();
  let offer = transfer::OfferSend::new_paths(vec![path]).await?;
  let relay_hints: Vec<transit::RelayHint> = vec![transit::RelayHint::from_urls(
    None,
    [magic_wormhole::transit::DEFAULT_RELAY_SERVER
      .parse()
      .unwrap()],
  )
  .map_err(convert_to_napi_error)?];
  let app_config = transfer::APP_CONFIG;
  let mailbox_connection = MailboxConnection::create(app_config, 2)
    .await
    .map_err(convert_to_napi_error)?;
  let code = mailbox_connection.code.clone();
  code_callback.call(Ok(code.0), ThreadsafeFunctionCallMode::NonBlocking);
  let wormhole = Wormhole::connect(mailbox_connection)
    .await
    .map_err(convert_to_napi_error)?;

  let transit_abilities = transit::Abilities::ALL_ABILITIES;
  let progress_handler = move |sent, total| {
    progress_callback.call(
      Ok(ProgressHandlerPayload {
        sent: BigInt::from(sent),
        total: BigInt::from(total),
      }),
      ThreadsafeFunctionCallMode::NonBlocking,
    );
  };
  start_callback.call(
    Ok(BigInt::from(filesize)),
    ThreadsafeFunctionCallMode::NonBlocking,
  );
  transfer::send(
    wormhole,
    relay_hints,
    transit_abilities,
    offer,
    &transit::log_transit_connection,
    progress_handler,
    // util::create_progress_handler(pb),
    ctrl_c(),
  )
  .await
  .context("Send process failed")
  .map_err(convert_to_napi_error)?;
  // pb2.finish();

  Ok(())
}

#[napi]
async fn receive(code: String, output_dir: String) -> Result<()> {
  Ok(())
}

#[napi]
pub fn call_threadsafe_function(tsfn: ThreadsafeFunction<u32>) -> Result<()> {
  for n in 0..100 {
    let tsfn = tsfn.clone();
    thread::spawn(move || {
      tsfn.call(Ok(n), ThreadsafeFunctionCallMode::NonBlocking);
    });
  }
  Ok(())
}
