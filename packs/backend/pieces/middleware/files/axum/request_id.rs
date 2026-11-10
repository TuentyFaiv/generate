//! Propagates or creates the `x-request-id` header.

use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use uuid::Uuid;

/// Header that carries the request id.
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// The current request id, available to handlers as `Extension<RequestId>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestId(pub String);

/// Reuses a valid incoming request id or creates one, and echoes it in the response.
///
/// Add it with `Router::layer(axum::middleware::from_fn(request_id))`.
pub async fn request_id(mut request: Request, next: Next) -> Response {
	let id = request
		.headers()
		.get(REQUEST_ID_HEADER)
		.and_then(|value| value.to_str().ok())
		.filter(|value| is_valid(value))
		.map_or_else(|| Uuid::now_v7().to_string(), str::to_owned);
	let header = HeaderValue::from_str(&id).ok();
	request.extensions_mut().insert(RequestId(id));

	let mut response = next.run(request).await;
	if let Some(header) = header {
		response.headers_mut().insert(REQUEST_ID_HEADER, header);
	}
	response
}

fn is_valid(id: &str) -> bool {
	(1..=128).contains(&id.len()) && id.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}
// @tfverse if tests

#[cfg(test)]
mod tests {
	use axum::body::Body;
	use axum::routing::get;
	use axum::{Extension, Router};
	use tower::ServiceExt;

	use super::*;

	async fn send(header: Option<&str>) -> Response {
		let app = Router::new()
			.route("/", get(|Extension(RequestId(id)): Extension<RequestId>| async move { id }))
			.layer(axum::middleware::from_fn(request_id));
		let mut request = Request::builder().uri("/");
		if let Some(header) = header {
			request = request.header(REQUEST_ID_HEADER, header);
		}
		app.oneshot(request.body(Body::empty()).unwrap()).await.unwrap()
	}

	#[tokio::test]
	async fn valid_incoming_id_should_be_reused() {
		let response = send(Some("abc-123")).await;

		assert_eq!(response.headers()[REQUEST_ID_HEADER], "abc-123");
	}

	#[tokio::test]
	async fn unsafe_incoming_id_should_be_replaced() {
		let response = send(Some("<script>")).await;

		assert_eq!(response.headers()[REQUEST_ID_HEADER].len(), 36);
	}
}
// @tfverse end
