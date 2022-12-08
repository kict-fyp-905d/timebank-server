use std::task::{Context, Poll};

use hyper::Body;
use tonic::body::BoxBody;
use tower::{Layer, Service};
use tracing::info;

#[derive(Debug, Clone, Default)]
pub struct RequestLoggerLayer;

impl<S> Layer<S> for RequestLoggerLayer {
    type Service = RequestLogger<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequestLogger { inner }
    }
}

#[derive(Debug, Clone)]
pub struct RequestLogger<S> {
    inner: S,
}

impl<S> Service<hyper::Request<Body>> for RequestLogger<S>
where
    S: Service<hyper::Request<Body>, Response = hyper::Response<BoxBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: hyper::Request<Body>) -> Self::Future {
        // This is necessary because tonic internally uses `tower::buffer::Buffer`.
        // See https://github.com/tower-rs/tower/issues/547#issuecomment-767629149
        // for details on why this is necessary
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            // Do extra async work here...
            let uri = req.uri().path();
            let method = req.method();

            info!("{}", format!("{method} {uri}"));

            let response = inner.call(req).await?;
            Ok(response)
        })
    }
}
