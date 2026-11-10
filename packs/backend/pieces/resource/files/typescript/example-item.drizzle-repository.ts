import { asc, eq } from "drizzle-orm";
import type { PgDatabase, PgQueryResultHKT } from "drizzle-orm/pg-core";

import type { CreateExampleItem, ExampleItem, UpdateExampleItem } from "./example-item.model";
import type { ExampleItemRepository, Page } from "./example-item.repository";
import { exampleItems } from "./example-item.table";

/** Stores example items in PostgreSQL with Drizzle. */
export class DrizzleExampleItemRepository implements ExampleItemRepository {
	readonly #db: PgDatabase<PgQueryResultHKT>;

	constructor(db: PgDatabase<PgQueryResultHKT>) {
		this.#db = db;
	}

	list({ limit, offset }: Page): Promise<ExampleItem[]> {
		return this.#db.select().from(exampleItems).orderBy(asc(exampleItems.createdAt)).limit(limit).offset(offset);
	}

	async findById(id: string): Promise<ExampleItem | undefined> {
		const [item] = await this.#db.select().from(exampleItems).where(eq(exampleItems.id, id));
		return item;
	}

	async create(input: CreateExampleItem): Promise<ExampleItem> {
		const [item] = await this.#db.insert(exampleItems).values(input).returning();
		if (!item) {
			throw new Error("Insert returned no row.");
		}
		return item;
	}

	async update(id: string, input: UpdateExampleItem): Promise<ExampleItem | undefined> {
		const [item] = await this.#db
			.update(exampleItems)
			.set({ ...input, updatedAt: new Date() })
			.where(eq(exampleItems.id, id))
			.returning();
		return item;
	}

	async delete(id: string): Promise<boolean> {
		const deleted = await this.#db
			.delete(exampleItems)
			.where(eq(exampleItems.id, id))
			.returning({ id: exampleItems.id });
		return deleted.length > 0;
	}
}
