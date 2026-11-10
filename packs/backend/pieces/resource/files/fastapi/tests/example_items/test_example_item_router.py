from uuid import uuid4

import pytest
from fastapi import FastAPI
from fastapi.testclient import TestClient

from app.errors import register_error_handlers
from app.example_items.dependencies import get_example_item_service
from app.example_items.repository import InMemoryExampleItemRepository
from app.example_items.router import router
from app.example_items.service import ExampleItemService


@pytest.fixture
def client() -> TestClient:
    app = FastAPI()
    register_error_handlers(app)
    app.include_router(router)
    service = ExampleItemService(InMemoryExampleItemRepository())
    app.dependency_overrides[get_example_item_service] = lambda: service
    return TestClient(app)


def test_post_creates_an_item(client: TestClient) -> None:
    created = client.post("/example-items", json={"name": "First"})

    assert created.status_code == 201
    assert client.get(f"/example-items/{created.json()['id']}").json()["name"] == "First"


def test_invalid_input_is_a_problem(client: TestClient) -> None:
    response = client.post("/example-items", json={"name": ""})

    assert response.status_code == 422
    assert response.headers["content-type"] == "application/problem+json"


def test_missing_item_is_a_404(client: TestClient) -> None:
    assert client.get(f"/example-items/{uuid4()}").status_code == 404
