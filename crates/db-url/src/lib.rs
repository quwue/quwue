use std::path::Path;

pub fn db_url(path: &Path) -> Option<String> {
  let path = path.to_str()?;

  Some(if cfg!(windows) {
    format!("sqlite:///{}", path.replace('\\', "\\\\"))
  } else {
    format!("sqlite:{}", path)
  })
}
