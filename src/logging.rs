use crate::common::*;

use tracing_subscriber::fmt::Layer;

// TODO: What changed? Why is this failing?

pub(crate) fn init(log_dir: Option<&Path>) {
  LogTracer::init().ok();

  let subscriber = tracing_subscriber::registry()
    .with(EnvFilter::from_default_env())
    .with(Layer::new());

  if let Some(log_dir) = log_dir {
    let appender = tracing_appender::rolling::daily(log_dir, "quwue.log");

    let (non_blocking, _guard) = tracing_appender::non_blocking(appender);

    let subscriber = subscriber.with(Layer::new().with_ansi(false).with_writer(non_blocking));

    tracing::subscriber::set_global_default(subscriber)
  } else {
    tracing::subscriber::set_global_default(subscriber)
  }
  .ok();

  info!("Logging initialized.");
}
