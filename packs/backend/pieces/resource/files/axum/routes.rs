use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use super::{ExampleItem, ExampleItemChanges, ExampleItemRepository, ExampleItemService, NewExampleItem};
use crate::error::AppError;

const MAX_PAGE: u32 = 100;

#[derive(Debug, Deserialize)]
struct Page {
	#[serde(default = "default_limit")]
	limit: u32,
	#[serde(default)]
	offset: u32,
}

const fn default_limit() -> u32 {
	20
}

type Service<R> = State<Arc<ExampleItemService<R>>>;

/// HTTP routes for example items; nest them with `Router::nest("/example-items", routes(service))`.
pub fn routes<R: ExampleItemRepository>(service: Arc<ExampleItemService<R>>) -> Router {
	Router::new()
		.route("/", get(list::<R>).post(create::<R>))
		.route("/{id}", get(show::<R>).patch(update::<R>).delete(remove::<R>))
		.with_state(service)
}

async fn list<R: ExampleItemRepository>(
	State(service): Service<R>,
	Query(page): Query<Page>,
) -> Result<Json<Vec<ExampleItem>>, AppError> {
	Ok(Json(service.list(page.limit.clamp(1, MAX_PAGE), page.offset).await?))
}

async fn show<R: ExampleItemRepository>(
	State(service): Service<R>,
	Path(id): Path<Uuid>,
) -> Result<Json<ExampleItem>, AppError> {
	Ok(Json(service.get(id).await?))
}

async fn create<R: ExampleItemRepository>(
	State(service): Service<R>,
	Json(new): Json<NewExampleItem>,
) -> Result<(StatusCode, Json<ExampleItem>), AppError> {
	Ok((StatusCode::CREATED, Json(service.create(new).await?)))
}

async fn update<R: ExampleItemRepository>(
	State(service): Service<R>,
	Path(id): Path<Uuid>,
	Json(changes): Json<ExampleItemChanges>,
) -> Result<Json<ExampleItem>, AppError> {
	Ok(Json(service.update(id, changes).await?))
}

async fn remove<R: ExampleItemRepository>(State(service): Service<R>, Path(id): Path<Uuid>) -> Result<StatusCode, AppError> {
	service.delete(id).await?;
	Ok(StatusCode::NO_CONTENT)
}
// @tfverse if tests

#[cfg(test)]
mod tests {
	use axum::body::{Body, to_bytes};
	use axum::http::{Request, header};
	use tower::ServiceExt;

	use super::*;
	use crate::example_items::InMemoryExampleItemRepository;

	fn app() -> Router {
		routes(Arc::new(ExampleItemService::new(InMemoryExampleItemRepository::default())))
	}

	fn post(body: &str) -> Request<Body> {
		Request::post("/")
			.header(header::CONTENT_TYPE, "application/json")
			.body(Body::from(body.to_owned()))
			.unwrap()
	}

	#[tokio::test]
	async fn post_should_create_an_item() {
		let response = app().oneshot(post(r#"{"name":"First"}"#)).await.unwrap();
		let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
		let item: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

		assert_eq!(item["name"], "First");
	}

	#[tokio::test]
	async fn post_should_answer_invalid_names_with_a_problem() {
		let response = app().oneshot(post(r#"{"name":""}"#)).await.unwrap();

		assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
		assert_eq!(response.headers()[header::CONTENT_TYPE], "application/problem+json");
	}

	#[tokio::test]
	async fn get_should_answer_missing_items_with_404() {
		let uri = format!("/{}", Uuid::now_v7());

		let response = app().oneshot(Request::get(uri).body(Body::empty()).unwrap()).await.unwrap();

		assert_eq!(response.status(), StatusCode::NOT_FOUND);
	}
}
// @tfverse end
