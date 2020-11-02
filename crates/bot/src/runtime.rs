use crate::common::*;

pub(crate) fn init() -> Result<Runtime> {
  let runtime = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .context(error::Runtime)?;

  Ok(runtime)
}
