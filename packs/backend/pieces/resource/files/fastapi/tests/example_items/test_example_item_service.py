from uuid import uuid4

import pytest

from app.errors import NotFoundError
from app.example_items.repository import InMemoryExampleItemRepository
from app.example_items.schemas import ExampleItemCreate, ExampleItemUpdate
from app.example_items.service import ExampleItemService

pytestmark = pytest.mark.anyio


@pytest.fixture
def anyio_backend() -> str:
    return "asyncio"


@pytest.fixture
def service() -> ExampleItemService:
    return ExampleItemService(InMemoryExampleItemRepository())


async def test_create_then_get_returns_the_item(service: ExampleItemService) -> None:
    item = await service.create(ExampleItemCreate(name="  First  "))

    assert item.name == "First"
    assert await service.get(item.id) == item


async def test_update_keeps_fields_that_are_not_sent(service: ExampleItemService) -> None:
    item = await service.create(ExampleItemCreate(name="First"))

    updated = await service.update(item.id, ExampleItemUpdate())

    assert updated.name == "First"
    assert updated.updated_at >= item.updated_at


async def test_missing_items_raise_not_found(service: ExampleItemService) -> None:
    with pytest.raises(NotFoundError):
        await service.delete(uuid4())
