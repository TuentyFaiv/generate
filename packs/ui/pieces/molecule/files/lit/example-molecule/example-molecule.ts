import { LitElement, css, html, nothing } from "lit";
import { customElement, property } from "lit/decorators.js";

/** Groups atoms under one accessible label, such as a field with its hint. */
@customElement("ui-example-molecule")
export class ExampleMolecule extends LitElement {
	static override styles = css`
		:host {
			display: block;
		}

		.root {
			display: grid;
			gap: 0.5rem;
		}

		.label {
			font-size: 0.875rem;
			font-weight: 600;
		}

		.description {
			margin: 0;
			font-size: 0.875rem;
			color: var(--ui-muted-foreground, #71717a);
		}

		.content {
			display: flex;
			flex-wrap: wrap;
			align-items: center;
			gap: 0.5rem;
		}
	`;

	/** Names the group for assistive technology. */
	@property() accessor label = "";

	/** Extra guidance shown under the label. */
	@property() accessor description = "";

	override render() {
		return html`
			<div class="root" role="group" aria-labelledby="label" aria-describedby=${this.description ? "description" : nothing}>
				<span id="label" class="label">${this.label}</span>
				${this.description ? html`<p id="description" class="description">${this.description}</p>` : nothing}
				<div class="content"><slot></slot></div>
			</div>
		`;
	}
}

declare global {
	interface HTMLElementTagNameMap {
		"ui-example-molecule": ExampleMolecule;
	}
}
