"""Application errors, sent to clients as RFC 9457 problem details."""

import logging
from typing import Any

from fastapi import FastAPI, Request
from fastapi.encoders import jsonable_encoder
from fastapi.exceptions import RequestValidationError
from fastapi.responses import JSONResponse

logger = logging.getLogger(__name__)


class AppError(Exception):
    """An error with an HTTP status."""

    status_code = 500
    title = "Internal Server Error"

    def __init__(self, detail: str) -> None:
        super().__init__(detail)
        self.detail = detail


class NotFoundError(AppError):
    """The requested resource does not exist."""

    status_code = 404
    title = "Not Found"

    def __init__(self, resource: str, identifier: object) -> None:
        super().__init__(f"No {resource} with id {identifier}.")


class ConflictError(AppError):
    """The request conflicts with the current state, such as a duplicate."""

    status_code = 409
    title = "Conflict"


def problem(status: int, title: str, detail: str, **extra: Any) -> JSONResponse:
    """A problem details response."""
    body = {"type": "about:blank", "title": title, "status": status, "detail": detail, **extra}
    return JSONResponse(body, status_code=status, media_type="application/problem+json")


def register_error_handlers(app: FastAPI) -> None:
    """Sends application, validation and unexpected errors as problem details."""

    @app.exception_handler(AppError)
    async def handle_app_error(_request: Request, error: AppError) -> JSONResponse:
        return problem(error.status_code, error.title, error.detail)

    @app.exception_handler(RequestValidationError)
    async def handle_validation_error(_request: Request, error: RequestValidationError) -> JSONResponse:
        errors = jsonable_encoder(error.errors(), exclude={"input", "ctx"})
        return problem(422, "Unprocessable Content", "The request is invalid.", errors=errors)

    @app.exception_handler(Exception)
    async def handle_unexpected_error(_request: Request, error: Exception) -> JSONResponse:
        logger.exception("request failed", exc_info=error)
        return problem(500, "Internal Server Error", "Something went wrong.")
