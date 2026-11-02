import { route } from "../lib/files.js";

export const prerender = false;
export const ALL = ({ request }) => route(request);
