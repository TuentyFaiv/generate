import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import ExampleAtom from "./example-atom.vue";

describe("ExampleAtom", () => {
	it("renders its content with the neutral tone by default", () => {
		const wrapper = mount(ExampleAtom, { slots: { default: "New" } });

		expect(wrapper.attributes("data-tone")).toBe("neutral");
		expect(wrapper.text()).toBe("New");
	});

	it("keeps native attributes and extra classes", () => {
		const wrapper = mount(ExampleAtom, {
			props: { tone: "accent" },
			attrs: { class: "extra", title: "Status" },
			slots: { default: "New" },
		});

		expect(wrapper.attributes("data-tone")).toBe("accent");
		expect(wrapper.classes()).toContain("extra");
		expect(wrapper.attributes("title")).toBe("Status");
	});
});
