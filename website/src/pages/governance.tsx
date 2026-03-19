import { SEO } from "@/components/seo";
import { Governance } from "@/components/sections/governance";

export function GovernancePage() {
  return (
    <>
      <SEO
        title="NetChain Governance - Decentralized Protocol Management | NetChain"
        description="NetChain governance model: community-driven proposals, on-chain voting, staking mechanisms, protocol parameter configuration, and transparent decision-making for the Proof of Internet blockchain."
        keywords="NetChain governance, blockchain governance, on-chain voting, staking, DAO, decentralized governance, protocol parameters, community proposals"
      />
      <Governance />
    </>
  );
}
