use crate::common::*;

#[async_trait]
pub(crate) trait ResponseFutureExt<T> {
  async fn optional_model(self) -> Result<Option<T>>;
}

#[async_trait]
impl<T: Unpin + DeserializeOwned + Send> ResponseFutureExt<T> for ResponseFuture<T> {
  async fn optional_model(self) -> Result<Option<T>> {
    match self.await {
      Ok(response) => Ok(Some(response.model().await?)),
      Err(error) => {
        if let twilight_http::error::ErrorType::Response { status, .. } = error.kind() {
          if status.raw() == 404 {
            return Ok(None);
          }
        }
        Err(error.into())
      },
    }
  }
}
