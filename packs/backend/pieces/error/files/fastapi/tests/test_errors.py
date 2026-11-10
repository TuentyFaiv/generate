from fastapi import FastAPI
from fastapi.testclient import TestClient

from app.errors import NotFoundError, register_error_handlers


def client() -> TestClient:
    app = FastAPI()
    register_error_handlers(app)

    @app.get("/missing")
    async def missing() -> None:
        raise NotFoundError("invoice", 42)

    @app.get("/broken")
    async def broken() -> None:
        raise RuntimeError("password=hunter2")

    @app.get("/typed/{number}")
    async def typed(number: int) -> int:
        return number

    return TestClient(app, raise_server_exceptions=False)


def test_app_errors_keep_their_status_and_message() -> None:
    response = client().get("/missing")

    assert response.status_code == 404
    assert response.headers["content-type"] == "application/problem+json"
    assert response.json()["detail"] == "No invoice with id 42."


def test_unexpected_errors_hide_their_message() -> None:
    response = client().get("/broken")

    assert response.status_code == 500
    assert "hunter2" not in response.text


def test_validation_errors_list_every_issue() -> None:
    response = client().get("/typed/abc")

    assert response.status_code == 422
    assert response.json()["errors"][0]["loc"] == ["path", "number"]
