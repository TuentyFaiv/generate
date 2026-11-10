"""Request and response bodies for example items."""

from datetime import datetime
from typing import Annotated
from uuid import UUID

from pydantic import BaseModel, ConfigDict, StringConstraints

Name = Annotated[str, StringConstraints(strip_whitespace=True, min_length=1, max_length=120)]


class ExampleItemCreate(BaseModel):
    """Fields accepted when creating an example item."""

    model_config = ConfigDict(extra="forbid")

    name: Name


class ExampleItemUpdate(BaseModel):
    """Fields accepted when changing an example item; missing ones stay as they are."""

    model_config = ConfigDict(extra="forbid")

    name: Name | None = None


class ExampleItemRead(BaseModel):
    """An example item as returned by the API."""

    model_config = ConfigDict(from_attributes=True)

    id: UUID
    name: str
    created_at: datetime
    updated_at: datetime
