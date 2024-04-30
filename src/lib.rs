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
use crate::error::convert_to_napi_error;
use error::generic_napi_err;

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
async fn receive(
  code: String,
  output_dir: String,
  start_callback: ThreadsafeFunction<BigInt>,
  progress_callback: ThreadsafeFunction<ProgressHandlerPayload>,
) -> Result<()> {
  let out_path = PathBuf::from(output_dir);
  let transit_abilities = transit::Abilities::ALL_ABILITIES;
  let code: magic_wormhole::Code = magic_wormhole::Code(code);
  let app_config = transfer::APP_CONFIG;
  let mailbox_connection = MailboxConnection::connect(app_config, code, true)
    .await
    .map_err(convert_to_napi_error)?;
  let wormhole = Wormhole::connect(mailbox_connection)
    .await
    .map_err(convert_to_napi_error)?;
  let relay_hints: Vec<transit::RelayHint> = vec![transit::RelayHint::from_urls(
    None,
    [magic_wormhole::transit::DEFAULT_RELAY_SERVER
      .parse()
      .unwrap()],
  )
  .map_err(convert_to_napi_error)?];
  let ctrl_c = util::install_ctrlc_handler().map_err(convert_to_napi_error)?;
  let req = transfer::request(wormhole, relay_hints, transit_abilities, ctrl_c())
    .await
    .context("Could not get an offer")
    .map_err(convert_to_napi_error)?;
  // turn out_path into &std::path::Path
  let target_dir = out_path.as_path();
  let progress_handler = move |sent, total| {
    progress_callback.call(
      Ok(ProgressHandlerPayload {
        sent: BigInt::from(sent),
        total: BigInt::from(total),
      }),
      ThreadsafeFunctionCallMode::NonBlocking,
    );
  };
  let _: Result<()> = match req {
    Some(transfer::ReceiveRequest::V1(req)) => {
      // receive_inner_v1(req, target_dir, no_confirm, ctrl_c).await
      let file_path = std::path::Path::new(target_dir).join(&req.filename);
      let mut file = async_std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&file_path)
        .await?;
      start_callback.call(
        Ok(BigInt::from(req.filesize)),
        ThreadsafeFunctionCallMode::NonBlocking,
      );
      req
        .accept(
          &transit::log_transit_connection,
          &mut file,
          //   create_progress_handler(pb),
          progress_handler,
          ctrl_c(),
        )
        .await
        .context("Receive process failed")
        .map_err(convert_to_napi_error)?;
      Ok(())
    }
    Some(transfer::ReceiveRequest::V2(req)) => {
      todo!("V2 is not implemented yet")
      // receive_inner_v2(req, target_dir, no_confirm, ctrl_c).await
    }
    None => Ok(()),
  };
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
