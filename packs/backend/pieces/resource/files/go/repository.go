package exampleitems

import (
	"cmp"
	"context"
	"errors"
	"maps"
	"slices"
	"sync"

	"github.com/google/uuid"
)

// ErrNotFound reports that no example item has the given id.
var ErrNotFound = errors.New("example item not found")

// Repository stores example items; the service depends on it, not on a database.
type Repository interface {
	List(ctx context.Context, limit, offset int) ([]ExampleItem, error)
	Get(ctx context.Context, id uuid.UUID) (ExampleItem, error)
	Create(ctx context.Context, item ExampleItem) error
	Update(ctx context.Context, item ExampleItem) error
	Delete(ctx context.Context, id uuid.UUID) error
}

// MemoryRepository keeps example items in memory, for tests and prototypes.
type MemoryRepository struct {
	mu    sync.RWMutex
	items map[uuid.UUID]ExampleItem
}

// NewMemoryRepository returns an empty MemoryRepository.
func NewMemoryRepository() *MemoryRepository {
	return &MemoryRepository{items: make(map[uuid.UUID]ExampleItem)}
}

func (r *MemoryRepository) List(_ context.Context, limit, offset int) ([]ExampleItem, error) {
	r.mu.RLock()
	defer r.mu.RUnlock()
	items := slices.SortedFunc(maps.Values(r.items), func(a, b ExampleItem) int {
		return cmp.Compare(a.CreatedAt.UnixNano(), b.CreatedAt.UnixNano())
	})
	start := min(offset, len(items))
	return items[start:min(start+limit, len(items))], nil
}

func (r *MemoryRepository) Get(_ context.Context, id uuid.UUID) (ExampleItem, error) {
	r.mu.RLock()
	defer r.mu.RUnlock()
	item, ok := r.items[id]
	if !ok {
		return ExampleItem{}, ErrNotFound
	}
	return item, nil
}

func (r *MemoryRepository) Create(_ context.Context, item ExampleItem) error {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.items[item.ID] = item
	return nil
}

func (r *MemoryRepository) Update(_ context.Context, item ExampleItem) error {
	r.mu.Lock()
	defer r.mu.Unlock()
	if _, ok := r.items[item.ID]; !ok {
		return ErrNotFound
	}
	r.items[item.ID] = item
	return nil
}

func (r *MemoryRepository) Delete(_ context.Context, id uuid.UUID) error {
	r.mu.Lock()
	defer r.mu.Unlock()
	if _, ok := r.items[id]; !ok {
		return ErrNotFound
	}
	delete(r.items, id)
	return nil
}
