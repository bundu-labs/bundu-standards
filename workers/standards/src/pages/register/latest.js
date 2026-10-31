import { routeCached } from "../../lib/files.js";

export const prerender = false;
export const ALL = ({ request }) => routeCached(request);
