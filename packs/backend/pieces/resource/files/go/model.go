// Package exampleitems manages example items: storage, business rules and HTTP handlers.
package exampleitems

import (
	"time"

	"github.com/google/uuid"
)

// ExampleItem is an example item as stored and returned by the API.
type ExampleItem struct {
	ID        uuid.UUID `json:"id"`
	Name      string    `json:"name"`
	CreatedAt time.Time `json:"createdAt"`
	UpdatedAt time.Time `json:"updatedAt"`
}

// NewExampleItem holds the fields accepted when creating an example item.
type NewExampleItem struct {
	Name string `json:"name"`
}

// ExampleItemChanges holds the fields accepted when changing an example item; nil fields stay as they are.
type ExampleItemChanges struct {
	Name *string `json:"name"`
}
