use chrono::Utc;
use uuid::Uuid;

use super::model::valid_name;
use super::{ExampleItem, ExampleItemChanges, ExampleItemRepository, NewExampleItem};
use crate::error::AppError;

/// Business rules for example items.
#[derive(Debug)]
pub struct ExampleItemService<R> {
	repository: R,
}

impl<R: ExampleItemRepository> ExampleItemService<R> {
	/// A service storing items in `repository`.
	pub fn new(repository: R) -> Self {
		Self { repository }
	}

	/// A page of items, oldest first.
	pub async fn list(&self, limit: u32, offset: u32) -> Result<Vec<ExampleItem>, AppError> {
		self.repository.list(limit, offset).await
	}

	/// The item with `id`.
	pub async fn get(&self, id: Uuid) -> Result<ExampleItem, AppError> {
		self.repository.find(id).await?.ok_or(AppError::NotFound)
	}

	/// Validates and stores a new item.
	pub async fn create(&self, new: NewExampleItem) -> Result<ExampleItem, AppError> {
		let now = Utc::now();
		let item = ExampleItem {
			id: Uuid::now_v7(),
			name: valid_name(&new.name)?,
			created_at: now,
			updated_at: now,
		};
		self.repository.insert(item).await
	}

	/// Applies `changes` to the item with `id`.
	pub async fn update(&self, id: Uuid, changes: ExampleItemChanges) -> Result<ExampleItem, AppError> {
		let mut item = self.get(id).await?;
		if let Some(name) = changes.name {
			item.name = valid_name(&name)?;
		}
		item.updated_at = Utc::now();
		self.repository.update(item).await?.ok_or(AppError::NotFound)
	}

	/// Deletes the item with `id`.
	pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
		if self.repository.delete(id).await? {
			Ok(())
		} else {
			Err(AppError::NotFound)
		}
	}
}
// @tfverse if tests

#[cfg(test)]
mod tests {
	use super::*;
	use crate::example_items::InMemoryExampleItemRepository;

	fn service() -> ExampleItemService<InMemoryExampleItemRepository> {
		ExampleItemService::new(InMemoryExampleItemRepository::default())
	}

	fn new(name: &str) -> NewExampleItem {
		NewExampleItem { name: name.to_owned() }
	}

	#[tokio::test]
	async fn create_should_trim_and_store_the_item() {
		let service = service();

		let item = service.create(new("  First  ")).await.unwrap();

		assert_eq!(item.name, "First");
		assert_eq!(service.get(item.id).await.unwrap(), item);
	}

	#[tokio::test]
	async fn create_should_reject_an_empty_name() {
		assert!(matches!(service().create(new(" ")).await, Err(AppError::Validation(_))));
	}

	#[tokio::test]
	async fn update_should_keep_fields_that_are_not_changed() {
		let service = service();
		let item = service.create(new("First")).await.unwrap();

		let updated = service.update(item.id, ExampleItemChanges::default()).await.unwrap();

		assert_eq!(updated.name, "First");
		assert!(updated.updated_at >= item.updated_at);
	}

	#[tokio::test]
	async fn delete_should_report_a_missing_item() {
		assert!(matches!(service().delete(Uuid::now_v7()).await, Err(AppError::NotFound)));
	}
}
// @tfverse end
