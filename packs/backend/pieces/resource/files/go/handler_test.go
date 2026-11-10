package exampleitems

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/google/uuid"
// @tfverse if stack.echo
	"github.com/labstack/echo/v4"

	"example.com/app/internal/apperror"
// @tfverse end
)

// @tfverse if stack.echo
const collection = "/example-items"
// @tfverse else
const collection = "/example-items/"
// @tfverse end

func server() http.Handler {
	handler := NewHandler(NewService(NewMemoryRepository()))
// @tfverse if stack.echo
	app := echo.New()
	app.HTTPErrorHandler = apperror.Handler
	handler.Register(app.Group("/example-items"))
	return app
// @tfverse else
	mux := http.NewServeMux()
	mux.Handle("/example-items/", http.StripPrefix("/example-items", handler.Routes()))
	return mux
// @tfverse end
}

func send(t *testing.T, app http.Handler, method, path, body string) *httptest.ResponseRecorder {
	t.Helper()
	request := httptest.NewRequest(method, path, strings.NewReader(body))
	request.Header.Set("Content-Type", "application/json")
	recorder := httptest.NewRecorder()
	app.ServeHTTP(recorder, request)
	return recorder
}

func TestPostCreatesAnItem(t *testing.T) {
	app := server()

	created := send(t, app, http.MethodPost, collection, `{"name":"First"}`)
	var item ExampleItem
	if err := json.NewDecoder(created.Body).Decode(&item); err != nil || created.Code != http.StatusCreated {
		t.Fatalf("status = %d, err = %v", created.Code, err)
	}

	if got := send(t, app, http.MethodGet, "/example-items/"+item.ID.String(), ""); got.Code != http.StatusOK {
		t.Fatalf("get status = %d", got.Code)
	}
}

func TestInvalidInputIsAProblem(t *testing.T) {
	response := send(t, server(), http.MethodPost, collection, `{"name":""}`)

	if response.Code != http.StatusUnprocessableEntity {
		t.Fatalf("status = %d", response.Code)
	}
	if got := response.Header().Get("Content-Type"); got != "application/problem+json" {
		t.Fatalf("content type = %q", got)
	}
}

func TestMissingItemIsA404(t *testing.T) {
	if got := send(t, server(), http.MethodGet, "/example-items/"+uuid.NewString(), ""); got.Code != http.StatusNotFound {
		t.Fatalf("status = %d", got.Code)
	}
}
