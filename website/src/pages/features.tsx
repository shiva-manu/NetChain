import { SEO } from "@/components/seo";
import { Features } from "@/components/sections/features";

export function FeaturesPage() {
  return (
    <>
      <SEO
        title="NetChain Features - Proof of Internet Blockchain Capabilities | NetChain"
        description="Explore NetChain's powerful features: Proof of Internet consensus, real-time network performance validation, anti-gaming mechanisms, slashing protection, and decentralized validator selection based on internet metrics."
        keywords="NetChain features, PoI consensus, blockchain features, validator selection, network performance, anti-gaming, slashing, decentralized blockchain"
      />
      <Features />
    </>
  );
}
