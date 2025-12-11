import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // Output standalone for optimized production builds
  output: "standalone",

  // Disable strict mode for better compatibility with WASM
  reactStrictMode: false,

  // Turbopack config (Next.js 16 default bundler)
  turbopack: {
    // Empty config to acknowledge Turbopack usage
  },

  // Headers for WASM and SharedArrayBuffer support
  async headers() {
    return [
      {
        source: "/:path*",
        headers: [
          {
            key: "Cross-Origin-Opener-Policy",
            value: "same-origin",
          },
          {
            key: "Cross-Origin-Embedder-Policy",
            value: "require-corp",
          },
        ],
      },
      {
        source: "/wasm/:path*",
        headers: [
          {
            key: "Content-Type",
            value: "application/wasm",
          },
          {
            key: "Cache-Control",
            value: "public, max-age=31536000, immutable",
          },
        ],
      },
    ];
  },
};

export default nextConfig;
