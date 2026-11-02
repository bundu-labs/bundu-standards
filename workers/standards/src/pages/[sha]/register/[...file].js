import { routeCached } from "../../../lib/files.js";

// On demand: a register file at a commit. Anything else is a 404 problem.
export const prerender = false;
export const ALL = ({ request }) => routeCached(request);
