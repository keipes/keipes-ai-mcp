import * as esbuild from "esbuild";

async function build() {
  console.log("Building Lambda functions...");

  await esbuild.build({
    entryPoints: ["src/index.ts"],
    bundle: true,
    outfile: "dist/src/index.js",
    platform: "node",
    target: "node18",
    external: ["aws-sdk"],
  });

  console.log("Build complete!");
}

build().catch(console.error);
