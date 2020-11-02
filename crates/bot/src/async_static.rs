#[macro_export]
macro_rules! async_static {
  {$name:ident, $ty:ty, $value:expr} => {
    mod $name {
      use super::*;

      use once_cell::sync::OnceCell;
      use tokio::sync::Mutex;

      static VALUE: OnceCell<$ty> = OnceCell::new();
      static INITIALIZED: OnceCell<Mutex<bool>> = OnceCell::new();

      pub(crate) async fn get() -> &'static $ty {
        let mutex = INITIALIZED.get_or_init(|| Mutex::new(false));

        let mut initialized = mutex.lock().await;

        if !*initialized {
          let value = $value;
          VALUE.set(value).expect("We are holding INITIALIZED mutex.");
          *initialized = true;
          drop(initialized);
        }

        VALUE.get().unwrap()
      }
    }
  }
}
