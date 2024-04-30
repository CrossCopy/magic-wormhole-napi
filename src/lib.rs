#![deny(clippy::all)]
#![allow(clippy::too_many_arguments)]

#[macro_use]
extern crate napi_derive;

use std::{path::PathBuf, sync::Arc};

use color_eyre::eyre::{self, Context};
use futures::FutureExt;
use indicatif::ProgressBar;
use magic_wormhole::{transfer, transit, MailboxConnection, Wormhole};
use napi::Result;

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

fn create_progress_bar(file_size: u64) -> ProgressBar {
  use indicatif::ProgressStyle;

  let pb = ProgressBar::new(file_size);
  pb.set_style(
    ProgressStyle::default_bar()
      // .template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
      .template("[{elapsed_precise}] [{wide_bar}] {bytes}/{total_bytes} ({eta})")
      .unwrap()
      .progress_chars("#>-"),
  );
  pb
}

fn create_progress_handler(pb: ProgressBar) -> impl FnMut(u64, u64) {
  move |sent, total| {
    if sent == 0 {
      pb.reset_elapsed();
      pb.set_length(total);
      pb.enable_steady_tick(std::time::Duration::from_millis(250));
    }
    pb.set_position(sent);
  }
}

fn install_ctrlc_handler(
) -> eyre::Result<impl Fn() -> futures::future::BoxFuture<'static, ()> + Clone> {
  use async_std::sync::{Condvar, Mutex};

  let notifier = Arc::new((Mutex::new(false), Condvar::new()));

  /* Register the handler */
  let notifier2 = notifier.clone();
  ctrlc::set_handler(move || {
    futures::executor::block_on(async {
      let mut has_notified = notifier2.0.lock().await;
      if *has_notified {
        /* Second signal. Exit */
        log::debug!("Exit.");
        std::process::exit(130);
      }
      /* First signal. */
      log::info!("Got Ctrl-C event. Press again to exit immediately");
      *has_notified = true;
      notifier2.1.notify_all();
    })
  })
  .context("Error setting Ctrl-C handler")?;

  Ok(move || {
    /* Transform the notification into a future that waits */
    let notifier = notifier.clone();
    async move {
      let (lock, cvar) = &*notifier;
      let mut started = lock.lock().await;
      while !*started {
        started = cvar.wait(started).await;
      }
    }
    .boxed()
  })
}

#[napi]
async fn send(filepath: String) -> Result<String> {
  let ctrl_c = install_ctrlc_handler().map_err(convert_to_napi_error)?;
  let path = PathBuf::from(filepath);
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
	println!("code: {:?}", code.0);
  let wormhole = Wormhole::connect(mailbox_connection)
    .await
    .map_err(convert_to_napi_error)?;

  let transit_abilities = transit::Abilities::ALL_ABILITIES;
  // progress_handler: impl FnMut(u64, u64) + 'static,
  // cancel: impl Future<Output = ()>,
  let pb = create_progress_bar(0);
  let pb2 = pb.clone();
  transfer::send(
    wormhole,
    relay_hints,
    transit_abilities,
    offer,
    &transit::log_transit_connection,
    create_progress_handler(pb),
    ctrl_c(),
  )
  .await
  .context("Send process failed")
  .map_err(convert_to_napi_error)?;
  pb2.finish();

  Ok(code.0)
}
