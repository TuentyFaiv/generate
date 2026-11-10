import type { PrismaClient } from "@prisma/client";

import type { CreateExampleItem, ExampleItem, UpdateExampleItem } from "./example-item.model";
import type { ExampleItemRepository, Page } from "./example-item.repository";

/** Stores example items with Prisma; needs an `ExampleItem` model in `schema.prisma`. */
export class PrismaExampleItemRepository implements ExampleItemRepository {
	readonly #prisma: PrismaClient;

	constructor(prisma: PrismaClient) {
		this.#prisma = prisma;
	}

	list({ limit, offset }: Page): Promise<ExampleItem[]> {
		return this.#prisma.exampleItem.findMany({ orderBy: { createdAt: "asc" }, take: limit, skip: offset });
	}

	async findById(id: string): Promise<ExampleItem | undefined> {
		return (await this.#prisma.exampleItem.findUnique({ where: { id } })) ?? undefined;
	}

	create(input: CreateExampleItem): Promise<ExampleItem> {
		return this.#prisma.exampleItem.create({ data: input });
	}

	async update(id: string, input: UpdateExampleItem): Promise<ExampleItem | undefined> {
		const { count } = await this.#prisma.exampleItem.updateMany({ where: { id }, data: input });
		return count > 0 ? this.findById(id) : undefined;
	}

	async delete(id: string): Promise<boolean> {
		const { count } = await this.#prisma.exampleItem.deleteMany({ where: { id } });
		return count > 0;
	}
}
