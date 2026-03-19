import { useEffect } from "react";
import { useLocation } from "react-router-dom";

type SEOProps = {
  title?: string;
  description?: string;
  keywords?: string;
  ogImage?: string;
  canonical?: string;
};

const baseUrl = "https://www.netchain.me";

const defaultSEO = {
  title: "NetChain | Experimental Proof of Internet Layer-1",
  description:
    "NetChain is an experimental Layer-1 blockchain in Rust that blends measured internet performance with stake, identity, reputation, attestation quorum, and slashing history.",
  keywords:
    "NetChain, Proof of Internet, blockchain, Layer-1, Rust blockchain, libp2p, validator telemetry, governance, staking",
  ogImage: `${baseUrl}/og-image.png`,
};

function upsertMeta({
  attribute,
  key,
  content,
}: {
  attribute: "name" | "property";
  key: string;
  content: string;
}) {
  let meta = document.head.querySelector(
    `meta[${attribute}="${key}"]`,
  ) as HTMLMetaElement | null;

  if (!meta) {
    meta = document.createElement("meta");
    meta.setAttribute(attribute, key);
    document.head.appendChild(meta);
  }

  meta.content = content;
}

export function SEO({
  title,
  description,
  keywords,
  ogImage,
  canonical,
}: SEOProps) {
  const location = useLocation();

  const seo = {
    title: title ?? defaultSEO.title,
    description: description ?? defaultSEO.description,
    keywords: keywords ?? defaultSEO.keywords,
    ogImage: ogImage ?? defaultSEO.ogImage,
    canonical: canonical ?? `${baseUrl}${location.pathname}`,
  };

  useEffect(() => {
    document.title = seo.title;

    upsertMeta({ attribute: "name", key: "description", content: seo.description });
    upsertMeta({ attribute: "name", key: "keywords", content: seo.keywords });
    upsertMeta({ attribute: "name", key: "theme-color", content: "#f5f2ea" });

    upsertMeta({ attribute: "property", key: "og:type", content: "website" });
    upsertMeta({ attribute: "property", key: "og:title", content: seo.title });
    upsertMeta({
      attribute: "property",
      key: "og:description",
      content: seo.description,
    });
    upsertMeta({ attribute: "property", key: "og:url", content: seo.canonical });
    upsertMeta({ attribute: "property", key: "og:image", content: seo.ogImage });
    upsertMeta({ attribute: "property", key: "og:site_name", content: "NetChain" });

    upsertMeta({ attribute: "name", key: "twitter:card", content: "summary_large_image" });
    upsertMeta({ attribute: "name", key: "twitter:title", content: seo.title });
    upsertMeta({
      attribute: "name",
      key: "twitter:description",
      content: seo.description,
    });
    upsertMeta({ attribute: "name", key: "twitter:image", content: seo.ogImage });

    let canonicalLink = document.head.querySelector(
      'link[rel="canonical"]',
    ) as HTMLLinkElement | null;

    if (!canonicalLink) {
      canonicalLink = document.createElement("link");
      canonicalLink.rel = "canonical";
      document.head.appendChild(canonicalLink);
    }

    canonicalLink.href = seo.canonical;
  }, [seo.canonical, seo.description, seo.keywords, seo.ogImage, seo.title]);

  return null;
}
