import { createDOM } from "@builder.io/qwik/testing";
import { describe, expect, it } from "vitest";

import { ExampleOrganism } from "./example-organism";

describe("ExampleOrganism", () => {
	it("is a section named by its title, with actions in their slot", async () => {
		const { screen, render } = await createDOM();
		await render(
			<ExampleOrganism heading="Team">
				<button q:slot="actions" type="button">
					Invite
				</button>
				<p>Three members</p>
			</ExampleOrganism>,
		);
		const section = screen.querySelector("section");
		const heading = screen.querySelector(`[id="${section?.getAttribute("aria-labelledby")}"]`);

		expect(heading?.textContent).toBe("Team");
		expect(section?.querySelector("header button")?.textContent).toBe("Invite");
		expect(section?.textContent).toContain("Three members");
	});
});
