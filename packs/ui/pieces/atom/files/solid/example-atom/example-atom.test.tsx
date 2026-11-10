import { cleanup, render, screen } from "@solidjs/testing-library";
import { afterEach, describe, expect, it } from "vitest";

import { ExampleAtom } from "./example-atom";

afterEach(cleanup);

describe("ExampleAtom", () => {
	it("renders its content with the neutral tone by default", () => {
		render(() => <ExampleAtom>New</ExampleAtom>);

		expect(screen.getByText("New").dataset.tone).toBe("neutral");
	});

	it("keeps native attributes and extra classes", () => {
		render(() => (
			<ExampleAtom tone="accent" class="extra" title="Status">
				New
			</ExampleAtom>
		));
		const atom = screen.getByText("New");

		expect(atom.dataset.tone).toBe("accent");
		expect(atom.classList.contains("extra")).toBe(true);
		expect(atom.getAttribute("title")).toBe("Status");
	});
});
