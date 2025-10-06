#### Install

```bash
npm i
```

#### Seed
Some tests have time constraints conditions, set the `build` function with seeded storage parameters in `pallets/network/src/lib.rs`.

##### Example
```bash
fn build(&self) {
	MinSubnetRegistrationEpochs::<T>::set(0);
	OverwatchEpochLengthMultiplier::<T>::set(1);
	OverwatchMinDiversificationRatio::<T>::set(0);
	OverwatchMinRepScore::<T>::set(0);
	OverwatchMinAvgAttestationRatio::<T>::set(0);
	OverwatchMinAge::<T>::set(0);
}
```

#### Build
```bash
cargo build --release
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

#### Use the polkadot api via papi (run while chain is running)
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
test -- -g "testing register subnet-0xzmghoq5702"
```

#### Note:

- Some tests require isolation due to subnet registration intervals.
- These test suites only verify the precompiles call the functions and they do and store the data that is expected. For logic tests see the pallets directory.

#### Todos:

- Convert all tests to manual sealing for faster testing
- Auto-chain restart for tests