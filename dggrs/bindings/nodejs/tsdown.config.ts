import { defineConfig } from 'tsdown'

export default defineConfig([
  {
    entry: ["./src/index.ts"],
    platform: "neutral",
    format: ["esm", "cjs"],
    dts: true,
    external: [
      "./src/index.node", // prevents bundling the native addon
      "fs",
      "path", // always externalize built-ins
    ],
  },
]);
