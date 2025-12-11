#!/bin/bash
set -e

echo "🎮 Building GBEmu..."

# Check for required tools
command -v cargo >/dev/null 2>&1 || { echo "❌ Rust/Cargo is required but not installed."; exit 1; }
command -v wasm-pack >/dev/null 2>&1 || { echo "❌ wasm-pack is required. Install with: cargo install wasm-pack"; exit 1; }
command -v node >/dev/null 2>&1 || { echo "❌ Node.js is required but not installed."; exit 1; }

# Build WASM core
echo "📦 Building WASM core..."
cd core
wasm-pack build --target web --out-dir ../web/lib/wasm --release
cd ..

# Copy WASM binary to public folder for runtime loading
echo "📋 Copying WASM to public folder..."
mkdir -p web/public/wasm
cp web/lib/wasm/gbemu_core_bg.wasm web/public/wasm/

# Install web dependencies
echo "📦 Installing web dependencies..."
cd web
if command -v pnpm >/dev/null 2>&1; then
    pnpm install
else
    npm install
fi

# Build Next.js app
echo "🔨 Building Next.js app..."
if command -v pnpm >/dev/null 2>&1; then
    pnpm build
else
    npm run build
fi

echo "✅ Build complete!"
echo ""
echo "To start the app:"
echo "  cd web && npm start"
echo ""
echo "For development:"
echo "  cd web && npm run dev"