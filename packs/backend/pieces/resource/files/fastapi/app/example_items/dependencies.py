"""Wires the example item service for FastAPI."""

# @tfverse if stack.sqlalchemy
from typing import Annotated

from fastapi import Depends
from sqlalchemy.ext.asyncio import AsyncSession

from app.database import get_session

from .repository import SqlAlchemyExampleItemRepository
from .service import ExampleItemService


def get_example_item_service(session: Annotated[AsyncSession, Depends(get_session)]) -> ExampleItemService:
    """A service backed by the request's database session."""
    return ExampleItemService(SqlAlchemyExampleItemRepository(session))
# @tfverse else
from functools import cache

from .repository import InMemoryExampleItemRepository
from .service import ExampleItemService


@cache
def get_example_item_service() -> ExampleItemService:
    """One in-memory service per process; swap in a database-backed repository for production."""
    return ExampleItemService(InMemoryExampleItemRepository())
# @tfverse end
