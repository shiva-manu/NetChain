import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import {
  ArrowLeft,
  Copy,
  Loader2,
  Wallet,
  Check,
  Coins,
  Lock,
} from "lucide-react";
import { SEO } from "@/components/seo";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { SectionBackground } from "@/components/sections/section-background";
import { NetChainClient, DEFAULT_RPC_URL } from "@/lib/netchain-client";
import type { AccountInfo } from "@/lib/types";

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);
  return (
    <Button
      variant="ghost"
      size="sm"
      className="h-6 w-6 p-0"
      onClick={() => {
        navigator.clipboard.writeText(text);
        setCopied(true);
        setTimeout(() => setCopied(false), 1500);
      }}
    >
      {copied ? (
        <Check className="w-3 h-3 text-green-500" />
      ) : (
        <Copy className="w-3 h-3" />
      )}
    </Button>
  );
}

export function ExplorerAccountPage() {
  const { address } = useParams<{ address: string }>();
  const [account, setAccount] = useState<AccountInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const client = new NetChainClient(DEFAULT_RPC_URL);

  useEffect(() => {
    if (!address) return;
    setLoading(true);
    client
      .getAccount(address)
      .then(setAccount)
      .catch((e) =>
        setError(e instanceof Error ? e.message : "Account not found")
      )
      .finally(() => setLoading(false));
  }, [address]);

  return (
    <>
      <SEO title={`Account ${address?.slice(0, 12)}... | NetChain Explorer`} />
      <div className="relative min-h-dvh">
        <SectionBackground variant="gradient" />
        <section className="relative z-10 container-wide py-24 md:py-32">
          <div className="max-w-4xl mx-auto">
            <Button variant="ghost" size="sm" className="mb-6" href="/explorer">
              <ArrowLeft className="w-4 h-4 mr-1" />
              Back to Explorer
            </Button>

            <div className="flex items-center gap-3 mb-8">
              <Badge variant="outline" className="gap-1.5">
                <Wallet className="w-3.5 h-3.5" />
                Account
              </Badge>
              <h1 className="text-2xl font-bold">Account Details</h1>
            </div>

            {loading ? (
              <div className="flex items-center justify-center py-24">
                <Loader2 className="w-6 h-6 animate-spin text-muted-foreground" />
              </div>
            ) : error ? (
              <Card>
                <CardContent className="py-12 text-center text-muted-foreground">
                  {error}
                </CardContent>
              </Card>
            ) : account ? (
              <>
                {/* Address */}
                <Card className="mb-6">
                  <CardHeader>
                    <CardTitle>Address</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="flex items-center gap-2">
                      <code className="text-xs font-mono bg-muted px-2 py-1 rounded break-all">
                        {account.address}
                      </code>
                      <CopyButton text={account.address} />
                    </div>
                  </CardContent>
                </Card>

                {/* Balances */}
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
                  <Card>
                    <CardContent className="pt-6">
                      <div className="flex items-center gap-2 mb-2">
                        <Coins className="w-4 h-4 text-primary" />
                        <span className="text-sm text-muted-foreground">
                          Balance
                        </span>
                      </div>
                      <div className="text-2xl font-bold">
                        {account.balance.toLocaleString()}
                      </div>
                      <div className="text-xs text-muted-foreground">NCN</div>
                    </CardContent>
                  </Card>

                  <Card>
                    <CardContent className="pt-6">
                      <div className="flex items-center gap-2 mb-2">
                        <Lock className="w-4 h-4 text-primary" />
                        <span className="text-sm text-muted-foreground">
                          Staked
                        </span>
                      </div>
                      <div className="text-2xl font-bold">
                        {account.staked_balance.toLocaleString()}
                      </div>
                      <div className="text-xs text-muted-foreground">NCN</div>
                    </CardContent>
                  </Card>

                  <Card>
                    <CardContent className="pt-6">
                      <div className="flex items-center gap-2 mb-2">
                        <span className="text-sm text-muted-foreground">
                          Nonce
                        </span>
                      </div>
                      <div className="text-2xl font-bold">{account.nonce}</div>
                      <div className="text-xs text-muted-foreground">
                        Transactions sent
                      </div>
                    </CardContent>
                  </Card>
                </div>
              </>
            ) : null}
          </div>
        </section>
      </div>
    </>
  );
}
