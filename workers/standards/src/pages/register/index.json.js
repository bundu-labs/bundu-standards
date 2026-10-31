import { REGISTER } from "../../lib/data.js";
import { indexJson } from "../../lib/register.js";

export const prerender = true;
export function GET({ site }) {
  return new Response(JSON.stringify(indexJson(REGISTER, site?.origin), null, 2) + "\n", {
    headers: { "content-type": "application/json; charset=utf-8", "access-control-allow-origin": "*" },
  });
}
