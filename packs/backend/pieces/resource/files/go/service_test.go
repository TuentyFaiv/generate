package exampleitems

import (
	"context"
	"errors"
	"testing"

	"github.com/google/uuid"

	"example.com/app/internal/apperror"
)

func status(err error) int {
	var appErr *apperror.Error
	if errors.As(err, &appErr) {
		return appErr.Status
	}
	return 0
}

func TestCreateTrimsAndStoresTheItem(t *testing.T) {
	service := NewService(NewMemoryRepository())

	item, err := service.Create(context.Background(), NewExampleItem{Name: "  First  "})
	if err != nil {
		t.Fatal(err)
	}
	stored, err := service.Get(context.Background(), item.ID)

	if err != nil || stored.Name != "First" {
		t.Fatalf("stored = %+v, err = %v", stored, err)
	}
}

func TestCreateRejectsAnEmptyName(t *testing.T) {
	_, err := NewService(NewMemoryRepository()).Create(context.Background(), NewExampleItem{Name: " "})

	if status(err) != 422 {
		t.Fatalf("err = %v", err)
	}
}

func TestUpdateKeepsFieldsThatAreNotSent(t *testing.T) {
	service := NewService(NewMemoryRepository())
	item, _ := service.Create(context.Background(), NewExampleItem{Name: "First"})

	updated, err := service.Update(context.Background(), item.ID, ExampleItemChanges{})

	if err != nil || updated.Name != "First" {
		t.Fatalf("updated = %+v, err = %v", updated, err)
	}
}

func TestDeleteReportsAMissingItem(t *testing.T) {
	err := NewService(NewMemoryRepository()).Delete(context.Background(), uuid.New())

	if status(err) != 404 {
		t.Fatalf("err = %v", err)
	}
}
