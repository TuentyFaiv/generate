import { experimental_AstroContainer as AstroContainer } from "astro/container";
import { describe, expect, it } from "vitest";

import ExampleAtom from "./example-atom.astro";

describe("ExampleAtom", () => {
	it("renders its content with the neutral tone by default", async () => {
		const container = await AstroContainer.create();
		const html = await container.renderToString(ExampleAtom, { slots: { default: "New" } });

		expect(html).toContain('data-tone="neutral"');
		expect(html).toContain("New");
	});

	it("keeps native attributes", async () => {
		const container = await AstroContainer.create();
		const html = await container.renderToString(ExampleAtom, {
			props: { tone: "accent", title: "Status" },
			slots: { default: "New" },
		});

		expect(html).toContain('data-tone="accent"');
		expect(html).toContain('title="Status"');
	});
});
