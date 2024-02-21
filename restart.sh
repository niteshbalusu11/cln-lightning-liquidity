#!/bin/bash

# Step 1: Build inside the container
docker exec -it -w /workspaces/cln-lightning-liquidity festive_jackson cargo build

# Check if the build was successful before proceeding
if [ $? -ne 0 ]; then
    echo "Cargo build failed. Exiting script."
    exit 1
fi

# Define source and destination paths
SOURCE_PATH="./target/debug/cln-lightning-liquidity"
DESTINATION_PATH="/Users/niteshchowdharybalusu/.polar/networks/1/volumes/c-lightning/alice/lightningd/cln-lightning-liquidity"

# Step 2: Copy the file, replacing it if it exists
cp -f $SOURCE_PATH $DESTINATION_PATH

if [ $? -ne 0 ]; then
    echo "Failed to copy the binary. Check permissions and paths."
    exit 1
fi

echo "Binary copied successfully."

# Step 3: Stop and start the plugin
RPC_FILE_PATH="/home/clightning/lightning-rpc"
PLUGIN_PATH="/home/clightning/.lightning/cln-lightning-liquidity"

# Stop the plugin
docker exec -it polar-n1-alice lightning-cli --rpc-file=$RPC_FILE_PATH -k plugin subcommand=stop plugin=$PLUGIN_PATH
if [ $? -ne 0 ]; then
    echo "Failed to stop the plugin. Check the lightning-cli command and paths."
    # exit 1
fi

# Start the plugin
docker exec -it polar-n1-alice lightning-cli --rpc-file=$RPC_FILE_PATH -k plugin subcommand=start plugin=$PLUGIN_PATH
if [ $? -ne 0 ]; then
    echo "Failed to start the plugin. Check the lightning-cli command and paths."
    exit 1
fi

echo "Plugin restarted successfully."
