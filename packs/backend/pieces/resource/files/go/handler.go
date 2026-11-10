package exampleitems

import (
	"encoding/json"
	"net/http"
	"strconv"

// @tfverse if stack.echo
	"github.com/google/uuid"
	"github.com/labstack/echo/v4"
// @tfverse else
	"github.com/go-chi/chi/v5"
	"github.com/google/uuid"
// @tfverse end

	"example.com/app/internal/apperror"
)

const (
	defaultLimit = 20
	maxLimit     = 100
	maxBodyBytes = 1 << 20
)

// Handler serves example items over HTTP.
type Handler struct {
	service *Service
}

// NewHandler returns a handler for service.
func NewHandler(service *Service) *Handler {
	return &Handler{service: service}
}
// @tfverse if stack.echo

// Register adds the routes to g, e.g. app.Group("/example-items"); set apperror.Handler as the HTTPErrorHandler.
func (h *Handler) Register(g *echo.Group) {
	g.GET("", h.list)
	g.POST("", h.create)
	g.GET("/:id", h.get)
	g.PATCH("/:id", h.update)
	g.DELETE("/:id", h.delete)
}

func (h *Handler) list(c echo.Context) error {
	limit, offset, err := page(c.QueryParam("limit"), c.QueryParam("offset"))
	if err != nil {
		return err
	}
	items, err := h.service.List(c.Request().Context(), limit, offset)
	if err != nil {
		return err
	}
	return c.JSON(http.StatusOK, items)
}

func (h *Handler) get(c echo.Context) error {
	id, err := parseID(c.Param("id"))
	if err != nil {
		return err
	}
	item, err := h.service.Get(c.Request().Context(), id)
	if err != nil {
		return err
	}
	return c.JSON(http.StatusOK, item)
}

func (h *Handler) create(c echo.Context) error {
	var input NewExampleItem
	if err := decode(c.Request(), &input); err != nil {
		return err
	}
	item, err := h.service.Create(c.Request().Context(), input)
	if err != nil {
		return err
	}
	return c.JSON(http.StatusCreated, item)
}

func (h *Handler) update(c echo.Context) error {
	id, err := parseID(c.Param("id"))
	if err != nil {
		return err
	}
	var changes ExampleItemChanges
	if err := decode(c.Request(), &changes); err != nil {
		return err
	}
	item, err := h.service.Update(c.Request().Context(), id, changes)
	if err != nil {
		return err
	}
	return c.JSON(http.StatusOK, item)
}

func (h *Handler) delete(c echo.Context) error {
	id, err := parseID(c.Param("id"))
	if err != nil {
		return err
	}
	if err := h.service.Delete(c.Request().Context(), id); err != nil {
		return err
	}
	return c.NoContent(http.StatusNoContent)
}
// @tfverse else

// Routes returns the routes; mount them with r.Mount("/example-items", h.Routes()).
func (h *Handler) Routes() http.Handler {
	r := chi.NewRouter()
	r.Get("/", h.list)
	r.Post("/", h.create)
	r.Get("/{id}", h.get)
	r.Patch("/{id}", h.update)
	r.Delete("/{id}", h.delete)
	return r
}

func (h *Handler) list(w http.ResponseWriter, r *http.Request) {
	limit, offset, err := page(r.URL.Query().Get("limit"), r.URL.Query().Get("offset"))
	if err != nil {
		apperror.Write(w, r, err)
		return
	}
	items, err := h.service.List(r.Context(), limit, offset)
	respond(w, r, http.StatusOK, items, err)
}

func (h *Handler) get(w http.ResponseWriter, r *http.Request) {
	id, err := parseID(chi.URLParam(r, "id"))
	if err != nil {
		apperror.Write(w, r, err)
		return
	}
	item, err := h.service.Get(r.Context(), id)
	respond(w, r, http.StatusOK, item, err)
}

func (h *Handler) create(w http.ResponseWriter, r *http.Request) {
	var input NewExampleItem
	if err := decode(r, &input); err != nil {
		apperror.Write(w, r, err)
		return
	}
	item, err := h.service.Create(r.Context(), input)
	respond(w, r, http.StatusCreated, item, err)
}

func (h *Handler) update(w http.ResponseWriter, r *http.Request) {
	id, err := parseID(chi.URLParam(r, "id"))
	if err != nil {
		apperror.Write(w, r, err)
		return
	}
	var changes ExampleItemChanges
	if err := decode(r, &changes); err != nil {
		apperror.Write(w, r, err)
		return
	}
	item, err := h.service.Update(r.Context(), id, changes)
	respond(w, r, http.StatusOK, item, err)
}

func (h *Handler) delete(w http.ResponseWriter, r *http.Request) {
	id, err := parseID(chi.URLParam(r, "id"))
	if err == nil {
		err = h.service.Delete(r.Context(), id)
	}
	if err != nil {
		apperror.Write(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func respond(w http.ResponseWriter, r *http.Request, status int, body any, err error) {
	if err != nil {
		apperror.Write(w, r, err)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(body)
}
// @tfverse end

func page(limitParam, offsetParam string) (limit, offset int, err error) {
	limit, offset = defaultLimit, 0
	if limitParam != "" {
		if limit, err = strconv.Atoi(limitParam); err != nil || limit < 1 || limit > maxLimit {
			return 0, 0, apperror.Invalid("limit must be a number from 1 to 100")
		}
	}
	if offsetParam != "" {
		if offset, err = strconv.Atoi(offsetParam); err != nil || offset < 0 {
			return 0, 0, apperror.Invalid("offset must be a number of 0 or more")
		}
	}
	return limit, offset, nil
}

func parseID(raw string) (uuid.UUID, error) {
	id, err := uuid.Parse(raw)
	if err != nil {
		return uuid.Nil, apperror.Invalid("id must be a UUID")
	}
	return id, nil
}

func decode(r *http.Request, target any) error {
	decoder := json.NewDecoder(http.MaxBytesReader(nil, r.Body, maxBodyBytes))
	decoder.DisallowUnknownFields()
	if err := decoder.Decode(target); err != nil {
		return apperror.BadRequest("the body must be a valid JSON object with known fields", err)
	}
	return nil
}
