import type { z } from "zod";

/** An error with an HTTP status, sent to clients as RFC 9457 problem details. */
export class AppError extends Error {
	readonly status: number;
	readonly title: string;
	readonly errors?: unknown;

	constructor(status: number, title: string, detail: string, errors?: unknown) {
		super(detail);
		this.name = new.target.name;
		this.status = status;
		this.title = title;
		this.errors = errors;
	}
}

/** The requested resource does not exist. */
export class NotFoundError extends AppError {
	constructor(resource: string, id: string) {
		super(404, "Not Found", `No ${resource} with id ${id}.`);
	}
}

/** The request is well-formed but its content is invalid. */
export class ValidationError extends AppError {
	constructor(detail: string, errors?: unknown) {
		super(422, "Unprocessable Content", detail, errors);
	}
}

/** The request conflicts with the current state, such as a duplicate. */
export class ConflictError extends AppError {
	constructor(detail: string) {
		super(409, "Conflict", detail);
	}
}

/** RFC 9457 problem details. */
export interface Problem {
	type: string;
	title: string;
	status: number;
	detail: string;
	errors?: unknown;
}

/** Turns any thrown value into problem details; unknown errors never leak their message. */
export function toProblem(error: unknown): Problem {
	if (error instanceof AppError) {
		const problem: Problem = { type: "about:blank", title: error.title, status: error.status, detail: error.message };
		return error.errors === undefined ? problem : { ...problem, errors: error.errors };
	}
	if (error instanceof SyntaxError) {
		return { type: "about:blank", title: "Bad Request", status: 400, detail: "The body must be valid JSON." };
	}
	return { type: "about:blank", title: "Internal Server Error", status: 500, detail: "Something went wrong." };
}

/** Parses `data` with `schema`, or throws a {@link ValidationError} listing every issue. */
export function validate<T extends z.ZodType>(schema: T, data: unknown): z.output<T> {
	const result = schema.safeParse(data);
	if (!result.success) {
		const errors = result.error.issues.map(({ path, message }) => ({ path: path.map(String).join("."), message }));
		throw new ValidationError("The request is invalid.", errors);
	}
	return result.data;
}
