import { SEO } from "@/components/seo";
import { HowItWorks } from "@/components/sections/how-it-works";

export function HowItWorksPage() {
  return (
    <>
      <SEO
        title="How NetChain Works - Proof of Internet Consensus Explained | NetChain"
        description="Learn how NetChain's Proof of Internet works: network metrics measurement, peer attestation, reputation building, validator selection algorithm, and block production based on real internet performance."
        keywords="how NetChain works, PoI consensus mechanism, validator selection, network metrics, peer attestation, blockchain consensus, proof of internet explained"
      />
      <HowItWorks />
    </>
  );
}
