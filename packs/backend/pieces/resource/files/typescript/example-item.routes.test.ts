// @tfverse if stack.express
import type { AddressInfo } from "node:net";

import express from "express";
// @tfverse else
import { Hono } from "hono";
// @tfverse end
import { describe, expect, it } from "vitest";

import { errorHandler } from "../../shared/error-handler";
import { InMemoryExampleItemRepository } from "./example-item.repository";
import { exampleItemRoutes } from "./example-item.routes";
import { ExampleItemService } from "./example-item.service";

type Send = (path: string, init?: RequestInit) => Promise<Response>;

// @tfverse if stack.express
async function withApi(run: (send: Send) => Promise<void>) {
	const app = express()
		.use(express.json())
		.use("/example-items", exampleItemRoutes(new ExampleItemService(new InMemoryExampleItemRepository())))
		.use(errorHandler);
	const server = app.listen(0);
	try {
		const { port } = server.address() as AddressInfo;
		await run((path, init) => fetch(`http://127.0.0.1:${port}/example-items${path}`, init));
	} finally {
		server.close();
	}
}
// @tfverse else
async function withApi(run: (send: Send) => Promise<void>) {
	const app = new Hono()
		.route("/example-items", exampleItemRoutes(new ExampleItemService(new InMemoryExampleItemRepository())))
		.onError(errorHandler);
	await run((path, init) => Promise.resolve(app.request(`/example-items${path}`, init)));
}
// @tfverse end

const json = { "Content-Type": "application/json" };

describe("example item routes", () => {
	it("creates and reads an example item", async () => {
		await withApi(async (send) => {
			const created = await send("", { method: "POST", headers: json, body: JSON.stringify({ name: "First" }) });
			const item = (await created.json()) as { id: string };

			expect(created.status).toBe(201);
			expect((await send(`/${item.id}`)).status).toBe(200);
		});
	});

	it("answers invalid input with problem details", async () => {
		await withApi(async (send) => {
			const response = await send("", { method: "POST", headers: json, body: JSON.stringify({ name: "" }) });

			expect(response.status).toBe(422);
			expect(response.headers.get("Content-Type")).toContain("application/problem+json");
		});
	});

	it("answers missing example items with 404 and bad ids with 422", async () => {
		await withApi(async (send) => {
			expect((await send("/00000000-0000-4000-8000-000000000000")).status).toBe(404);
			expect((await send("/not-a-uuid")).status).toBe(422);
		});
	});
});
