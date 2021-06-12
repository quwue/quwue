use crate::common::*;

static NEXT_ACTION_START_TIME: Mutex<Option<Instant>> = Mutex::const_new(None);

pub(crate) async fn wait() {
  let mut next_action_start_time = NEXT_ACTION_START_TIME.lock().await;

  let now = Instant::now();

  let instant = next_action_start_time.get_or_insert(now);

  if let Some(duration) = instant.checked_duration_since(now) {
    tokio::time::sleep(duration).await;
  }

  *next_action_start_time = Some(Instant::now() + Duration::from_millis(1100));
}
