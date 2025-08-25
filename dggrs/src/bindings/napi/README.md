## Generate the bindings

### Install cli

```
yarn global add @napi-rs/cli
# or
npm install -g @napi-rs/cli
# or
pnpm add -g @napi-rs/cli
```

then from the root of the repo:
```
cd dggrs/src/bindings/napi
```

then:
```
npx napi build --manifest-path ../../../Cargo.toml --output-dir ../../../../gp-bindings/nodejs/src  --release
```

This will generate two files in the directory `gp-bindings/nodejs/src`:
- `index.d.ts`, the types.
- `index.node`, the binary nodejs file.


NOTE: Eventually a github job will be added for this statement workflow.