"""The example item entity."""

from dataclasses import dataclass
from datetime import datetime
from uuid import UUID


@dataclass(frozen=True, slots=True)
class ExampleItem:
    """An example item as stored and returned by the API."""

    id: UUID
    name: str
    created_at: datetime
    updated_at: datetime
