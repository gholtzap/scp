#!/bin/bash

set -e

echo "Setting up SCP repository..."

if [ ! -f .env ]; then
    echo "Creating .env file from .env.example..."
    cp .env.example .env

    echo ""
    echo "Please configure the following variables in .env:"
    echo "  - MONGODB_URI (MongoDB connection string)"
    echo "  - NRF_URI (Network Repository Function URI)"
    echo "  - NF_INSTANCE_ID (Unique UUID for this NF instance)"
    echo ""

    read -p "Press Enter to continue with default values or Ctrl+C to exit and configure manually..."

    NF_UUID=$(uuidgen | tr '[:upper:]' '[:lower:]' | tr -d '-')

    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s|NF_INSTANCE_ID=.*|NF_INSTANCE_ID=$NF_UUID|g" .env
    else
        sed -i "s|NF_INSTANCE_ID=.*|NF_INSTANCE_ID=$NF_UUID|g" .env
    fi

    echo "Generated NF_INSTANCE_ID: $NF_UUID"
    echo "Note: You still need to configure MONGODB_URI and NRF_URI in .env"
else
    echo ".env file already exists, skipping..."
fi

echo ""
echo "Fetching Rust dependencies..."
cargo fetch

echo ""
echo "Building project..."
cargo build

echo ""
echo "Setup complete!"
echo ""
echo "Next steps:"
echo "  1. Configure .env with your MongoDB credentials and NRF URI"
echo "  2. Run 'cargo run' to start the SCP server"
