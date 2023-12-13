# Arbitrum ERC20 implementation

### Prerequisites

1. Install rust
2. Install and configure arbitrum stylus


### Run


1. Check deployment

```
cargo stylus check
```

2. Export abi

```
cargo stylus export-abi
```

3. Deploy on testnet
```
cargo stylus deploy --private-key=<YOUR_KEY> --estimate-gas-only
```
