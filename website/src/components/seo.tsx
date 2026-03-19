import { useEffect } from 'react';
import { useLocation } from 'react-router-dom';

interface SEOProps {
  title?: string;
  description?: string;
  keywords?: string;
  ogImage?: string;
  canonical?: string;
}

const defaultSEO = {
  title: 'NetChain - Proof of Internet Blockchain | Layer-1 Network Performance Protocol',
  description: 'NetChain is a revolutionary Layer-1 blockchain powered by Proof of Internet (PoI). Validator selection based on real network performance metrics - download speed, upload speed, latency, and uptime. Join the future of decentralized networks.',
  keywords: 'NetChain, Proof of Internet, PoI, blockchain, Layer-1, cryptocurrency, validator, network performance, decentralized, distributed ledger, consensus mechanism, DeFi, Web3',
  ogImage: 'https://www.netchain.me/og-image.png',
};

export function SEO({ title, description, keywords, ogImage, canonical }: SEOProps) {
  const location = useLocation();
  const baseUrl = 'https://www.netchain.me';
  
  const seo = {
    title: title || defaultSEO.title,
    description: description || defaultSEO.description,
    keywords: keywords || defaultSEO.keywords,
    ogImage: ogImage || defaultSEO.ogImage,
    canonical: canonical || `${baseUrl}${location.pathname}`,
  };

  useEffect(() => {
    // Update title
    document.title = seo.title;

    // Update or create meta tags
    const updateMeta = (name: string, content: string, property = false) => {
      const attr = property ? 'property' : 'name';
      let meta = document.querySelector(`meta[${attr}="${name}"]`) as HTMLMetaElement;
      
      if (!meta) {
        meta = document.createElement('meta');
        meta.setAttribute(attr, name);
        document.head.appendChild(meta);
      }
      
      meta.content = content;
    };

    // Standard meta tags
    updateMeta('description', seo.description);
    updateMeta('keywords', seo.keywords);

    // Open Graph tags
    updateMeta('og:title', seo.title, true);
    updateMeta('og:description', seo.description, true);
    updateMeta('og:url', seo.canonical, true);
    updateMeta('og:image', seo.ogImage, true);

    // Twitter tags
    updateMeta('twitter:title', seo.title, true);
    updateMeta('twitter:description', seo.description, true);
    updateMeta('twitter:image', seo.ogImage, true);

    // Update canonical link
    let canonicalLink = document.querySelector('link[rel="canonical"]') as HTMLLinkElement;
    
    if (!canonicalLink) {
      canonicalLink = document.createElement('link');
      canonicalLink.rel = 'canonical';
      document.head.appendChild(canonicalLink);
    }
    
    canonicalLink.href = seo.canonical;

  }, [seo.title, seo.description, seo.keywords, seo.ogImage, seo.canonical]);

  return null;
}
