// @tfverse if stack.hono
import { createMiddleware } from "hono/factory";

// @tfverse else
import type { RequestHandler } from "express";

// @tfverse end
/** Header that carries the request id. */
export const REQUEST_ID_HEADER = "x-request-id";

const VALID_REQUEST_ID = /^[\w-]{1,128}$/;

/** A trusted request id: the caller's when it is valid, a new UUID otherwise. */
export function requestIdFrom(incoming: string | undefined): string {
	return incoming && VALID_REQUEST_ID.test(incoming) ? incoming : crypto.randomUUID();
}

// @tfverse if stack.hono
/** Reuses or creates the request id, stores it as `c.get("requestId")` and echoes it in the response. */
export const requestId = createMiddleware<{ Variables: { requestId: string } }>(async (c, next) => {
	const value = requestIdFrom(c.req.header(REQUEST_ID_HEADER));
	c.set("requestId", value);
	c.header(REQUEST_ID_HEADER, value);
	await next();
});
// @tfverse else
/** Reuses or creates the request id, stores it as `response.locals.requestId` and echoes it. */
export const requestId: RequestHandler = (request, response, next) => {
	const value = requestIdFrom(request.get(REQUEST_ID_HEADER));
	response.locals.requestId = value;
	response.setHeader(REQUEST_ID_HEADER, value);
	next();
};
// @tfverse end
