import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import ExampleMolecule from "./example-molecule.vue";

describe("ExampleMolecule", () => {
	it("names the group with its label and description", () => {
		const wrapper = mount(ExampleMolecule, {
			props: { label: "Notifications", description: "Choose how we reach you." },
			slots: { default: '<button type="button">Email</button>' },
		});
		const group = wrapper.get('[role="group"]');

		expect(wrapper.get(`[id="${group.attributes("aria-labelledby")}"]`).text()).toBe("Notifications");
		expect(wrapper.get(`[id="${group.attributes("aria-describedby")}"]`).text()).toBe("Choose how we reach you.");
		expect(wrapper.get("button").text()).toBe("Email");
	});

	it("leaves out the description when there is none", () => {
		const wrapper = mount(ExampleMolecule, { props: { label: "Notifications" } });

		expect(wrapper.get('[role="group"]').attributes("aria-describedby")).toBeUndefined();
	});
});
