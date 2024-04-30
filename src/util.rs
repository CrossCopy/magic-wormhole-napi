use std::sync::Arc;

use async_std::{io, io::prelude::*};
use color_eyre::eyre::{self, Context};
use futures::{Future, FutureExt};
use indicatif::ProgressBar;
use magic_wormhole::{transfer, transit};

pub async fn ask_user(message: impl std::fmt::Display, default_answer: bool) -> bool {
  let message = format!(
    "{} ({}/{}) ",
    message,
    if default_answer { "Y" } else { "y" },
    if default_answer { "n" } else { "N" }
  );

  let mut stdout = io::stdout();
  let stdin = io::stdin();

  loop {
    stdout.write(message.as_bytes()).await.unwrap();

    stdout.flush().await.unwrap();

    let mut answer = String::new();
    stdin.read_line(&mut answer).await.unwrap();

    match answer.to_lowercase().trim() {
      "y" | "yes" => break true,
      "n" | "no" => break false,
      "" => break default_answer,
      _ => {
        stdout
          .write("Please type y or n!\n".as_bytes())
          .await
          .unwrap();
        stdout.flush().await.unwrap();
        continue;
      }
    };
  }
}

/// A weird mixture of [`futures::future::Abortable`], [`async_std::sync::Condvar`] and [`futures::future::Select`] tailored to our Ctrl+C handling.
///
/// At it's core, it is an `Abortable` but instead of having an `AbortHandle`, we use a future that resolves as trigger.
/// Under the hood, it is implementing the same functionality as a `select`, but mapping one of the outcomes to an error type.
pub async fn cancellable<T>(
  future: impl Future<Output = T> + Unpin,
  cancel: impl Future<Output = ()>,
) -> Result<T, Cancelled> {
  use futures::future::Either;
  futures::pin_mut!(cancel);
  match futures::future::select(future, cancel).await {
    Either::Left((val, _)) => Ok(val),
    Either::Right(((), _)) => Err(Cancelled),
  }
}

/// Indicator that the [`Cancellable`] task was cancelled.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Cancelled;

impl std::fmt::Display for Cancelled {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Task has been cancelled")
  }
}

pub fn create_progress_bar(file_size: u64) -> ProgressBar {
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

pub fn create_progress_handler(pb: ProgressBar) -> impl FnMut(u64, u64) {
  move |sent, total| {
    if sent == 0 {
      pb.reset_elapsed();
      pb.set_length(total);
      pb.enable_steady_tick(std::time::Duration::from_millis(250));
    }
    pb.set_position(sent);
  }
}

pub fn install_ctrlc_handler(
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
