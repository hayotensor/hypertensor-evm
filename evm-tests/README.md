#### Install

```bash
npm i
```

#### Run the node locally
```bash
./target/release/solochain-template-node --dev
```

#### Run locally with manual sealing
- Overwatch node testing
```bash
./target/release/solochain-template-node --dev \
	--tmp --log lalala=trace \
	--chain=eth_dev \
	--sealing=manual \
	--validator \
	--force-authoring \
	--no-grandpa \
	--execution=Native \
	--unsafe-force-node-key-generation
```

#### Use the polkadot api via papi
```bash
npx papi add dev -w ws://127.0.0.1:9944
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
Most tests must be performed independently because of time based logic on some functionality like unbonding

```bash
npm test -- -g "testing register subnet"
```

#### Note:

- Some tests require isolation due to subnet registration intervals.
- These test suites only verify the precompiles call the functions and they do and store the data that is expected. For logic tests see the pallets directory.

#### Todos:

- Convert all tests to manual sealing for faster testing