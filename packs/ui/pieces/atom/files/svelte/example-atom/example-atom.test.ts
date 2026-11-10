import { cleanup, render } from "@testing-library/svelte";
import { createRawSnippet } from "svelte";
import { afterEach, describe, expect, it } from "vitest";

import ExampleAtom from "./example-atom.svelte";

const text = (value: string) => createRawSnippet(() => ({ render: () => `<span>${value}</span>` }));

afterEach(cleanup);

describe("ExampleAtom", () => {
	it("renders its content with the neutral tone by default", () => {
		const { container } = render(ExampleAtom, { children: text("New") });
		const atom = container.querySelector("[data-tone]");

		expect(atom?.getAttribute("data-tone")).toBe("neutral");
		expect(atom?.textContent?.trim()).toBe("New");
	});

	it("keeps native attributes and extra classes", () => {
		const { container } = render(ExampleAtom, { tone: "accent", class: "extra", title: "Status", children: text("New") });
		const atom = container.querySelector("[data-tone]");

		expect(atom?.getAttribute("data-tone")).toBe("accent");
		expect(atom?.classList.contains("extra")).toBe(true);
		expect(atom?.getAttribute("title")).toBe("Status");
	});
});
