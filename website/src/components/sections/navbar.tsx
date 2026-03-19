import { useEffect, useState } from "react";
import { ArrowUpRight, Github, LayoutDashboard, Menu, X } from "lucide-react";
import { Link, NavLink, type NavLinkRenderProps } from "react-router-dom";

import { Sheet, SheetContent, SheetTitle, SheetTrigger } from "@/components/ui/sheet";
import { REPOSITORY_URL, siteNavigation } from "@/content/site";
import { cn } from "@/lib/utils";

function navLinkClassName({ isActive }: NavLinkRenderProps) {
  return cn(
    "rounded-full px-4 py-2 text-sm font-semibold transition-colors",
    isActive
      ? "bg-secondary text-foreground"
      : "text-muted-foreground hover:bg-secondary/70 hover:text-foreground",
  );
}

function BrandMark() {
  return (
    <div className="relative flex size-11 items-center justify-center rounded-2xl border border-border/80 bg-card shadow-[0_18px_40px_-24px_rgba(15,23,42,0.35)]">
      <div
        className="absolute inset-1 rounded-xl border border-primary/18 bg-[radial-gradient(circle_at_top,_color-mix(in_oklab,var(--accent)_28%,transparent),_transparent_62%)]"
        aria-hidden="true"
      />
      <div className="relative grid grid-cols-2 gap-1">
        <span className="size-2.5 rounded-full bg-primary" />
        <span className="size-2.5 rounded-full bg-accent" />
        <span className="size-2.5 rounded-full bg-foreground/18" />
        <span className="size-2.5 rounded-full bg-primary/35" />
      </div>
    </div>
  );
}

export function Navbar() {
  const [open, setOpen] = useState(false);
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const handleScroll = () => {
      setScrolled(window.scrollY > 12);
    };

    handleScroll();
    window.addEventListener("scroll", handleScroll, { passive: true });

    return () => {
      window.removeEventListener("scroll", handleScroll);
    };
  }, []);

  return (
    <header
      className={cn(
        "sticky top-0 z-50 border-b border-transparent transition-colors",
        (open || scrolled) &&
          "border-border/70 bg-background/88 backdrop-blur-xl supports-[backdrop-filter]:bg-background/78",
      )}
    >
      <nav className="container-site flex h-20 items-center justify-between gap-6" aria-label="Primary navigation">
        <Link to="/" className="flex items-center gap-4" aria-label="NetChain home">
          <BrandMark />
          <div className="min-w-0">
            <p className="font-heading text-2xl leading-none text-foreground">NetChain</p>
            <p className="mt-1 text-[0.7rem] font-semibold uppercase tracking-[0.24em] text-muted-foreground">
              Proof of Internet
            </p>
          </div>
        </Link>

        <div className="hidden items-center gap-1 lg:flex">
          {siteNavigation.map((item) => (
            <NavLink key={item.to} to={item.to} className={navLinkClassName}>
              {item.label}
            </NavLink>
          ))}
        </div>

        <div className="hidden items-center gap-3 lg:flex">
          <a
            href={REPOSITORY_URL}
            target="_blank"
            rel="noreferrer noopener"
            className="inline-flex items-center gap-2 rounded-full border border-border bg-card/90 px-4 py-2 text-sm font-semibold text-foreground transition-colors hover:bg-card"
          >
            <Github className="size-4" aria-hidden="true" />
            GitHub
            <ArrowUpRight className="size-4" aria-hidden="true" />
          </a>
          <NavLink
            to="/dashboard"
            className={({ isActive }) =>
              cn(
                "inline-flex items-center gap-2 rounded-full px-4 py-2 text-sm font-semibold transition-colors",
                isActive
                  ? "bg-primary text-primary-foreground"
                  : "bg-primary text-primary-foreground hover:bg-primary/90",
              )
            }
          >
            <LayoutDashboard className="size-4" aria-hidden="true" />
            Explorer
          </NavLink>
        </div>

        <div className="flex items-center lg:hidden">
          <Sheet open={open} onOpenChange={setOpen}>
            <SheetTrigger
              className="inline-flex size-11 items-center justify-center rounded-full border border-border bg-card text-foreground transition-colors hover:bg-secondary"
              aria-label={open ? "Close navigation menu" : "Open navigation menu"}
            >
              {open ? <X className="size-5" aria-hidden="true" /> : <Menu className="size-5" aria-hidden="true" />}
            </SheetTrigger>
            <SheetContent
              side="right"
              className="w-[min(26rem,100vw)] overscroll-contain border-border bg-background/96 px-6 pt-6 backdrop-blur-2xl"
            >
              <SheetTitle className="sr-only">Navigation</SheetTitle>
              <div className="flex items-center gap-4">
                <BrandMark />
                <div>
                  <p className="font-heading text-2xl text-foreground">NetChain</p>
                  <p className="mt-1 text-[0.72rem] font-semibold uppercase tracking-[0.24em] text-muted-foreground">
                    Proof of Internet
                  </p>
                </div>
              </div>

              <div className="story-rule my-6" aria-hidden="true" />

              <div className="flex flex-col gap-2">
                {siteNavigation.map((item) => (
                  <NavLink
                    key={item.to}
                    to={item.to}
                    onClick={() => setOpen(false)}
                    className={({ isActive }) =>
                      cn(
                        "rounded-2xl border px-4 py-3 text-base font-semibold transition-colors",
                        isActive
                          ? "border-primary/20 bg-secondary text-foreground"
                          : "border-border/70 bg-card/90 text-foreground hover:bg-secondary/80",
                      )
                    }
                  >
                    {item.label}
                  </NavLink>
                ))}
              </div>

              <div className="story-rule my-6" aria-hidden="true" />

              <div className="grid gap-3">
                <NavLink
                  to="/dashboard"
                  onClick={() => setOpen(false)}
                  className="inline-flex items-center justify-between rounded-2xl bg-primary px-4 py-3 text-base font-semibold text-primary-foreground transition-colors hover:bg-primary/90"
                >
                  Explorer
                  <LayoutDashboard className="size-4" aria-hidden="true" />
                </NavLink>
                <a
                  href={REPOSITORY_URL}
                  target="_blank"
                  rel="noreferrer noopener"
                  className="inline-flex items-center justify-between rounded-2xl border border-border bg-card/90 px-4 py-3 text-base font-semibold text-foreground transition-colors hover:bg-card"
                >
                  GitHub Repository
                  <ArrowUpRight className="size-4" aria-hidden="true" />
                </a>
              </div>
            </SheetContent>
          </Sheet>
        </div>
      </nav>
    </header>
  );
}
