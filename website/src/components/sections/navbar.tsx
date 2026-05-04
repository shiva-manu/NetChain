import { useEffect, useState } from "react";
import { Link, useLocation } from "react-router-dom";
import { Menu, Blocks, ExternalLink } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ThemeToggle } from "@/components/theme-toggle";
import { Sheet, SheetContent, SheetTrigger, SheetClose } from "@/components/ui/sheet";
import { cn } from "@/lib/utils";

const navLinks = [
  { href: "/features", label: "Features" },
  { href: "/technology", label: "Technology" },
  { href: "/docs", label: "Docs" },
  { href: "/faucet", label: "Faucet" },
  { href: "/dashboard", label: "Explorer" },
];

export function Navbar() {
  const [scrolled, setScrolled] = useState(false);
  const location = useLocation();

  useEffect(() => {
    const handler = () => setScrolled(window.scrollY > 10);
    window.addEventListener("scroll", handler, { passive: true });
    return () => window.removeEventListener("scroll", handler);
  }, []);

  return (
    <header
      className={cn(
        "fixed top-0 left-0 right-0 z-50 transition-all duration-300",
        scrolled
          ? "border-b border-border/40 bg-background/80 backdrop-blur-md"
          : "bg-transparent"
      )}
    >
      <nav className="container-wide flex items-center justify-between h-16">
        {/* Logo */}
        <Link to="/" className="flex items-center gap-2.5 group">
          <div className="flex items-center justify-center w-8 h-8 rounded-lg bg-primary text-primary-foreground">
            <Blocks className="w-4 h-4" />
          </div>
          <span className="font-semibold text-lg tracking-tight text-foreground">
            NetChain
          </span>
        </Link>

        {/* Desktop nav */}
        <div className="hidden md:flex items-center gap-1">
          {navLinks.map((link) => (
            <Link
              key={link.href}
              to={link.href}
              className={cn(
                "relative px-3.5 py-2 text-sm font-medium transition-colors rounded-lg",
                location.pathname === link.href
                  ? "text-foreground"
                  : "text-muted-foreground hover:text-foreground"
              )}
            >
              {link.label}
              {location.pathname === link.href && (
                <span className="absolute bottom-0 left-1/2 -translate-x-1/2 w-4 h-0.5 bg-primary rounded-full" />
              )}
            </Link>
          ))}
        </div>

        {/* Desktop actions */}
        <div className="hidden md:flex items-center gap-2">
          <ThemeToggle />
          <Button variant="terminal" size="sm" href="/get-started">
            Get Started
          </Button>
          <Button variant="default" size="sm" href="/dashboard">
            Dashboard
            <ExternalLink className="w-3.5 h-3.5 ml-1" />
          </Button>
        </div>

        {/* Mobile menu */}
        <div className="flex md:hidden items-center gap-2">
          <ThemeToggle />
          <Sheet>
            <SheetTrigger
              className="inline-flex items-center justify-center rounded-lg h-10 w-10 hover:bg-muted transition-colors"
              aria-label="Open menu"
            >
              <Menu className="w-5 h-5" />
            </SheetTrigger>
            <SheetContent side="right" className="w-72">
              <div className="flex flex-col gap-1 mt-8">
                {navLinks.map((link) => (
                  <SheetClose
                    key={link.href}
                    render={
                      <Link
                        to={link.href}
                        className={cn(
                          "px-4 py-3 text-sm font-medium transition-colors rounded-lg",
                          location.pathname === link.href
                            ? "text-primary bg-primary/10"
                            : "text-muted-foreground hover:text-foreground hover:bg-muted"
                        )}
                      />
                    }
                  >
                    {link.label}
                  </SheetClose>
                ))}
                <div className="flex flex-col gap-3 pt-4 mt-4 border-t border-border">
                  <SheetClose
                    render={
                      <Link
                        to="/get-started"
                        className="inline-flex items-center justify-center gap-2 h-10 px-5 text-sm font-medium rounded-lg border border-primary/40 text-primary hover:bg-primary/10 transition-colors"
                      />
                    }
                  >
                    Get Started
                  </SheetClose>
                  <SheetClose
                    render={
                      <Link
                        to="/dashboard"
                        className="inline-flex items-center justify-center gap-2 h-10 px-5 text-sm font-medium rounded-lg bg-primary text-primary-foreground hover:brightness-110 transition-colors"
                      />
                    }
                  >
                    Dashboard
                  </SheetClose>
                </div>
              </div>
            </SheetContent>
          </Sheet>
        </div>
      </nav>
    </header>
  );
}
