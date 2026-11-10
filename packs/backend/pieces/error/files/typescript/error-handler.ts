// @tfverse if stack.hono
import type { ErrorHandler } from "hono";
import { HTTPException } from "hono/http-exception";
import type { ContentfulStatusCode } from "hono/utils/http-status";

import { toProblem } from "./errors";

/** Sends every error as problem details; register it with `app.onError(errorHandler)`. */
export const errorHandler: ErrorHandler = (error, c) => {
	if (error instanceof HTTPException) {
		return error.getResponse();
	}
	const problem = toProblem(error);
	if (problem.status >= 500) {
		console.error(error);
	}
	return c.body(JSON.stringify(problem), problem.status as ContentfulStatusCode, {
		"Content-Type": "application/problem+json",
	});
};
// @tfverse else
import type { ErrorRequestHandler } from "express";

import { toProblem } from "./errors";

/** Sends every error as problem details; register it last with `app.use(errorHandler)`. */
export const errorHandler: ErrorRequestHandler = (error, _request, response, next) => {
	if (response.headersSent) {
		next(error);
		return;
	}
	const problem = toProblem(error);
	if (problem.status >= 500) {
		console.error(error);
	}
	response.status(problem.status).type("application/problem+json").json(problem);
};
// @tfverse end
