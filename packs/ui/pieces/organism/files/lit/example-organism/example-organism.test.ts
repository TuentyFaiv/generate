import { afterEach, describe, expect, it } from "vitest";

import "./example-organism";

afterEach(() => {
	document.body.replaceChildren();
});

describe("ExampleOrganism", () => {
	it("renders its heading and an actions slot", async () => {
		const organism = document.createElement("ui-example-organism");
		organism.heading = "Team";
		document.body.append(organism);
		await organism.updateComplete;

		expect(organism.shadowRoot?.querySelector("h2")?.textContent).toBe("Team");
		expect(organism.shadowRoot?.querySelector('slot[name="actions"]')).not.toBeNull();
	});
});
