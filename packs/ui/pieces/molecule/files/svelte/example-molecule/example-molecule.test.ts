import { cleanup, render, screen } from "@testing-library/svelte";
import { createRawSnippet } from "svelte";
import { afterEach, describe, expect, it } from "vitest";

import ExampleMolecule from "./example-molecule.svelte";

const button = createRawSnippet(() => ({ render: () => `<button type="button">Email</button>` }));

afterEach(cleanup);

describe("ExampleMolecule", () => {
	it("names the group with its label and description", () => {
		render(ExampleMolecule, { label: "Notifications", description: "Choose how we reach you.", children: button });
		const group = screen.getByRole("group", { name: "Notifications" });

		expect(group.getAttribute("aria-describedby")).toBe(screen.getByText("Choose how we reach you.").id);
		expect(screen.getByRole("button", { name: "Email" })).toBeTruthy();
	});

	it("leaves out the description when there is none", () => {
		render(ExampleMolecule, { label: "Notifications" });

		expect(screen.getByRole("group", { name: "Notifications" }).hasAttribute("aria-describedby")).toBe(false);
	});
});
