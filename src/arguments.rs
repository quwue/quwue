use crate::common::*;

#[derive(StructOpt)]
pub(crate) struct Arguments {
  #[structopt(long)]
  pub(crate) db_path: PathBuf,
  #[structopt(long)]
  pub(crate) log_dir: Option<PathBuf>,
}
