//! Model, storage, business rules and HTTP routes for example items.

mod model;
mod repository;
mod routes;
mod service;

pub use model::{ExampleItem, ExampleItemChanges, NewExampleItem};
pub use repository::{ExampleItemRepository, InMemoryExampleItemRepository};
pub use repository::PgExampleItemRepository; // @tfverse if stack.sqlx
pub use routes::routes;
pub use service::ExampleItemService;
