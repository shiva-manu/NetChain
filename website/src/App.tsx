import { useEffect } from "react";
import { Navigate, Outlet, Route, Routes, useLocation } from "react-router-dom";

import { Footer } from "@/components/sections/footer";
import { Navbar } from "@/components/sections/navbar";
import { Dashboard } from "@/pages/dashboard";
import { DocsPage } from "@/pages/docs";
import { FeaturesPage } from "@/pages/features";
import { GetStartedPage } from "@/pages/get-started";
import { GovernancePage } from "@/pages/governance";
import { HomePage } from "@/pages/home";
import { HowItWorksPage } from "@/pages/how-it-works";
import { TechnologyPage } from "@/pages/technology";

function RouteEffects() {
  const location = useLocation();

  useEffect(() => {
    window.scrollTo(0, 0);
  }, [location.pathname]);

  return null;
}

function MarketingLayout() {
  return (
    <div className="relative min-h-dvh">
      <a
        href="#main-content"
        className="sr-only focus:not-sr-only focus:fixed focus:left-4 focus:top-4 focus:z-[100] focus:rounded-full focus:bg-card focus:px-4 focus:py-2 focus:text-sm focus:font-semibold focus:text-foreground"
      >
        Skip to main content
      </a>
      <Navbar />
      <main id="main-content">
        <Outlet />
      </main>
      <Footer />
    </div>
  );
}

function App() {
  return (
    <>
      <RouteEffects />
      <Routes>
        <Route element={<MarketingLayout />}>
          <Route index element={<HomePage />} />
          <Route path="features" element={<FeaturesPage />} />
          <Route path="how-it-works" element={<HowItWorksPage />} />
          <Route path="technology" element={<TechnologyPage />} />
          <Route path="governance" element={<GovernancePage />} />
          <Route path="get-started" element={<GetStartedPage />} />
          <Route path="docs" element={<DocsPage />} />
        </Route>
        <Route path="/dashboard" element={<Dashboard />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </>
  );
}

export default App;
