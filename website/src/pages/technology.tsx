import { SEO } from "@/components/seo";
import { Technology } from "@/components/sections/technology";

export function TechnologyPage() {
  return (
    <>
      <SEO
        title="NetChain Technology Stack - Rust, libp2p, Layer-1 Architecture | NetChain"
        description="Deep dive into NetChain's technology: Rust implementation, libp2p networking, Ed25519 cryptography, Sled storage, JSON-RPC API, WebSocket events, and modular blockchain architecture for optimal performance."
        keywords="NetChain technology, Rust blockchain, libp2p, Layer-1 architecture, Ed25519, blockchain technology, RPC API, WebSocket, Sled database"
      />
      <Technology />
    </>
  );
}
