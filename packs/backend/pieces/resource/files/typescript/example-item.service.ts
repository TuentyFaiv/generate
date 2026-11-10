import { NotFoundError } from "../../shared/errors";
import type { CreateExampleItem, ExampleItem, UpdateExampleItem } from "./example-item.model";
import type { ExampleItemRepository, Page } from "./example-item.repository";

/** Business rules for example items. */
export class ExampleItemService {
	readonly #repository: ExampleItemRepository;

	constructor(repository: ExampleItemRepository) {
		this.#repository = repository;
	}

	list(page: Page): Promise<ExampleItem[]> {
		return this.#repository.list(page);
	}

	async get(id: string): Promise<ExampleItem> {
		const item = await this.#repository.findById(id);
		if (!item) {
			throw new NotFoundError("example item", id);
		}
		return item;
	}

	create(input: CreateExampleItem): Promise<ExampleItem> {
		return this.#repository.create(input);
	}

	async update(id: string, input: UpdateExampleItem): Promise<ExampleItem> {
		const item = await this.#repository.update(id, input);
		if (!item) {
			throw new NotFoundError("example item", id);
		}
		return item;
	}

	async delete(id: string): Promise<void> {
		if (!(await this.#repository.delete(id))) {
			throw new NotFoundError("example item", id);
		}
	}
}
