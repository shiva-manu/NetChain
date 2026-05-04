import { useEffect } from "react";
import { Navigate, Outlet, Route, Routes, useLocation } from "react-router-dom";

import { Navbar } from "@/components/sections/navbar";
import { Footer } from "@/components/sections/footer";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Dashboard } from "@/pages/dashboard";
import { DocsPage } from "@/pages/docs";
import { FaucetPage } from "@/pages/faucet";
import { FeaturesPage } from "@/pages/features";
import { GetStartedPage } from "@/pages/get-started";
import { HomePage } from "@/pages/home";
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
    <div className="relative min-h-dvh overflow-x-hidden">
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
    <TooltipProvider>
      <RouteEffects />
      <Routes>
        <Route element={<MarketingLayout />}>
          <Route index element={<HomePage />} />
          <Route path="features" element={<FeaturesPage />} />
          <Route path="technology" element={<TechnologyPage />} />
          <Route path="docs" element={<DocsPage />} />
          <Route path="faucet" element={<FaucetPage />} />
          <Route path="get-started" element={<GetStartedPage />} />
          <Route path="dashboard" element={<Dashboard />} />
        </Route>
        {/* Redirect old routes */}
        <Route path="/how-it-works" element={<Navigate to="/" replace />} />
        <Route path="/governance" element={<Navigate to="/features" replace />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </TooltipProvider>
  );
}

export default App;
