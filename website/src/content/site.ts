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
  { label: "Technology", to: "/technology" },
  { label: "Docs", to: "/docs" },
  { label: "Faucet", to: "/faucet" },
  { label: "Explorer", to: "/dashboard" },
];

export const footerGroups: FooterGroup[] = [
  {
    title: "Explore",
    links: [
      { label: "Homepage", to: "/" },
      { label: "Features", to: "/features" },
      { label: "Technology", to: "/technology" },
      { label: "Faucet", to: "/faucet" },
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
