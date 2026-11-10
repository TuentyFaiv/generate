import { Component } from "@angular/core";
import { TestBed } from "@angular/core/testing";

import { ExampleOrganism } from "./example-organism";

@Component({
	imports: [ExampleOrganism],
	template: `
		<ui-example-organism heading="Team">
			<button actions type="button">Invite</button>
			<p>Three members</p>
		</ui-example-organism>
	`,
})
class Host {}

describe("ExampleOrganism", () => {
	it("is a region named by its heading, with projected actions", async () => {
		const fixture = TestBed.createComponent(Host);
		fixture.detectChanges();
		await fixture.whenStable();
		const region: HTMLElement = fixture.nativeElement.querySelector('[role="region"]');
		const heading = region.querySelector(`[id="${region.getAttribute("aria-labelledby")}"]`);

		expect(heading?.textContent).toBe("Team");
		expect(region.querySelector("header button")?.textContent).toBe("Invite");
		expect(region.textContent).toContain("Three members");
	});
});
