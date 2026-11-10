import { z } from "zod";

/** An example item as stored and returned by the API. */
export const exampleItemSchema = z.object({
	id: z.uuid(),
	name: z.string().trim().min(1).max(120),
	createdAt: z.date(),
	updatedAt: z.date(),
});

/** Fields accepted when creating an example item. */
export const createExampleItemSchema = exampleItemSchema.pick({ name: true }).strict();

/** Fields accepted when changing an example item; missing ones stay as they are. */
export const updateExampleItemSchema = createExampleItemSchema.partial();

export type ExampleItem = z.infer<typeof exampleItemSchema>;
export type CreateExampleItem = z.infer<typeof createExampleItemSchema>;
export type UpdateExampleItem = z.infer<typeof updateExampleItemSchema>;
