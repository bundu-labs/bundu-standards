import { defineConfig } from "astro/config";
import cloudflare from "@astrojs/cloudflare";

// standards.bundu.org: the Bundu standards register for people (static pages,
// built from register/register.yaml) and for machines (a read-only endpoint for
// the register files at a commit). Astro on Cloudflare Workers.
export default defineConfig({
  site: "https://standards.bundu.org",
  adapter: cloudflare(),
  trailingSlash: "ignore",
  // No sessions: nothing here has a user, so no KV namespace to provision.
  session: false,
  build: {
    // No inline <style>: the Content-Security-Policy allows only our own origin.
    inlineStylesheets: "never",
  },
  vite: {
    // register/register.yaml sits two folders above this project.
    server: { fs: { allow: ["../.."] } },
  },
});
