import { useEffect, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import {
  Blocks,
  Search,
  ArrowRight,
  Loader2,
  Clock,
  Hash,
  User,
  FileText,
} from "lucide-react";
import { SEO } from "@/components/seo";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { SectionBackground } from "@/components/sections/section-background";
import { NetChainClient, DEFAULT_RPC_URL } from "@/lib/netchain-client";
import type { BlockDetails, ChainInfo } from "@/lib/types";

function truncateHash(hash: string, chars = 8) {
  if (hash.length <= chars * 2 + 3) return hash;
  return `${hash.slice(0, chars)}...${hash.slice(-chars)}`;
}

function truncateAddress(addr: string, chars = 8) {
  if (addr.length <= chars * 2 + 3) return addr;
  return `${addr.slice(0, chars)}...${addr.slice(-chars)}`;
}

function timeAgo(timestamp: string): string {
  const now = Date.now();
  const then = new Date(timestamp).getTime();
  const seconds = Math.floor((now - then) / 1000);
  if (seconds < 60) return `${seconds}s ago`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

export function ExplorerPage() {
  const navigate = useNavigate();
  const [searchQuery, setSearchQuery] = useState("");
  const [chainInfo, setChainInfo] = useState<ChainInfo | null>(null);
  const [blocks, setBlocks] = useState<BlockDetails[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const client = new NetChainClient(DEFAULT_RPC_URL);

  useEffect(() => {
    async function fetchData() {
      try {
        const [info, recentBlocks] = await Promise.all([
          client.getChainInfo(),
          client.getBlocks(0, 10),
        ]);
        setChainInfo(info);
        setBlocks(recentBlocks);
        setError(null);
      } catch (e) {
        setError(e instanceof Error ? e.message : "Failed to fetch data");
      } finally {
        setLoading(false);
      }
    }
    fetchData();
    const interval = setInterval(fetchData, 15000);
    return () => clearInterval(interval);
  }, []);

  function handleSearch(e: React.FormEvent) {
    e.preventDefault();
    const q = searchQuery.trim();
    if (!q) return;

    if (/^\d+$/.test(q)) {
      navigate(`/explorer/block/${q}`);
    } else if (q.length === 64 && /^[0-9a-fA-F]+$/.test(q)) {
      navigate(`/explorer/tx/${q}`);
    } else if (q.length === 40 && /^[0-9a-fA-F]+$/.test(q)) {
      navigate(`/explorer/account/${q}`);
    } else {
      navigate(`/explorer/block/${q}`);
    }
  }

  return (
    <>
      <SEO
        title="Block Explorer | NetChain"
        description="Explore blocks, transactions, and accounts on the NetChain blockchain."
      />
      <div className="relative min-h-dvh">
        <SectionBackground variant="gradient" />
        <section className="relative z-10 container-wide py-24 md:py-32">
          <div className="mx-auto max-w-2xl text-center mb-12">
            <Badge variant="outline" className="mb-4 gap-1.5">
              <Blocks className="w-3.5 h-3.5" />
              Block Explorer
            </Badge>
            <h1 className="text-3xl font-bold tracking-tight sm:text-4xl mb-4">
              Explore NetChain
            </h1>
            <p className="text-muted-foreground">
              Search blocks, transactions, and accounts on the NetChain blockchain.
            </p>
          </div>

          {/* Search Bar */}
          <form onSubmit={handleSearch} className="mx-auto max-w-xl mb-12">
            <div className="relative">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
              <Input
                type="text"
                placeholder="Search by block height, hash, or address..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="pl-10 pr-20 h-12 text-base"
              />
              <Button
                type="submit"
                size="sm"
                className="absolute right-1.5 top-1/2 -translate-y-1/2"
              >
                Search
              </Button>
            </div>
          </form>

          {/* Chain Stats */}
          {chainInfo && (
            <div className="grid grid-cols-2 md:grid-cols-3 gap-4 mb-12 max-w-3xl mx-auto">
              <Card>
                <CardContent className="pt-6 text-center">
                  <div className="text-2xl font-bold">
                    {chainInfo.height.toLocaleString()}
                  </div>
                  <div className="text-sm text-muted-foreground">
                    Chain Height
                  </div>
                </CardContent>
              </Card>
              <Card>
                <CardContent className="pt-6 text-center">
                  <div className="text-2xl font-mono text-xs font-bold">
                    {truncateHash(chainInfo.latest_block_hash)}
                  </div>
                  <div className="text-sm text-muted-foreground">
                    Latest Hash
                  </div>
                </CardContent>
              </Card>
              <Card className="col-span-2 md:col-span-1">
                <CardContent className="pt-6 text-center">
                  <div className="text-2xl font-mono text-xs font-bold">
                    {truncateHash(chainInfo.genesis_hash)}
                  </div>
                  <div className="text-sm text-muted-foreground">
                    Genesis Hash
                  </div>
                </CardContent>
              </Card>
            </div>
          )}

          {/* Recent Blocks */}
          <Card className="max-w-4xl mx-auto">
            <CardHeader>
              <div className="flex items-center justify-between">
                <div>
                  <CardTitle className="flex items-center gap-2">
                    <Blocks className="w-5 h-5" />
                    Recent Blocks
                  </CardTitle>
                  <CardDescription>
                    Latest blocks produced on the chain
                  </CardDescription>
                </div>
              </div>
            </CardHeader>
            <CardContent>
              {loading ? (
                <div className="flex items-center justify-center py-12">
                  <Loader2 className="w-6 h-6 animate-spin text-muted-foreground" />
                </div>
              ) : error ? (
                <div className="text-center py-12 text-muted-foreground">
                  {error}
                </div>
              ) : blocks.length === 0 ? (
                <div className="text-center py-12 text-muted-foreground">
                  No blocks yet
                </div>
              ) : (
                <div className="space-y-2">
                  {blocks.map((block) => (
                    <Link
                      key={block.index}
                      to={`/explorer/block/${block.index}`}
                      className="flex items-center gap-4 p-3 rounded-lg hover:bg-muted/50 transition-colors"
                    >
                      <div className="flex-shrink-0 w-16 text-center">
                        <div className="text-sm font-bold">{block.index}</div>
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 text-sm">
                          <Hash className="w-3.5 h-3.5 text-muted-foreground flex-shrink-0" />
                          <span className="font-mono text-xs truncate">
                            {truncateHash(block.hash, 10)}
                          </span>
                        </div>
                        <div className="flex items-center gap-3 mt-1 text-xs text-muted-foreground">
                          <span className="flex items-center gap-1">
                            <User className="w-3 h-3" />
                            {truncateAddress(block.validator)}
                          </span>
                          <span className="flex items-center gap-1">
                            <FileText className="w-3 h-3" />
                            {block.transactions.length} txs
                          </span>
                        </div>
                      </div>
                      <div className="flex-shrink-0 text-xs text-muted-foreground flex items-center gap-1">
                        <Clock className="w-3 h-3" />
                        {timeAgo(block.timestamp)}
                      </div>
                      <ArrowRight className="w-4 h-4 text-muted-foreground flex-shrink-0" />
                    </Link>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        </section>
      </div>
    </>
  );
}
