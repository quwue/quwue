use crate::common::*;

#[derive(StructOpt)]
pub(crate) struct Arguments {
  #[structopt(long)]
  pub(crate) db_name: String,
  #[structopt(long)]
  pub(crate) log_dir: Option<PathBuf>,
}
