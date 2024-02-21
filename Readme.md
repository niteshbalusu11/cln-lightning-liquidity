# cln-lightning-liquidity

### Buy lightning channels from any node on the network selling using [lsps1](https://github.com/BitcoinAndLightningLayerSpecs/lsp) protocol.

- Here's a website showing the current offers: [https://sparkseer.space/chanoffers](https://sparkseer.space/chanoffers)
- Tools compatible for LND [BalanceOfSatoshis](https://github.com/alexbosworth/balanceofsatoshis)

### Installation
- You need to install [Rust](https://www.rust-lang.org/)

```
# Clone the repo
git clone https://github.com/niteshbalusu11/cln-lightning-liquidity.git

# Go into it
cd cln-lightning-liquidity

# Build
cargo build --release

# In your core lightning config
plugin=/path/to/cln-lightning-liquidity

# Start the plugin
```

### Usage

#### Help command:
- lightning-cli buy-inbound-channel method=help
```json
{   "cli_params": {
      "amount": "<number> enter the channel size you want to buy",
      "blocks": "<number> enter the number of blocks you want to wait for the channel to be confirmed",
      "method": "Method can be one of the following: (help, buy, getinfo, getorder)",
      "orderid": "<orderid> returns the status of the order",
      "type": "<private/public> the type of channel you want to buy",
      "uri": "<uri> pubkey@host:port"
   }
}
```

### Example getinfo
- lightning-cli buy-inbound-channel method=getinfo uri="pubkey@ip:port"

#### Example buy a channel
- lightning-cli buy-inbound-channel method=buy uri="pubkey@ip:port" amount=100000 blocks=144 method=buy type=private

### Example getorder
- lightning-cli buy-inbound-channel method=getorder uri="pubkey@ip:port" orderid="orderid"

#### Everything gets logged to cln log file. Yeah, I have no idea how to print it to the console yet.