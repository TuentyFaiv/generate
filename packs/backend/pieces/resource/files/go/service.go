package exampleitems

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"time"
	"unicode/utf8"

	"github.com/google/uuid"

	"example.com/app/internal/apperror"
)

const maxNameLen = 120

// Service holds the business rules for example items.
type Service struct {
	repository Repository
	now        func() time.Time
}

// NewService returns a service storing example items in repository.
func NewService(repository Repository) *Service {
	return &Service{repository: repository, now: time.Now}
}

// List returns a page of example items, oldest first.
func (s *Service) List(ctx context.Context, limit, offset int) ([]ExampleItem, error) {
	return s.repository.List(ctx, limit, offset)
}

// Get returns the example item with id.
func (s *Service) Get(ctx context.Context, id uuid.UUID) (ExampleItem, error) {
	item, err := s.repository.Get(ctx, id)
	return item, notFound(err, id)
}

// Create validates and stores a new example item.
func (s *Service) Create(ctx context.Context, input NewExampleItem) (ExampleItem, error) {
	name, err := validName(input.Name)
	if err != nil {
		return ExampleItem{}, err
	}
	id, err := uuid.NewV7()
	if err != nil {
		return ExampleItem{}, err
	}
	now := s.now().UTC()
	item := ExampleItem{ID: id, Name: name, CreatedAt: now, UpdatedAt: now}
	return item, s.repository.Create(ctx, item)
}

// Update applies changes to the example item with id.
func (s *Service) Update(ctx context.Context, id uuid.UUID, changes ExampleItemChanges) (ExampleItem, error) {
	item, err := s.Get(ctx, id)
	if err != nil {
		return ExampleItem{}, err
	}
	if changes.Name != nil {
		if item.Name, err = validName(*changes.Name); err != nil {
			return ExampleItem{}, err
		}
	}
	item.UpdatedAt = s.now().UTC()
	return item, notFound(s.repository.Update(ctx, item), id)
}

// Delete deletes the example item with id.
func (s *Service) Delete(ctx context.Context, id uuid.UUID) error {
	return notFound(s.repository.Delete(ctx, id), id)
}

func validName(name string) (string, error) {
	name = strings.TrimSpace(name)
	if length := utf8.RuneCountInString(name); length == 0 || length > maxNameLen {
		return "", apperror.Invalid(fmt.Sprintf("name must be 1 to %d characters", maxNameLen))
	}
	return name, nil
}

func notFound(err error, id uuid.UUID) error {
	if errors.Is(err, ErrNotFound) {
		return apperror.NotFound(fmt.Sprintf("No example item with id %s.", id))
	}
	return err
}
