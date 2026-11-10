// @tfverse if stack.hono
import { Hono } from "hono";
// @tfverse else
import { Router } from "express";
// @tfverse end
import { z } from "zod";

import { validate } from "../../shared/errors";
import { createExampleItemSchema, updateExampleItemSchema } from "./example-item.model";
import type { ExampleItemService } from "./example-item.service";

const idSchema = z.uuid();

const pageSchema = z.object({
	limit: z.coerce.number().int().min(1).max(100).default(20),
	offset: z.coerce.number().int().min(0).default(0),
});

// @tfverse if stack.hono
/** HTTP routes for example items; mount them with `app.route("/example-items", exampleItemRoutes(service))`. */
export function exampleItemRoutes(service: ExampleItemService) {
	return new Hono()
		.get("/", async (c) => c.json(await service.list(validate(pageSchema, c.req.query()))))
		.get("/:id", async (c) => c.json(await service.get(validate(idSchema, c.req.param("id")))))
		.post("/", async (c) => {
			const input = validate(createExampleItemSchema, await c.req.json());
			return c.json(await service.create(input), 201);
		})
		.patch("/:id", async (c) => {
			const id = validate(idSchema, c.req.param("id"));
			const input = validate(updateExampleItemSchema, await c.req.json());
			return c.json(await service.update(id, input));
		})
		.delete("/:id", async (c) => {
			await service.delete(validate(idSchema, c.req.param("id")));
			return c.body(null, 204);
		});
}
// @tfverse else
/** HTTP routes for example items; mount them after `express.json()` with `app.use("/example-items", …)`. */
export function exampleItemRoutes(service: ExampleItemService): Router {
	return Router()
		.get("/", async (request, response) => {
			response.json(await service.list(validate(pageSchema, request.query)));
		})
		.get("/:id", async (request, response) => {
			response.json(await service.get(validate(idSchema, request.params.id)));
		})
		.post("/", async (request, response) => {
			response.status(201).json(await service.create(validate(createExampleItemSchema, request.body)));
		})
		.patch("/:id", async (request, response) => {
			const id = validate(idSchema, request.params.id);
			response.json(await service.update(id, validate(updateExampleItemSchema, request.body)));
		})
		.delete("/:id", async (request, response) => {
			await service.delete(validate(idSchema, request.params.id));
			response.status(204).end();
		});
}
// @tfverse end
