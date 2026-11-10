"""The `example_items` table."""

from datetime import datetime
from typing import Self
from uuid import UUID

from sqlalchemy import DateTime, String
from sqlalchemy.orm import Mapped, mapped_column

from app.database import Base

from .models import ExampleItem


class ExampleItemRow(Base):
    """A row of `example_items`."""

    __tablename__ = "example_items"

    id: Mapped[UUID] = mapped_column(primary_key=True)
    name: Mapped[str] = mapped_column(String(120))
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True))
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True))

    @classmethod
    def from_entity(cls, item: ExampleItem) -> Self:
        return cls(id=item.id, name=item.name, created_at=item.created_at, updated_at=item.updated_at)

    def to_entity(self) -> ExampleItem:
        return ExampleItem(id=self.id, name=self.name, created_at=self.created_at, updated_at=self.updated_at)
