import { SEO } from "@/components/seo";
import { Docs } from "@/components/sections/docs";

export function DocsPage() {
  return (
    <>
      <SEO
        title="Developer Documentation | NetChain"
        description="Complete developer documentation for NetChain. Learn how to run a node, become a validator, use the RPC API, WebSocket events, wallet CLI, and contribute to the open source project."
        keywords="NetChain documentation, blockchain API, RPC reference, WebSocket API, validator guide, node setup, developer docs, open source blockchain"
      />
      <Docs />
    </>
  );
}
