use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use db::Db;
use std::time::SystemTime;
use twilight_model::id::UserId;

async fn benchmark(db: &Db, id: u64) {
  let user_id = UserId(id);
  db.user(user_id).await.unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
  let db_url = db_url::db_url(&format!(
    "quwue-bench-{}",
    SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)
      .unwrap()
      .as_millis()
  ));

  let runtime = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .unwrap();
  let db = runtime.block_on(async { Db::connect(&db_url).await.unwrap() });
  let mut id = 0;
  c.bench_with_input(BenchmarkId::new("benchmark", 1000), &db, |b, s| {
    b.to_async(&runtime).iter(|| {
      id += 1;
      benchmark(s, id)
    })
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
