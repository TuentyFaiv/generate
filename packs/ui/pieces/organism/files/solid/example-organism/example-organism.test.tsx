import { cleanup, render, screen } from "@solidjs/testing-library";
import { afterEach, describe, expect, it } from "vitest";

import { ExampleOrganism } from "./example-organism";

afterEach(cleanup);

describe("ExampleOrganism", () => {
	it("is a region named by its heading", () => {
		render(() => (
			<ExampleOrganism heading="Team" actions={<button type="button">Invite</button>}>
				<p>Three members</p>
			</ExampleOrganism>
		));
		const region = screen.getByRole("region", { name: "Team" });

		expect(region.textContent).toContain("Three members");
		expect(screen.getByRole("button", { name: "Invite" })).toBeTruthy();
	});

	it("renders without actions", () => {
		render(() => <ExampleOrganism heading="Team" />);

		expect(screen.queryByRole("button")).toBeNull();
	});
});
