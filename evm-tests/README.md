
## run the node locally
```bash
./target/release/solochain-template-node --dev
```

## use the polkadot api
```bash
# npx papi add devnet -n ws://127.0.0.1:9944
npx papi add dev -w ws://127.0.0.1:9944
```

## run  tests

```bash
npm run test
```

<!-- 

## To run a particular test case, you can pass an argument with the name or part of the name. For example:

```bash
yarn run test -- -g "Can set subnet parameter"
``` -->
