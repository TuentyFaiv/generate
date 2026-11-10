//! Application errors, sent to clients as RFC 9457 problem details.

use axum::Json;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// Why a request failed.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
	/// The requested resource does not exist.
	#[error("the resource does not exist")]
	NotFound,
	/// The request is well-formed but its content is invalid.
	#[error("{0}")]
	Validation(String),
	/// The request conflicts with the current state, such as a duplicate.
	#[error("{0}")]
	Conflict(String),
	/// An unexpected failure; its details are logged, never sent.
	#[error(transparent)]
	Internal(Box<dyn std::error::Error + Send + Sync>),
}

impl AppError {
	/// Wraps an unexpected error.
	pub fn internal(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
		Self::Internal(error.into())
	}

	fn status(&self) -> StatusCode {
		match self {
			Self::NotFound => StatusCode::NOT_FOUND,
			Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
			Self::Conflict(_) => StatusCode::CONFLICT,
			Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
		}
	}
}
// @tfverse if stack.sqlx

impl From<sqlx::Error> for AppError {
	fn from(error: sqlx::Error) -> Self {
		match error {
			sqlx::Error::RowNotFound => Self::NotFound,
			sqlx::Error::Database(database) if database.is_unique_violation() => {
				Self::Conflict("a record with these values already exists".to_owned())
			}
			other => Self::internal(other),
		}
	}
}
// @tfverse end

#[derive(Debug, Serialize)]
struct Problem {
	#[serde(rename = "type")]
	kind: &'static str,
	title: &'static str,
	status: u16,
	detail: String,
}

impl IntoResponse for AppError {
	fn into_response(self) -> Response {
		let status = self.status();
		let detail = if status.is_server_error() {
			tracing::error!(error = %self, "request failed");
			"Something went wrong.".to_owned()
		} else {
			self.to_string()
		};
		let problem = Problem {
			kind: "about:blank",
			title: status.canonical_reason().unwrap_or("Error"),
			status: status.as_u16(),
			detail,
		};
		(status, [(header::CONTENT_TYPE, "application/problem+json")], Json(problem)).into_response()
	}
}
// @tfverse if tests

#[cfg(test)]
mod tests {
	use axum::body::to_bytes;

	use super::*;

	async fn body(error: AppError) -> (StatusCode, serde_json::Value) {
		let response = error.into_response();
		let status = response.status();
		let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
		(status, serde_json::from_slice(&bytes).unwrap())
	}

	#[tokio::test]
	async fn not_found_should_be_a_404_problem() {
		let (status, problem) = body(AppError::NotFound).await;

		assert_eq!(status, StatusCode::NOT_FOUND);
		assert_eq!(problem["title"], "Not Found");
	}

	#[tokio::test]
	async fn internal_errors_should_hide_their_message() {
		let (status, problem) = body(AppError::internal("password=hunter2")).await;

		assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
		assert!(!problem["detail"].as_str().unwrap().contains("hunter2"));
	}
}
// @tfverse end
