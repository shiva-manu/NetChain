import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import {
  ArrowLeft,
  Blocks,
  Clock,
  Copy,
  FileText,
  Loader2,
  User,
  Check,
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
import type { BlockDetails } from "@/lib/types";

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

function getTxTypeLabel(txType: unknown): string {
  if (typeof txType === "string") return txType;
  if (typeof txType === "object" && txType !== null) {
    return Object.keys(txType)[0];
  }
  return "Unknown";
}

export function ExplorerBlockPage() {
  const { height } = useParams<{ height: string }>();
  const [block, setBlock] = useState<BlockDetails | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const client = new NetChainClient(DEFAULT_RPC_URL);

  useEffect(() => {
    if (!height) return;
    setLoading(true);
    client
      .getBlock(Number(height))
      .then(setBlock)
      .catch((e) => setError(e instanceof Error ? e.message : "Block not found"))
      .finally(() => setLoading(false));
  }, [height]);

  return (
    <>
      <SEO title={`Block ${height} | NetChain Explorer`} />
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
                <Blocks className="w-3.5 h-3.5" />
                Block
              </Badge>
              <h1 className="text-2xl font-bold">#{height}</h1>
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
            ) : block ? (
              <>
                {/* Block Header */}
                <Card className="mb-6">
                  <CardHeader>
                    <CardTitle>Block Details</CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                      <div>
                        <div className="text-sm text-muted-foreground mb-1">
                          Height
                        </div>
                        <div className="font-mono text-sm font-bold">
                          {block.index}
                        </div>
                      </div>
                      <div>
                        <div className="text-sm text-muted-foreground mb-1">
                          Timestamp
                        </div>
                        <div className="flex items-center gap-1.5 text-sm">
                          <Clock className="w-3.5 h-3.5 text-muted-foreground" />
                          {new Date(block.timestamp).toLocaleString()}
                        </div>
                      </div>
                    </div>

                    <div>
                      <div className="text-sm text-muted-foreground mb-1">
                        Block Hash
                      </div>
                      <div className="flex items-center gap-2">
                        <code className="text-xs font-mono bg-muted px-2 py-1 rounded break-all">
                          {block.hash}
                        </code>
                        <CopyButton text={block.hash} />
                      </div>
                    </div>

                    <div>
                      <div className="text-sm text-muted-foreground mb-1">
                        Previous Hash
                      </div>
                      <div className="flex items-center gap-2">
                        <code className="text-xs font-mono bg-muted px-2 py-1 rounded break-all">
                          {block.previous_hash}
                        </code>
                        <CopyButton text={block.previous_hash} />
                      </div>
                    </div>

                    <div>
                      <div className="text-sm text-muted-foreground mb-1">
                        Merkle Root
                      </div>
                      <div className="flex items-center gap-2">
                        <code className="text-xs font-mono bg-muted px-2 py-1 rounded break-all">
                          {block.merkle_root}
                        </code>
                        <CopyButton text={block.merkle_root} />
                      </div>
                    </div>

                    <div>
                      <div className="text-sm text-muted-foreground mb-1">
                        Validator
                      </div>
                      <Link
                        to={`/explorer/account/${block.validator}`}
                        className="flex items-center gap-1.5 text-sm text-primary hover:underline"
                      >
                        <User className="w-3.5 h-3.5" />
                        {block.validator}
                      </Link>
                    </div>

                    <div>
                      <div className="text-sm text-muted-foreground mb-1">
                        Transactions
                      </div>
                      <div className="flex items-center gap-1.5 text-sm">
                        <FileText className="w-3.5 h-3.5 text-muted-foreground" />
                        {block.transactions.length} transaction
                        {block.transactions.length !== 1 ? "s" : ""}
                      </div>
                    </div>
                  </CardContent>
                </Card>

                {/* Transactions */}
                {block.transactions.length > 0 && (
                  <Card>
                    <CardHeader>
                      <CardTitle>Transactions</CardTitle>
                    </CardHeader>
                    <CardContent>
                      <div className="space-y-3">
                        {block.transactions.map((signedTx, idx) => {
                          const txType = signedTx.tx?.tx_type;
                          const txTypeLabel = getTxTypeLabel(txType);

                          return (
                            <div
                              key={idx}
                              className="p-3 rounded-lg border bg-card/50"
                            >
                              <div className="flex items-center justify-between mb-2">
                                <Badge variant="secondary" className="text-xs">
                                  {txTypeLabel}
                                </Badge>
                                <span className="text-xs text-muted-foreground">
                                  Fee: {signedTx.tx?.fee ?? 0}
                                </span>
                              </div>
                              <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 text-sm">
                                <div>
                                  <span className="text-muted-foreground">
                                    From:{" "}
                                  </span>
                                  <Link
                                    to={`/explorer/account/${signedTx.tx?.sender}`}
                                    className="text-primary hover:underline font-mono text-xs"
                                  >
                                    {signedTx.tx?.sender?.slice(0, 12)}...
                                  </Link>
                                </div>
                                <div>
                                  <span className="text-muted-foreground">
                                    To:{" "}
                                  </span>
                                  {signedTx.tx?.receiver ? (
                                    <Link
                                      to={`/explorer/account/${signedTx.tx.receiver}`}
                                      className="text-primary hover:underline font-mono text-xs"
                                    >
                                      {signedTx.tx.receiver.slice(0, 12)}...
                                    </Link>
                                  ) : (
                                    <span className="text-xs text-muted-foreground">
                                      -
                                    </span>
                                  )}
                                </div>
                                <div>
                                  <span className="text-muted-foreground">
                                    Amount:{" "}
                                  </span>
                                  <span className="font-mono">
                                    {signedTx.tx?.amount ?? 0}
                                  </span>
                                </div>
                                <div>
                                  <span className="text-muted-foreground">
                                    Nonce:{" "}
                                  </span>
                                  <span className="font-mono">
                                    {signedTx.tx?.nonce ?? 0}
                                  </span>
                                </div>
                              </div>
                            </div>
                          );
                        })}
                      </div>
                    </CardContent>
                  </Card>
                )}
              </>
            ) : null}
          </div>
        </section>
      </div>
    </>
  );
}
