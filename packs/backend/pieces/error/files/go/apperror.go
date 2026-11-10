// Package apperror defines application errors, sent to clients as RFC 9457 problem details.
package apperror

import (
	"encoding/json"
	"errors"
	"log/slog"
	"net/http"
// @tfverse if stack.echo

	"github.com/labstack/echo/v4"
// @tfverse end
)

// Error is an error with an HTTP status.
type Error struct {
	Status int
	Detail string
	Err    error
}

func (e *Error) Error() string { return e.Detail }

func (e *Error) Unwrap() error { return e.Err }

// NotFound reports that the requested resource does not exist.
func NotFound(detail string) *Error {
	return &Error{Status: http.StatusNotFound, Detail: detail}
}

// Invalid reports a well-formed request with invalid content.
func Invalid(detail string) *Error {
	return &Error{Status: http.StatusUnprocessableEntity, Detail: detail}
}

// BadRequest reports a malformed request, such as invalid JSON.
func BadRequest(detail string, err error) *Error {
	return &Error{Status: http.StatusBadRequest, Detail: detail, Err: err}
}

// Conflict reports a request that conflicts with the current state, such as a duplicate.
func Conflict(detail string) *Error {
	return &Error{Status: http.StatusConflict, Detail: detail}
}

// Problem is an RFC 9457 problem details body.
type Problem struct {
	Type   string `json:"type"`
	Title  string `json:"title"`
	Status int    `json:"status"`
	Detail string `json:"detail"`
}

// ProblemFor turns any error into problem details; unknown errors never leak their message.
func ProblemFor(err error) Problem {
	var appErr *Error
	if errors.As(err, &appErr) {
		return Problem{Type: "about:blank", Title: http.StatusText(appErr.Status), Status: appErr.Status, Detail: appErr.Detail}
	}
	return Problem{
		Type:   "about:blank",
		Title:  http.StatusText(http.StatusInternalServerError),
		Status: http.StatusInternalServerError,
		Detail: "Something went wrong.",
	}
}

// Write sends err as problem details, logging unexpected errors.
func Write(w http.ResponseWriter, r *http.Request, err error) {
	problem := ProblemFor(err)
	if problem.Status >= http.StatusInternalServerError {
		slog.ErrorContext(r.Context(), "request failed", "error", err, "path", r.URL.Path)
	}
	w.Header().Set("Content-Type", "application/problem+json")
	w.WriteHeader(problem.Status)
	_ = json.NewEncoder(w).Encode(problem)
}
// @tfverse if stack.echo

// Handler is an echo.HTTPErrorHandler that sends every error as problem details.
func Handler(err error, c echo.Context) {
	if c.Response().Committed {
		return
	}
	var httpErr *echo.HTTPError
	if errors.As(err, &httpErr) {
		err = &Error{Status: httpErr.Code, Detail: http.StatusText(httpErr.Code), Err: httpErr}
	}
	Write(c.Response(), c.Request(), err)
}
// @tfverse end
