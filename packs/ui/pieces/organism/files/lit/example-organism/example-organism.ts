import { LitElement, css, html } from "lit";
import { customElement, property } from "lit/decorators.js";

/** A distinct, labelled region of the page; put controls in the `actions` slot. */
@customElement("ui-example-organism")
export class ExampleOrganism extends LitElement {
	static override styles = css`
		:host {
			display: block;
		}

		.root {
			display: grid;
			gap: 1rem;
			padding: 1.5rem;
			border: 1px solid var(--ui-border, #e4e4e7);
			border-radius: 0.75rem;
		}

		.header {
			display: flex;
			flex-wrap: wrap;
			align-items: center;
			justify-content: space-between;
			gap: 0.75rem;
		}

		.title {
			margin: 0;
			font-size: 1.125rem;
			font-weight: 600;
		}

		.actions {
			display: flex;
			gap: 0.5rem;
		}
	`;

	/** Heading that names the section. */
	@property() accessor heading = "";

	override render() {
		return html`
			<section class="root" aria-labelledby="heading">
				<header class="header">
					<h2 id="heading" class="title">${this.heading}</h2>
					<div class="actions"><slot name="actions"></slot></div>
				</header>
				<slot></slot>
			</section>
		`;
	}
}

declare global {
	interface HTMLElementTagNameMap {
		"ui-example-organism": ExampleOrganism;
	}
}
