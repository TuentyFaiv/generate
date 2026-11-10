import { experimental_AstroContainer as AstroContainer } from "astro/container";
import { describe, expect, it } from "vitest";

import ExampleMolecule from "./example-molecule.astro";

describe("ExampleMolecule", () => {
	it("links the group to its label and description", async () => {
		const container = await AstroContainer.create();
		const html = await container.renderToString(ExampleMolecule, {
			props: { label: "Notifications", description: "Choose how we reach you." },
			slots: { default: '<button type="button">Email</button>' },
		});
		const labelId = html.match(/aria-labelledby="([^"]+)"/)?.[1];
		const descriptionId = html.match(/aria-describedby="([^"]+)"/)?.[1];

		expect(html).toContain(`id="${labelId}"`);
		expect(html).toContain(`id="${descriptionId}"`);
		expect(html).toContain("Email");
	});

	it("leaves out the description when there is none", async () => {
		const container = await AstroContainer.create();
		const html = await container.renderToString(ExampleMolecule, { props: { label: "Notifications" } });

		expect(html).not.toContain("aria-describedby");
	});
});
