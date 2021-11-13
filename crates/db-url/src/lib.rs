use std::path::Path;

pub fn db_url(name: &str) -> String {
  format!("postgresql://localhost/{}", name)
}
