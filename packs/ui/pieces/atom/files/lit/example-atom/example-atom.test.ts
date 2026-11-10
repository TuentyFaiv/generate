import { afterEach, describe, expect, it } from "vitest";

import "./example-atom";

afterEach(() => {
	document.body.replaceChildren();
});

describe("ExampleAtom", () => {
	it("reflects the tone and projects its content", async () => {
		const atom = document.createElement("ui-example-atom");
		atom.tone = "accent";
		atom.textContent = "New";
		document.body.append(atom);
		await atom.updateComplete;

		expect(atom.getAttribute("tone")).toBe("accent");
		expect(atom.shadowRoot?.querySelector("slot")).not.toBeNull();
	});
});
