import { pgTable, text, timestamp, uuid } from "drizzle-orm/pg-core";

/** The `example_items` table. */
export const exampleItems = pgTable("example_items", {
	id: uuid("id").primaryKey().defaultRandom(),
	name: text("name").notNull(),
	createdAt: timestamp("created_at", { withTimezone: true }).notNull().defaultNow(),
	updatedAt: timestamp("updated_at", { withTimezone: true })
		.notNull()
		.defaultNow()
		.$onUpdate(() => new Date()),
});
