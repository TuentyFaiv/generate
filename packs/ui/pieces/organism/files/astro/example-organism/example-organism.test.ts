import { experimental_AstroContainer as AstroContainer } from "astro/container";
import { describe, expect, it } from "vitest";

import ExampleOrganism from "./example-organism.astro";

describe("ExampleOrganism", () => {
	it("is a section named by its heading, with actions when given", async () => {
		const container = await AstroContainer.create();
		const html = await container.renderToString(ExampleOrganism, {
			props: { heading: "Team" },
			slots: { default: "<p>Three members</p>", actions: '<button type="button">Invite</button>' },
		});
		const headingId = html.match(/aria-labelledby="([^"]+)"/)?.[1];

		expect(html).toContain(`id="${headingId}"`);
		expect(html).toContain("Invite");
		expect(html).toContain("Three members");
	});

	it("renders without actions", async () => {
		const container = await AstroContainer.create();
		const html = await container.renderToString(ExampleOrganism, { props: { heading: "Team" } });

		expect(html).not.toContain("<button");
	});
});
