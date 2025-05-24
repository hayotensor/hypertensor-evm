#### Run the node locally
```bash
./target/release/solochain-template-node --dev
```

#### Build smart contracts

```bash
npm run build
```

#### Run tests

```bash
npm test
```

#### To run a particular test case, you can pass an argument with the name or part of the name. For example:

```bash
npm test -- -g "testing register subnet"
```

#### TODO:

use polkadot-api via papi or run node per test automatically.

#### Note:

- Some tests require isolation due to subnet registration intervals.
- These test suites only verify the precompiles call the functions and they do and store the data that is expected. For logic tests see the pallets directory.

<!-- #### Use the polkadot api via papi
```bash
npx papi add dev -w ws://127.0.0.1:9944
```
 -->
