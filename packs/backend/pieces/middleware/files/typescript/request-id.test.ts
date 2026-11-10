// @tfverse if stack.express
import type { AddressInfo } from "node:net";

import express from "express";
// @tfverse else
import { Hono } from "hono";
// @tfverse end
import { describe, expect, it } from "vitest";

import { REQUEST_ID_HEADER, requestId } from "./request-id";

// @tfverse if stack.express
async function send(headers: Record<string, string> = {}) {
	const app = express()
		.use(requestId)
		.get("/", (_request, response) => {
			response.send(response.locals.requestId);
		});
	const server = app.listen(0);
	try {
		const { port } = server.address() as AddressInfo;
		return await fetch(`http://127.0.0.1:${port}/`, { headers });
	} finally {
		server.close();
	}
}
// @tfverse else
const app = new Hono().use(requestId).get("/", (c) => c.text(c.get("requestId")));

function send(headers: Record<string, string> = {}) {
	return Promise.resolve(app.request("/", { headers }));
}
// @tfverse end

describe("requestId", () => {
	it("reuses a valid incoming id", async () => {
		const response = await send({ [REQUEST_ID_HEADER]: "abc-123" });

		expect(response.headers.get(REQUEST_ID_HEADER)).toBe("abc-123");
		expect(await response.text()).toBe("abc-123");
	});

	it("replaces a missing or unsafe id with a new one", async () => {
		const cases: Record<string, string>[] = [{}, { [REQUEST_ID_HEADER]: "<script>" }];
		for (const headers of cases) {
			const response = await send(headers);

			expect(response.headers.get(REQUEST_ID_HEADER)).toMatch(/^[0-9a-f-]{36}$/);
		}
	});
});
