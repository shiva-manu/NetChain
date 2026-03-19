export type SiteNavItem = {
  label: string;
  to: string;
};

export type FooterLink = {
  label: string;
  to?: string;
  href?: string;
};

export type FooterGroup = {
  title: string;
  links: FooterLink[];
};

export const REPOSITORY_URL = "https://github.com/shiva-manu/NetChain";

export const siteNavigation: SiteNavItem[] = [
  { label: "Features", to: "/features" },
  { label: "How It Works", to: "/how-it-works" },
  { label: "Technology", to: "/technology" },
  { label: "Governance", to: "/governance" },
  { label: "Docs", to: "/docs" },
  { label: "Get Started", to: "/get-started" },
];

export const footerGroups: FooterGroup[] = [
  {
    title: "Explore",
    links: [
      { label: "Homepage", to: "/" },
      { label: "Features", to: "/features" },
      { label: "Technology", to: "/technology" },
      { label: "Governance", to: "/governance" },
    ],
  },
  {
    title: "Build",
    links: [
      { label: "Docs", to: "/docs" },
      { label: "Get Started", to: "/get-started" },
      { label: "Explorer", to: "/dashboard" },
      { label: "GitHub Repository", href: REPOSITORY_URL },
    ],
  },
];
