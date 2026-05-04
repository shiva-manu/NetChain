import { useState } from "react";
import { Droplet, AlertTriangle, Check, Wallet, ArrowRight, Sparkles, Coins } from "lucide-react";
import { SEO } from "@/components/seo";
import { Card, CardHeader, CardTitle, CardDescription, CardContent, CardFooter } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { FadeIn } from "@/components/ui/fade-in";
import { SectionBackground } from "@/components/sections/section-background";
import { useNetChain } from "@/lib/use-netchain";
import { cn } from "@/lib/utils";

export function FaucetPage() {
  const { requestTokens, isConnected } = useNetChain();
  const [address, setAddress] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [lastTx, setLastTx] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!address) return;
    setIsLoading(true);
    setError(null);
    setLastTx(null);
    try {
      const txHash = await requestTokens(address);
      setLastTx(txHash);
      setAddress("");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to request tokens");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="relative min-h-screen">
      <SEO title="Faucet | NetChain" description="Request testnet tokens for the NetChain blockchain. Get NCN to test transactions, staking, and governance." keywords="NetChain faucet, testnet tokens, crypto faucet, NCN tokens" />

      {/* Hero Section */}
      <section className="relative pt-32 pb-24 overflow-hidden">
        <SectionBackground variant="gradient" />
        <div className="absolute inset-0 bg-grid-fine opacity-30" />

        <div className="container-wide relative z-10">
          <FadeIn direction="up">
            <div className="max-w-2xl mx-auto text-center space-y-8">
              <Badge variant="signal" size="lg">
                <Droplet className="w-4 h-4" />
                Testnet Faucet
              </Badge>
              <h1 className="text-5xl md:text-6xl font-bold tracking-tighter">
                Get Testnet <span className="text-gradient">NCN</span>
              </h1>
              <p className="text-xl text-muted-foreground leading-relaxed">
                Request free test tokens to explore the NetChain ecosystem. These tokens have no real-world value and are for testing purposes only.
              </p>
            </div>
          </FadeIn>
        </div>
      </section>

      {/* Faucet Form */}
      <section className="container-wide pb-24 relative z-10">
        <FadeIn direction="up" delay={200}>
          <div className="max-w-lg mx-auto">
            <Card variant="glass" size="lg" className="shadow-xl">
              <CardHeader className="border-b border-border/50 pb-6">
                <div className="flex items-center gap-3 mb-2">
                  <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-primary to-accent text-white flex items-center justify-center">
                    <Coins className="w-6 h-6" />
                  </div>
                  <div>
                    <CardTitle className="text-xl">Request Funds</CardTitle>
                    <CardDescription>Enter your wallet address to receive 10 NCN</CardDescription>
                  </div>
                </div>
              </CardHeader>

              <CardContent className="pt-6 space-y-5">
                <form onSubmit={handleSubmit} className="space-y-5">
                  <div className="space-y-2">
                    <label htmlFor="address" className="text-sm font-medium text-foreground">Wallet Address</label>
                    <div className="relative">
                      <Wallet className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-muted-foreground" />
                      <Input id="address" placeholder="netchain1..." value={address} onChange={(e) => setAddress(e.target.value)} className="pl-12 h-14 text-base" autoComplete="off" spellCheck="false" />
                    </div>
                  </div>

                  {error && (
                    <div className="p-4 rounded-xl bg-destructive/10 border border-destructive/20 text-destructive text-sm flex items-start gap-3">
                      <AlertTriangle className="w-5 h-5 shrink-0 mt-0.5" />
                      <span>{error}</span>
                    </div>
                  )}

                  {lastTx && (
                    <div className="p-4 rounded-xl bg-success/10 border border-success/20 text-success space-y-2">
                      <div className="flex items-center gap-2 font-semibold">
                        <Check className="w-5 h-5" />
                        <span>Tokens Sent Successfully!</span>
                      </div>
                      <div className="text-xs break-all opacity-80 font-mono bg-success/10 p-2 rounded-lg">Tx: {lastTx}</div>
                    </div>
                  )}

                  <Button type="submit" variant="premium" size="lg" fullWidth disabled={isLoading || !address} isLoading={isLoading} rightIcon={!isLoading ? <Sparkles className="w-5 h-5" /> : undefined}>
                    {isLoading ? "Requesting..." : "Request 10 NCN"}
                  </Button>
                </form>
              </CardContent>

              <CardFooter className="bg-secondary/30 text-sm text-muted-foreground justify-center text-center p-5 border-t border-border/50">
                <div className="space-y-1">
                  <p>Limit: 1 request per hour per address</p>
                  <p>Network Status: <span className={cn("font-semibold", isConnected ? "text-success" : "text-destructive")}>{isConnected ? "Operational" : "Disconnected"}</span></p>
                </div>
              </CardFooter>
            </Card>

            <FadeIn direction="up" delay={400}>
              <div className="mt-10 text-center">
                <p className="text-muted-foreground mb-5">Don't have a wallet yet?</p>
                <div className="flex flex-wrap justify-center gap-4">
                  <Button variant="secondary" size="md" href="/get-started"><Wallet className="w-4 h-4" /> Get Wallet</Button>
                  <Button variant="ghost" size="md" href="/dashboard" rightIcon={<ArrowRight className="w-4 h-4" />}>Go to Dashboard</Button>
                </div>
              </div>
            </FadeIn>
          </div>
        </FadeIn>
      </section>
    </div>
  );
}
