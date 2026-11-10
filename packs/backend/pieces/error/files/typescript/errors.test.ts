import { describe, expect, it } from "vitest";
import { z } from "zod";

import { NotFoundError, ValidationError, toProblem, validate } from "./errors";

describe("toProblem", () => {
	it("keeps the status and message of application errors", () => {
		expect(toProblem(new NotFoundError("invoice", "42"))).toEqual({
			type: "about:blank",
			title: "Not Found",
			status: 404,
			detail: "No invoice with id 42.",
		});
	});

	it("hides the message of unexpected errors", () => {
		const problem = toProblem(new Error("connection string with a password"));

		expect(problem.status).toBe(500);
		expect(problem.detail).not.toContain("password");
	});
});

describe("validate", () => {
	it("returns the parsed value", () => {
		expect(validate(z.object({ name: z.string().trim() }), { name: " Ada " })).toEqual({ name: "Ada" });
	});

	it("lists every issue", () => {
		const schema = z.object({ name: z.string(), age: z.number() });

		expect(() => validate(schema, {})).toThrow(ValidationError);
		try {
			validate(schema, {});
		} catch (error) {
			expect((error as ValidationError).errors).toHaveLength(2);
		}
	});
});
