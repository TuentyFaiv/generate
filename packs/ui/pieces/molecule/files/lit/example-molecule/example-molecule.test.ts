import { afterEach, describe, expect, it } from "vitest";

import "./example-molecule";

afterEach(() => {
	document.body.replaceChildren();
});

describe("ExampleMolecule", () => {
	it("names the group with its label and description", async () => {
		const molecule = document.createElement("ui-example-molecule");
		molecule.label = "Notifications";
		molecule.description = "Choose how we reach you.";
		document.body.append(molecule);
		await molecule.updateComplete;
		const group = molecule.shadowRoot?.querySelector('[role="group"]');

		expect(molecule.shadowRoot?.querySelector("#label")?.textContent).toBe("Notifications");
		expect(group?.getAttribute("aria-describedby")).toBe("description");
	});

	it("leaves out the description when there is none", async () => {
		const molecule = document.createElement("ui-example-molecule");
		molecule.label = "Notifications";
		document.body.append(molecule);
		await molecule.updateComplete;

		expect(molecule.shadowRoot?.querySelector('[role="group"]')?.hasAttribute("aria-describedby")).toBe(false);
	});
});
