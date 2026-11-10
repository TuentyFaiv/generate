import { cleanup, render, screen } from "@testing-library/svelte";
import { createRawSnippet } from "svelte";
import { afterEach, describe, expect, it } from "vitest";

import ExampleOrganism from "./example-organism.svelte";

const invite = createRawSnippet(() => ({ render: () => `<button type="button">Invite</button>` }));
const members = createRawSnippet(() => ({ render: () => "<p>Three members</p>" }));

afterEach(cleanup);

describe("ExampleOrganism", () => {
	it("is a region named by its heading", () => {
		render(ExampleOrganism, { heading: "Team", actions: invite, children: members });
		const region = screen.getByRole("region", { name: "Team" });

		expect(region.textContent).toContain("Three members");
		expect(screen.getByRole("button", { name: "Invite" })).toBeTruthy();
	});

	it("renders without actions", () => {
		render(ExampleOrganism, { heading: "Team" });

		expect(screen.queryByRole("button")).toBeNull();
	});
});
