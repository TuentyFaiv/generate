import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import ExampleOrganism from "./example-organism.vue";

describe("ExampleOrganism", () => {
	it("is a section named by its heading", () => {
		const wrapper = mount(ExampleOrganism, {
			props: { heading: "Team" },
			slots: { default: "<p>Three members</p>", actions: '<button type="button">Invite</button>' },
		});

		expect(wrapper.get(`[id="${wrapper.attributes("aria-labelledby")}"]`).text()).toBe("Team");
		expect(wrapper.get("header button").text()).toBe("Invite");
		expect(wrapper.text()).toContain("Three members");
	});

	it("renders without actions", () => {
		const wrapper = mount(ExampleOrganism, { props: { heading: "Team" } });

		expect(wrapper.find("button").exists()).toBe(false);
	});
});
