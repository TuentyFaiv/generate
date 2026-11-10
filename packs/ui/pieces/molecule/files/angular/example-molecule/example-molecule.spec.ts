import { Component } from "@angular/core";
import { TestBed } from "@angular/core/testing";

import { ExampleMolecule } from "./example-molecule";

@Component({
	imports: [ExampleMolecule],
	template: `
		<ui-example-molecule label="Notifications" description="Choose how we reach you.">
			<button type="button">Email</button>
		</ui-example-molecule>
		<ui-example-molecule label="Privacy" />
	`,
})
class Host {}

describe("ExampleMolecule", () => {
	it("links the group to its label and description", async () => {
		const fixture = TestBed.createComponent(Host);
		fixture.detectChanges();
		await fixture.whenStable();
		const [group, plain]: HTMLElement[] = Array.from(fixture.nativeElement.querySelectorAll('[role="group"]'));
		const label = group.querySelector(`[id="${group.getAttribute("aria-labelledby")}"]`);
		const description = group.querySelector(`[id="${group.getAttribute("aria-describedby")}"]`);

		expect(label?.textContent).toBe("Notifications");
		expect(description?.textContent).toBe("Choose how we reach you.");
		expect(group.querySelector("button")?.textContent).toBe("Email");
		expect(plain.hasAttribute("aria-describedby")).toBe(false);
	});
});
