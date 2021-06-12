use crate::common::*;

pub(crate) fn init() {
  use tracing_subscriber::fmt::Layer;

  let appender = tracing_appender::rolling::never(env!("CARGO_MANIFEST_DIR"), "log.txt");

  let (non_blocking, _guard) = tracing_appender::non_blocking(appender);

  let subscriber = tracing_subscriber::registry()
    .with(EnvFilter::from_default_env())
    .with(Layer::new())
    .with(Layer::new().with_ansi(false).with_writer(non_blocking));

  LogTracer::init().expect("Failed to initialize log tracer");

  tracing::subscriber::set_global_default(subscriber)
    .expect("Failed to set global default tracing subscriber");

  info!("Logging initialized.");
}
