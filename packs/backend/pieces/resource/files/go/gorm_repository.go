package exampleitems

import (
	"context"
	"errors"

	"github.com/google/uuid"
	"gorm.io/gorm"
)

// GormRepository stores example items in the `example_items` table with GORM.
type GormRepository struct {
	db *gorm.DB
}

// NewGormRepository returns a repository using db.
func NewGormRepository(db *gorm.DB) *GormRepository {
	return &GormRepository{db: db}
}

func (r *GormRepository) List(ctx context.Context, limit, offset int) ([]ExampleItem, error) {
	var items []ExampleItem
	err := r.db.WithContext(ctx).Order("created_at").Limit(limit).Offset(offset).Find(&items).Error
	return items, err
}

func (r *GormRepository) Get(ctx context.Context, id uuid.UUID) (ExampleItem, error) {
	var item ExampleItem
	err := r.db.WithContext(ctx).First(&item, "id = ?", id).Error
	if errors.Is(err, gorm.ErrRecordNotFound) {
		return ExampleItem{}, ErrNotFound
	}
	return item, err
}

func (r *GormRepository) Create(ctx context.Context, item ExampleItem) error {
	return r.db.WithContext(ctx).Create(&item).Error
}

func (r *GormRepository) Update(ctx context.Context, item ExampleItem) error {
	result := r.db.WithContext(ctx).
		Model(&ExampleItem{ID: item.ID}).
		Updates(map[string]any{"name": item.Name, "updated_at": item.UpdatedAt})
	return rowsAffected(result)
}

func (r *GormRepository) Delete(ctx context.Context, id uuid.UUID) error {
	return rowsAffected(r.db.WithContext(ctx).Delete(&ExampleItem{}, "id = ?", id))
}

func rowsAffected(result *gorm.DB) error {
	if result.Error != nil {
		return result.Error
	}
	if result.RowsAffected == 0 {
		return ErrNotFound
	}
	return nil
}
