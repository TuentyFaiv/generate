from fastapi import FastAPI, Request
from fastapi.testclient import TestClient

from app.middleware.request_id import REQUEST_ID_HEADER, RequestIdMiddleware


def client() -> TestClient:
    app = FastAPI()
    app.add_middleware(RequestIdMiddleware)

    @app.get("/")
    async def echo(request: Request) -> str:
        return str(request.state.request_id)

    return TestClient(app)


def test_valid_incoming_id_is_reused() -> None:
    response = client().get("/", headers={REQUEST_ID_HEADER: "abc-123"})

    assert response.headers[REQUEST_ID_HEADER] == "abc-123"
    assert response.json() == "abc-123"


def test_unsafe_incoming_id_is_replaced() -> None:
    response = client().get("/", headers={REQUEST_ID_HEADER: "<script>"})

    assert len(response.headers[REQUEST_ID_HEADER]) == 36
