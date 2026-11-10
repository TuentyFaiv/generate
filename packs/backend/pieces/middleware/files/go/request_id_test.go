package middleware

import (
	"net/http"
	"net/http/httptest"
	"testing"
// @tfverse if stack.echo

	"github.com/labstack/echo/v4"
// @tfverse end
)

func send(header string) *httptest.ResponseRecorder {
	request := httptest.NewRequest(http.MethodGet, "/", nil)
	if header != "" {
		request.Header.Set(RequestIdHeader, header)
	}
	recorder := httptest.NewRecorder()
// @tfverse if stack.echo
	app := echo.New()
	app.Use(RequestId())
	app.GET("/", func(c echo.Context) error { return c.String(http.StatusOK, RequestIdFrom(c.Request().Context())) })
	app.ServeHTTP(recorder, request)
// @tfverse else
	handler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) { _, _ = w.Write([]byte(RequestIdFrom(r.Context()))) })
	RequestId(handler).ServeHTTP(recorder, request)
// @tfverse end
	return recorder
}

func TestValidIncomingIDIsReused(t *testing.T) {
	recorder := send("abc-123")

	if got := recorder.Header().Get(RequestIdHeader); got != "abc-123" {
		t.Fatalf("header = %q", got)
	}
	if recorder.Body.String() != "abc-123" {
		t.Fatalf("body = %q", recorder.Body.String())
	}
}

func TestUnsafeIncomingIDIsReplaced(t *testing.T) {
	if got := send("<script>").Header().Get(RequestIdHeader); len(got) != 36 {
		t.Fatalf("header = %q", got)
	}
}
