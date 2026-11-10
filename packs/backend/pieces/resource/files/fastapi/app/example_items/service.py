"""Business rules for example items."""

from dataclasses import replace
from datetime import UTC, datetime
from uuid import UUID, uuid4

from app.errors import NotFoundError

from .models import ExampleItem
from .repository import ExampleItemRepository
from .schemas import ExampleItemCreate, ExampleItemUpdate


class ExampleItemService:
    """Creates, reads, changes and deletes example items."""

    def __init__(self, repository: ExampleItemRepository) -> None:
        self._repository = repository

    async def list(self, *, limit: int, offset: int) -> list[ExampleItem]:
        return await self._repository.list(limit=limit, offset=offset)

    async def get(self, item_id: UUID) -> ExampleItem:
        item = await self._repository.get(item_id)
        if item is None:
            raise NotFoundError("example item", item_id)
        return item

    async def create(self, data: ExampleItemCreate) -> ExampleItem:
        now = datetime.now(UTC)
        return await self._repository.add(ExampleItem(id=uuid4(), name=data.name, created_at=now, updated_at=now))

    async def update(self, item_id: UUID, data: ExampleItemUpdate) -> ExampleItem:
        current = await self.get(item_id)
        changes = data.model_dump(exclude_unset=True, exclude_none=True)
        saved = await self._repository.save(replace(current, **changes, updated_at=datetime.now(UTC)))
        if saved is None:
            raise NotFoundError("example item", item_id)
        return saved

    async def delete(self, item_id: UUID) -> None:
        if not await self._repository.delete(item_id):
            raise NotFoundError("example item", item_id)
