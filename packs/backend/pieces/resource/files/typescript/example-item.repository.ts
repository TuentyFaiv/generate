import type { CreateExampleItem, ExampleItem, UpdateExampleItem } from "./example-item.model";

/** A page of results. */
export interface Page {
	limit: number;
	offset: number;
}

/** Storage for example items; the service depends on this, not on a database. */
export interface ExampleItemRepository {
	list(page: Page): Promise<ExampleItem[]>;
	findById(id: string): Promise<ExampleItem | undefined>;
	create(input: CreateExampleItem): Promise<ExampleItem>;
	update(id: string, input: UpdateExampleItem): Promise<ExampleItem | undefined>;
	delete(id: string): Promise<boolean>;
}

/** Keeps example items in memory, for tests and prototypes. */
export class InMemoryExampleItemRepository implements ExampleItemRepository {
	readonly #items = new Map<string, ExampleItem>();

	list({ limit, offset }: Page): Promise<ExampleItem[]> {
		return Promise.resolve([...this.#items.values()].slice(offset, offset + limit));
	}

	findById(id: string): Promise<ExampleItem | undefined> {
		return Promise.resolve(this.#items.get(id));
	}

	create(input: CreateExampleItem): Promise<ExampleItem> {
		const now = new Date();
		const item: ExampleItem = { id: crypto.randomUUID(), name: input.name, createdAt: now, updatedAt: now };
		this.#items.set(item.id, item);
		return Promise.resolve(item);
	}

	update(id: string, input: UpdateExampleItem): Promise<ExampleItem | undefined> {
		const current = this.#items.get(id);
		if (!current) {
			return Promise.resolve(undefined);
		}
		const item: ExampleItem = { ...current, name: input.name ?? current.name, updatedAt: new Date() };
		this.#items.set(id, item);
		return Promise.resolve(item);
	}

	delete(id: string): Promise<boolean> {
		return Promise.resolve(this.#items.delete(id));
	}
}
