import { SEO } from "@/components/seo";
import { GetStarted } from "@/components/sections/get-started";

export function GetStartedPage() {
  return (
    <>
      <SEO
        title="Get Started with NetChain - Run a Node, Become a Validator | NetChain"
        description="Start using NetChain: run a blockchain node, become a validator, create a wallet, stake tokens, and join the Proof of Internet network. Complete setup guide and developer documentation."
        keywords="NetChain setup, run blockchain node, become validator, NetChain wallet, staking guide, PoI validator, blockchain development, NetChain tutorial"
      />
      <GetStarted />
    </>
  );
}
