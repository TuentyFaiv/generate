import { LitElement, css, html } from "lit";
import { customElement, property } from "lit/decorators.js";

export type ExampleAtomTone = "neutral" | "accent";

/** The smallest UI building block: an inline, label-sized element. */
@customElement("ui-example-atom")
export class ExampleAtom extends LitElement {
	static override styles = css`
		:host {
			display: inline-flex;
			align-items: center;
			gap: 0.25rem;
			padding: 0.125rem 0.5rem;
			border-radius: 999px;
			font-size: 0.75rem;
			font-weight: 600;
			line-height: 1.5;
			color: var(--ui-neutral-foreground, #3f3f46);
			background: var(--ui-neutral, #f4f4f5);
		}

		:host([tone="accent"]) {
			color: var(--ui-accent-foreground, #ffffff);
			background: var(--ui-accent, #4f46e5);
		}
	`;

	/** Visual emphasis. */
	@property({ reflect: true }) accessor tone: ExampleAtomTone = "neutral";

	override render() {
		return html`<slot></slot>`;
	}
}

declare global {
	interface HTMLElementTagNameMap {
		"ui-example-atom": ExampleAtom;
	}
}
