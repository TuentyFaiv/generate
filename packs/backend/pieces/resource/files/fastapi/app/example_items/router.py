"""HTTP routes for example items; include them with `app.include_router(router)`."""

from typing import Annotated
from uuid import UUID

from fastapi import APIRouter, Depends, Query, status

from .dependencies import get_example_item_service
from .models import ExampleItem
from .schemas import ExampleItemCreate, ExampleItemRead, ExampleItemUpdate
from .service import ExampleItemService

router = APIRouter(prefix="/example-items", tags=["example items"])

Service = Annotated[ExampleItemService, Depends(get_example_item_service)]


@router.get("", response_model=list[ExampleItemRead])
async def list_example_items(
    service: Service,
    limit: Annotated[int, Query(ge=1, le=100)] = 20,
    offset: Annotated[int, Query(ge=0)] = 0,
) -> list[ExampleItem]:
    return await service.list(limit=limit, offset=offset)


@router.get("/{item_id}", response_model=ExampleItemRead)
async def get_example_item(item_id: UUID, service: Service) -> ExampleItem:
    return await service.get(item_id)


@router.post("", response_model=ExampleItemRead, status_code=status.HTTP_201_CREATED)
async def create_example_item(data: ExampleItemCreate, service: Service) -> ExampleItem:
    return await service.create(data)


@router.patch("/{item_id}", response_model=ExampleItemRead)
async def update_example_item(item_id: UUID, data: ExampleItemUpdate, service: Service) -> ExampleItem:
    return await service.update(item_id, data)


@router.delete("/{item_id}", status_code=status.HTTP_204_NO_CONTENT)
async def delete_example_item(item_id: UUID, service: Service) -> None:
    await service.delete(item_id)
