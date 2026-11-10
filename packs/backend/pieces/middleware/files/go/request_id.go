// Package middleware holds HTTP middleware.
package middleware

import (
	"context"
	"net/http" // @tfverse if !stack.echo
	"regexp"

	"github.com/google/uuid"
	"github.com/labstack/echo/v4" // @tfverse if stack.echo
)

// RequestIdHeader carries the request id.
const RequestIdHeader = "x-request-id"

var validRequestId = regexp.MustCompile(`^[\w-]{1,128}$`)

type requestIdKey struct{}

// RequestIdFrom returns the request id stored by the middleware, or "".
func RequestIdFrom(ctx context.Context) string {
	id, _ := ctx.Value(requestIdKey{}).(string)
	return id
}

func requestId(incoming string) string {
	if validRequestId.MatchString(incoming) {
		return incoming
	}
	return uuid.NewString()
}
// @tfverse if stack.echo

// RequestId reuses a valid incoming request id or creates one, stores it in the request context and echoes it.
func RequestId() echo.MiddlewareFunc {
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			id := requestId(c.Request().Header.Get(RequestIdHeader))
			c.SetRequest(c.Request().WithContext(context.WithValue(c.Request().Context(), requestIdKey{}, id)))
			c.Response().Header().Set(RequestIdHeader, id)
			return next(c)
		}
	}
}
// @tfverse else

// RequestId reuses a valid incoming request id or creates one, stores it in the request context and echoes it.
func RequestId(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		id := requestId(r.Header.Get(RequestIdHeader))
		w.Header().Set(RequestIdHeader, id)
		next.ServeHTTP(w, r.WithContext(context.WithValue(r.Context(), requestIdKey{}, id)))
	})
}
// @tfverse end
