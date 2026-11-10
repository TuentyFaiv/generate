import { ChangeDetectionStrategy, Component, input } from "@angular/core";

let nextId = 0;

/** A distinct, labelled region of the page; project controls with an `actions` attribute. */
@Component({
	selector: "ui-example-organism",
	templateUrl: "./example-organism.html",
	styleUrl: "./example-organism.css", // @tfverse if !stack.tailwind
	changeDetection: ChangeDetectionStrategy.OnPush,
	host: {
		role: "region",
		class: "grid gap-4 rounded-xl border border-zinc-200 p-6", // @tfverse if stack.tailwind
		"[attr.aria-labelledby]": "headingId",
	},
})
export class ExampleOrganism {
	/** Heading that names the section. */
	readonly heading = input.required<string>();

	protected readonly headingId = `ui-example-organism-${nextId++}`;
}
