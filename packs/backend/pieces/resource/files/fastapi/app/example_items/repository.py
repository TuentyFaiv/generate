"""Storage for example items."""

from typing import Protocol
from uuid import UUID

# @tfverse if stack.sqlalchemy
from sqlalchemy import delete, select
from sqlalchemy.ext.asyncio import AsyncSession

# @tfverse end
from .models import ExampleItem
from .tables import ExampleItemRow  # @tfverse if stack.sqlalchemy


class ExampleItemRepository(Protocol):
    """What the service needs from storage."""

    async def list(self, *, limit: int, offset: int) -> list[ExampleItem]: ...

    async def get(self, item_id: UUID) -> ExampleItem | None: ...

    async def add(self, item: ExampleItem) -> ExampleItem: ...

    async def save(self, item: ExampleItem) -> ExampleItem | None: ...

    async def delete(self, item_id: UUID) -> bool: ...


class InMemoryExampleItemRepository:
    """Keeps example items in memory, for tests and prototypes."""

    def __init__(self) -> None:
        self._items: dict[UUID, ExampleItem] = {}

    async def list(self, *, limit: int, offset: int) -> list[ExampleItem]:
        items = sorted(self._items.values(), key=lambda item: item.created_at)
        return items[offset : offset + limit]

    async def get(self, item_id: UUID) -> ExampleItem | None:
        return self._items.get(item_id)

    async def add(self, item: ExampleItem) -> ExampleItem:
        self._items[item.id] = item
        return item

    async def save(self, item: ExampleItem) -> ExampleItem | None:
        if item.id not in self._items:
            return None
        self._items[item.id] = item
        return item

    async def delete(self, item_id: UUID) -> bool:
        return self._items.pop(item_id, None) is not None
# @tfverse if stack.sqlalchemy


class SqlAlchemyExampleItemRepository:
    """Stores example items in the `example_items` table."""

    def __init__(self, session: AsyncSession) -> None:
        self._session = session

    async def list(self, *, limit: int, offset: int) -> list[ExampleItem]:
        query = select(ExampleItemRow).order_by(ExampleItemRow.created_at).limit(limit).offset(offset)
        return [row.to_entity() for row in await self._session.scalars(query)]

    async def get(self, item_id: UUID) -> ExampleItem | None:
        row = await self._session.get(ExampleItemRow, item_id)
        return row.to_entity() if row else None

    async def add(self, item: ExampleItem) -> ExampleItem:
        self._session.add(ExampleItemRow.from_entity(item))
        await self._session.commit()
        return item

    async def save(self, item: ExampleItem) -> ExampleItem | None:
        row = await self._session.get(ExampleItemRow, item.id)
        if row is None:
            return None
        row.name = item.name
        row.updated_at = item.updated_at
        await self._session.commit()
        return item

    async def delete(self, item_id: UUID) -> bool:
        deleted = await self._session.scalars(
            delete(ExampleItemRow).where(ExampleItemRow.id == item_id).returning(ExampleItemRow.id)
        )
        await self._session.commit()
        return deleted.first() is not None
# @tfverse end
