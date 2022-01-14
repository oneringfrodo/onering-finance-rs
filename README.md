# onering-finance

OneRing Finance program built on solana blockchain. You will find requirement document [here](https://docs.google.com/document/d/1aPkFisFQSQzbG9o5smNHlu3VJRebEziA-o50AEYEjV0/edit).

## Installation
```bash
yarn install
```

## Build program
```bash
anchor build
```

## Test program
```bash
anchor test
```

## Deploy program
```bash
anchor deploy --provider.cluster devnet \
    --provider.wallet ~/.config/solana/JBxidGWnhtPTGg8xw7sFT9tF4cfGtHnjYNp5GDJvGveh.json
```

## Upgrade program
```bash
anchor upgrade --provider.cluster devnet \
    --program-id RNGF2q87ouXMQGTxgcFPrxdUC2SFTx9HoBvhCSfpuUd \
    --provider.wallet ~/.config/solana/JBxidGWnhtPTGg8xw7sFT9tF4cfGtHnjYNp5GDJvGveh.json \
    ./target/deploy/onering_finance.so
```

## Deploy troubleshoot
```bash
solana program show --buffers -u devnet --buffer-authority JBxidGWnhtPTGg8xw7sFT9tF4cfGtHnjYNp5GDJvGveh
solana program close --buffers -u devnet -k ~/.config/solana/JBxidGWnhtPTGg8xw7sFT9tF4cfGtHnjYNp5GDJvGveh.json
```
