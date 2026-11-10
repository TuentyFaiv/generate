import { describe, expect, it } from "vitest";

import { NotFoundError } from "../../shared/errors";
import { InMemoryExampleItemRepository } from "./example-item.repository";
import { ExampleItemService } from "./example-item.service";

const missingId = "00000000-0000-4000-8000-000000000000";

function service() {
	return new ExampleItemService(new InMemoryExampleItemRepository());
}

describe("ExampleItemService", () => {
	it("creates, reads, updates and deletes an example item", async () => {
		const items = service();
		const created = await items.create({ name: "First" });

		expect(await items.get(created.id)).toEqual(created);
		expect((await items.update(created.id, { name: "Renamed" })).name).toBe("Renamed");
		await items.delete(created.id);
		await expect(items.get(created.id)).rejects.toBeInstanceOf(NotFoundError);
	});

	it("pages through example items", async () => {
		const items = service();
		for (const name of ["a", "b", "c"]) {
			await items.create({ name });
		}

		expect((await items.list({ limit: 2, offset: 1 })).map((item) => item.name)).toEqual(["b", "c"]);
	});

	it("reports missing example items", async () => {
		const items = service();

		await expect(items.update(missingId, { name: "x" })).rejects.toBeInstanceOf(NotFoundError);
		await expect(items.delete(missingId)).rejects.toBeInstanceOf(NotFoundError);
	});
});
