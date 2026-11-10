import { ChangeDetectionStrategy, Component, input } from "@angular/core";

let nextId = 0;

/** Groups atoms under one accessible label, such as a field with its hint. */
@Component({
	selector: "ui-example-molecule",
	templateUrl: "./example-molecule.html",
	styleUrl: "./example-molecule.css", // @tfverse if !stack.tailwind
	changeDetection: ChangeDetectionStrategy.OnPush,
	host: {
		role: "group",
		class: "grid gap-2", // @tfverse if stack.tailwind
		"[attr.aria-labelledby]": "labelId",
		"[attr.aria-describedby]": "description() ? descriptionId : null",
	},
})
export class ExampleMolecule {
	/** Names the group for assistive technology. */
	readonly label = input.required<string>();
	/** Extra guidance shown under the label. */
	readonly description = input<string>();

	private readonly id = `ui-example-molecule-${nextId++}`;
	protected readonly labelId = `${this.id}-label`;
	protected readonly descriptionId = `${this.id}-description`;
}
