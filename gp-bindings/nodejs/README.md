# tsdown-starter

A starter for creating a TypeScript package.

## Development

- Install dependencies:

```bash
npm install
```

- Run the unit tests:

```bash
npm run test
```

- Build the library:

```bash
npm run build
```
The `build` script will build a new folder `dist` with the bundled files inside, and copy the index.node and index.d.ts files into that same folder. This folder will be used in the npm library as the source code files.


NOTE: Eventually a github job will be added for this statement workflow.