import { createDOM } from "@builder.io/qwik/testing";
import { describe, expect, it } from "vitest";

import { ExampleMolecule } from "./example-molecule";

describe("ExampleMolecule", () => {
	it("names the group with its label and description", async () => {
		const { screen, render } = await createDOM();
		await render(
			<ExampleMolecule label="Notifications" description="Choose how we reach you.">
				<button type="button">Email</button>
			</ExampleMolecule>,
		);
		const group = screen.querySelector('[role="group"]');
		const label = screen.querySelector(`[id="${group?.getAttribute("aria-labelledby")}"]`);
		const description = screen.querySelector(`[id="${group?.getAttribute("aria-describedby")}"]`);

		expect(label?.textContent).toBe("Notifications");
		expect(description?.textContent).toBe("Choose how we reach you.");
		expect(group?.querySelector("button")?.textContent).toBe("Email");
	});

	it("leaves out the description when there is none", async () => {
		const { screen, render } = await createDOM();
		await render(<ExampleMolecule label="Notifications" />);

		expect(screen.querySelector('[role="group"]')?.hasAttribute("aria-describedby")).toBe(false);
	});
});
