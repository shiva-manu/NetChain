import { useState, useEffect } from "react";
import { Link, NavLink, type NavLinkRenderProps } from "react-router-dom";
import { ThemeToggle } from "@/components/theme-toggle";
import {
  Sheet,
  SheetContent,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet";
import { Menu, X, LayoutDashboard, Github, ArrowRight } from "lucide-react";
import { cn } from "@/lib/utils";

const navLinks = [
  { label: "Features", to: "/features" },
  { label: "How It Works", to: "/how-it-works" },
  { label: "Technology", to: "/technology" },
  { label: "Governance", to: "/governance" },
  { label: "Docs", to: "/docs" },
];

function navLinkClassName({ isActive }: NavLinkRenderProps) {
  return cn(
    "relative px-3 py-2 text-sm font-medium transition-all duration-300",
    "after:absolute after:bottom-0 after:left-1/2 after:h-0.5 after:w-0 after:-translate-x-1/2",
    "after:bg-gradient-to-r after:from-primary after:to-accent after:transition-all after:duration-300",
    "hover:after:w-full focus-visible:after:w-full",
    isActive 
      ? "text-foreground after:w-full" 
      : "text-muted-foreground hover:text-foreground"
  );
}

export function Navbar() {
  const [open, setOpen] = useState(false);
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const handleScroll = () => {
      setScrolled(window.scrollY > 20);
    };
    
    window.addEventListener("scroll", handleScroll, { passive: true });
    return () => window.removeEventListener("scroll", handleScroll);
  }, []);

  return (
    <header
      className={cn(
        "sticky top-0 z-50 w-full transition-all duration-500",
        scrolled
          ? "border-b border-border/40 bg-background/80 backdrop-blur-xl supports-[backdrop-filter]:bg-background/60"
          : "bg-transparent"
      )}
    >
      <nav
        className="mx-auto flex h-16 max-w-7xl items-center justify-between px-4 sm:px-6 lg:px-8"
        aria-label="Main navigation"
      >
        {/* Logo */}
        <Link
          to="/"
          className="group flex items-center gap-3 transition-opacity hover:opacity-90"
        >
          <div className="relative flex size-9 items-center justify-center">
            {/* Glow effect */}
            <div 
              className="absolute inset-0 rounded-lg bg-gradient-to-br from-primary to-accent opacity-20 blur-md transition-opacity group-hover:opacity-40" 
              aria-hidden="true"
            />
            {/* Logo container */}
            <div className="relative flex size-9 items-center justify-center rounded-lg bg-gradient-to-br from-primary to-accent">
              <svg
                viewBox="0 0 24 24"
                fill="none"
                className="size-5 text-primary-foreground"
                aria-hidden="true"
              >
                <path
                  d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"
                  fill="currentColor"
                />
              </svg>
            </div>
          </div>
          <span className="text-lg font-bold tracking-tight text-foreground">
            NetChain
          </span>
        </Link>

        {/* Desktop Navigation */}
        <div className="hidden items-center gap-1 lg:flex">
          {navLinks.map((link) => (
            <NavLink key={link.to} to={link.to} className={navLinkClassName}>
              {link.label}
            </NavLink>
          ))}
        </div>

        {/* Desktop Actions */}
        <div className="hidden items-center gap-3 lg:flex">
          <ThemeToggle />
          
          <NavLink
            to="/dashboard"
            className={({ isActive }) =>
              cn(
                "flex items-center gap-2 rounded-lg px-3 py-2 text-sm font-medium transition-all duration-300",
                "border border-border/50 hover:border-primary/50 hover:bg-primary/5",
                isActive ? "border-primary/50 bg-primary/5 text-primary" : "text-muted-foreground hover:text-foreground"
              )
            }
          >
            <LayoutDashboard className="size-4" aria-hidden="true" />
            Explorer
          </NavLink>
          
          <a
            href="https://github.com/shiva-manu/NetChain"
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center gap-2 rounded-lg border border-border/50 px-3 py-2 text-sm font-medium text-muted-foreground transition-all duration-300 hover:border-border hover:bg-muted/50 hover:text-foreground"
            aria-label="View NetChain on GitHub"
          >
            <Github className="size-4" aria-hidden="true" />
            GitHub
          </a>
          
          <Link
            to="/get-started"
            className="group relative flex items-center gap-2 overflow-hidden rounded-lg bg-gradient-to-r from-primary to-accent px-4 py-2 text-sm font-semibold text-primary-foreground transition-all duration-300 hover:shadow-lg hover:shadow-primary/25"
          >
            <span className="relative z-10">Get Started</span>
            <ArrowRight className="relative z-10 size-4 transition-transform duration-300 group-hover:translate-x-0.5" aria-hidden="true" />
            {/* Shine effect */}
            <div 
              className="absolute inset-0 -translate-x-full bg-gradient-to-r from-transparent via-white/20 to-transparent transition-transform duration-500 group-hover:translate-x-full"
              aria-hidden="true"
            />
          </Link>
        </div>

        {/* Mobile Actions */}
        <div className="flex items-center gap-2 lg:hidden">
          <ThemeToggle />
          <Sheet open={open} onOpenChange={setOpen}>
            <SheetTrigger
              className="inline-flex size-10 cursor-pointer items-center justify-center rounded-lg border border-border/50 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground focus-visible:outline-2 focus-visible:outline-ring"
              aria-label={open ? "Close menu" : "Open menu"}
            >
              {open ? <X className="size-5" /> : <Menu className="size-5" />}
            </SheetTrigger>
            <SheetContent side="right" className="w-80 border-border/50 bg-background/95 backdrop-blur-xl">
              <SheetTitle className="sr-only">Navigation menu</SheetTitle>
              
              <div className="flex flex-col gap-2 pt-8">
                {/* Mobile nav links */}
                {navLinks.map((link, index) => (
                  <NavLink
                    key={link.to}
                    to={link.to}
                    onClick={() => setOpen(false)}
                    className={({ isActive }) =>
                      cn(
                        "rounded-lg px-4 py-3 text-base font-medium transition-all duration-300",
                        "opacity-0 animate-slide-in-right",
                        isActive
                          ? "bg-primary/10 text-primary"
                          : "text-muted-foreground hover:bg-muted hover:text-foreground"
                      )
                    }
                    style={{ animationDelay: `${index * 50}ms`, animationFillMode: 'forwards' }}
                  >
                    {link.label}
                  </NavLink>
                ))}
                
                {/* Explorer link */}
                <NavLink
                  to="/dashboard"
                  onClick={() => setOpen(false)}
                  className={({ isActive }) =>
                    cn(
                      "flex items-center gap-3 rounded-lg px-4 py-3 text-base font-medium transition-all duration-300",
                      "opacity-0 animate-slide-in-right",
                      isActive
                        ? "bg-primary/10 text-primary"
                        : "text-muted-foreground hover:bg-muted hover:text-foreground"
                    )
                  }
                  style={{ animationDelay: `${navLinks.length * 50}ms`, animationFillMode: 'forwards' }}
                >
                  <LayoutDashboard className="size-5" aria-hidden="true" />
                  Explorer
                </NavLink>

                {/* Divider */}
                <div className="my-4 h-px bg-border/50" aria-hidden="true" />

                {/* CTA buttons */}
                <div 
                  className="flex flex-col gap-3 opacity-0 animate-fade-in-up"
                  style={{ animationDelay: `${(navLinks.length + 1) * 50}ms`, animationFillMode: 'forwards' }}
                >
                  <a
                    href="https://github.com/shiva-manu/NetChain"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="flex h-11 w-full items-center justify-center gap-2 rounded-lg border border-border bg-background text-sm font-medium text-foreground transition-colors hover:bg-muted"
                  >
                    <Github className="size-4" aria-hidden="true" />
                    View on GitHub
                  </a>
                  <Link
                    to="/get-started"
                    onClick={() => setOpen(false)}
                    className="flex h-11 w-full items-center justify-center gap-2 rounded-lg bg-gradient-to-r from-primary to-accent text-sm font-semibold text-primary-foreground transition-opacity hover:opacity-90"
                  >
                    Get Started
                    <ArrowRight className="size-4" aria-hidden="true" />
                  </Link>
                </div>
              </div>
            </SheetContent>
          </Sheet>
        </div>
      </nav>
    </header>
  );
}
