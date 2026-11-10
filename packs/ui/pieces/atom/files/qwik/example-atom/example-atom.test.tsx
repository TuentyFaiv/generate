import { createDOM } from "@builder.io/qwik/testing";
import { describe, expect, it } from "vitest";

import { ExampleAtom } from "./example-atom";

describe("ExampleAtom", () => {
	it("renders its content with the neutral tone by default", async () => {
		const { screen, render } = await createDOM();
		await render(<ExampleAtom>New</ExampleAtom>);
		const atom = screen.querySelector("span");

		expect(atom?.getAttribute("data-tone")).toBe("neutral");
		expect(atom?.textContent).toBe("New");
	});

	it("keeps native attributes", async () => {
		const { screen, render } = await createDOM();
		await render(
			<ExampleAtom tone="accent" title="Status">
				New
			</ExampleAtom>,
		);
		const atom = screen.querySelector("span");

		expect(atom?.getAttribute("data-tone")).toBe("accent");
		expect(atom?.getAttribute("title")).toBe("Status");
	});
});
