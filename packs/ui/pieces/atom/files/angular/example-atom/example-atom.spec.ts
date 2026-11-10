import { Component } from "@angular/core";
import { TestBed } from "@angular/core/testing";

import { ExampleAtom } from "./example-atom";

@Component({
	imports: [ExampleAtom],
	template: `<ui-example-atom tone="accent">New</ui-example-atom>`,
})
class Host {}

describe("ExampleAtom", () => {
	it("projects its content and reflects the tone", async () => {
		const fixture = TestBed.createComponent(Host);
		fixture.detectChanges();
		await fixture.whenStable();
		const atom: HTMLElement = fixture.nativeElement.querySelector("ui-example-atom");

		expect(atom.textContent).toBe("New");
		expect(atom.getAttribute("data-tone")).toBe("accent");
	});
});
