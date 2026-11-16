import { env } from "cloudflare:workers";
import { routeCached } from "../../../lib/files.js";

// On demand: a register file at a commit. Anything else is a 404 problem.
// The Rust Worker (workers/standards-api) when the STANDARDS_API service binding exists;
// this JS otherwise, so nothing changes until the binding is added (#87).
export const prerender = false;
export const ALL = ({ request }) =>
  env.STANDARDS_API ? env.STANDARDS_API.fetch(request) : routeCached(request);
