import { useEffect } from "react";
import {
  Navigate,
  Outlet,
  Route,
  Routes,
  useLocation,
} from "react-router-dom";
import { ThemeProvider } from "@/components/theme-provider";
import { Navbar } from "@/components/sections/navbar";
import { Footer } from "@/components/sections/footer";
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
      <Navbar />
      <main>
        <Outlet />
      </main>
      <Footer />
    </div>
  );
}

function App() {
  return (
    <ThemeProvider defaultTheme="dark" storageKey="netchain-ui-theme">
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
    </ThemeProvider>
  );
}

export default App;
