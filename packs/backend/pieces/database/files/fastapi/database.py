"""SQLAlchemy engine, sessions and the declarative base."""

import os
from collections.abc import AsyncIterator
from functools import cache

from sqlalchemy.ext.asyncio import AsyncEngine, AsyncSession, async_sessionmaker, create_async_engine
from sqlalchemy.orm import DeclarativeBase


class Base(DeclarativeBase):
    """Base class of every table."""


@cache
def get_engine() -> AsyncEngine:
    """The engine for `DATABASE_URL`, created on first use."""
    return create_async_engine(os.environ["DATABASE_URL"], pool_pre_ping=True)


@cache
def get_session_factory() -> async_sessionmaker[AsyncSession]:
    """Creates sessions bound to the engine."""
    return async_sessionmaker(get_engine(), expire_on_commit=False)


async def get_session() -> AsyncIterator[AsyncSession]:
    """A session per request, for `Depends(get_session)`."""
    async with get_session_factory()() as session:
        yield session
