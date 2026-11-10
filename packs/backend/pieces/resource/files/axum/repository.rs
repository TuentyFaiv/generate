use std::collections::HashMap;
use std::sync::{PoisonError, RwLock};

// @tfverse if stack.sqlx
use sqlx::PgPool;
// @tfverse end
use uuid::Uuid;

use super::ExampleItem;
use crate::error::AppError;

/// Storage for example items; the service depends on this, not on a database.
pub trait ExampleItemRepository: Send + Sync + 'static {
	/// A page of items, oldest first.
	fn list(&self, limit: u32, offset: u32) -> impl Future<Output = Result<Vec<ExampleItem>, AppError>> + Send;
	/// The item with `id`, if any.
	fn find(&self, id: Uuid) -> impl Future<Output = Result<Option<ExampleItem>, AppError>> + Send;
	/// Stores a new item.
	fn insert(&self, item: ExampleItem) -> impl Future<Output = Result<ExampleItem, AppError>> + Send;
	/// Replaces a stored item; `None` when it does not exist.
	fn update(&self, item: ExampleItem) -> impl Future<Output = Result<Option<ExampleItem>, AppError>> + Send;
	/// Deletes an item; returns whether it existed.
	fn delete(&self, id: Uuid) -> impl Future<Output = Result<bool, AppError>> + Send;
}

/// Keeps example items in memory, for tests and prototypes.
#[derive(Debug, Default)]
pub struct InMemoryExampleItemRepository {
	items: RwLock<HashMap<Uuid, ExampleItem>>,
}

impl ExampleItemRepository for InMemoryExampleItemRepository {
	async fn list(&self, limit: u32, offset: u32) -> Result<Vec<ExampleItem>, AppError> {
		let items = self.items.read().unwrap_or_else(PoisonError::into_inner);
		let mut page: Vec<ExampleItem> = items.values().cloned().collect();
		page.sort_by_key(|item| item.created_at);
		Ok(page.into_iter().skip(offset as usize).take(limit as usize).collect())
	}

	async fn find(&self, id: Uuid) -> Result<Option<ExampleItem>, AppError> {
		Ok(self.items.read().unwrap_or_else(PoisonError::into_inner).get(&id).cloned())
	}

	async fn insert(&self, item: ExampleItem) -> Result<ExampleItem, AppError> {
		let mut items = self.items.write().unwrap_or_else(PoisonError::into_inner);
		items.insert(item.id, item.clone());
		Ok(item)
	}

	async fn update(&self, item: ExampleItem) -> Result<Option<ExampleItem>, AppError> {
		let mut items = self.items.write().unwrap_or_else(PoisonError::into_inner);
		Ok(items.get_mut(&item.id).map(|stored| {
			stored.clone_from(&item);
			item
		}))
	}

	async fn delete(&self, id: Uuid) -> Result<bool, AppError> {
		Ok(self.items.write().unwrap_or_else(PoisonError::into_inner).remove(&id).is_some())
	}
}
// @tfverse if stack.sqlx

/// Stores example items in the PostgreSQL table `example_items`.
#[derive(Clone, Debug)]
pub struct PgExampleItemRepository {
	pool: PgPool,
}

impl PgExampleItemRepository {
	/// A repository using `pool`.
	#[must_use]
	pub fn new(pool: PgPool) -> Self {
		Self { pool }
	}
}

impl ExampleItemRepository for PgExampleItemRepository {
	async fn list(&self, limit: u32, offset: u32) -> Result<Vec<ExampleItem>, AppError> {
		let items = sqlx::query_as(
			"SELECT id, name, created_at, updated_at FROM example_items ORDER BY created_at LIMIT $1 OFFSET $2",
		)
		.bind(i64::from(limit))
		.bind(i64::from(offset))
		.fetch_all(&self.pool)
		.await?;
		Ok(items)
	}

	async fn find(&self, id: Uuid) -> Result<Option<ExampleItem>, AppError> {
		let item = sqlx::query_as("SELECT id, name, created_at, updated_at FROM example_items WHERE id = $1")
			.bind(id)
			.fetch_optional(&self.pool)
			.await?;
		Ok(item)
	}

	async fn insert(&self, item: ExampleItem) -> Result<ExampleItem, AppError> {
		sqlx::query("INSERT INTO example_items (id, name, created_at, updated_at) VALUES ($1, $2, $3, $4)")
			.bind(item.id)
			.bind(&item.name)
			.bind(item.created_at)
			.bind(item.updated_at)
			.execute(&self.pool)
			.await?;
		Ok(item)
	}

	async fn update(&self, item: ExampleItem) -> Result<Option<ExampleItem>, AppError> {
		let result = sqlx::query("UPDATE example_items SET name = $2, updated_at = $3 WHERE id = $1")
			.bind(item.id)
			.bind(&item.name)
			.bind(item.updated_at)
			.execute(&self.pool)
			.await?;
		Ok((result.rows_affected() > 0).then_some(item))
	}

	async fn delete(&self, id: Uuid) -> Result<bool, AppError> {
		let result = sqlx::query("DELETE FROM example_items WHERE id = $1")
			.bind(id)
			.execute(&self.pool)
			.await?;
		Ok(result.rows_affected() > 0)
	}
}
// @tfverse end
