import { ChangeDetectionStrategy, Component, input } from "@angular/core";

export type ExampleAtomTone = "neutral" | "accent";

/** The smallest UI building block: an inline, label-sized element. */
@Component({
	selector: "ui-example-atom",
	template: "<ng-content />",
	styleUrl: "./example-atom.css", // @tfverse if !stack.tailwind
	changeDetection: ChangeDetectionStrategy.OnPush,
	host: {
		class: "inline-flex items-center gap-1 rounded-full bg-zinc-100 px-2 py-0.5 text-xs font-semibold text-zinc-700 data-[tone=accent]:bg-indigo-600 data-[tone=accent]:text-white", // @tfverse if stack.tailwind
		"[attr.data-tone]": "tone()",
	},
})
export class ExampleAtom {
	/** Visual emphasis. */
	readonly tone = input<ExampleAtomTone>("neutral");
}
