import { BrowserRouter, Routes, Route } from "react-router-dom";
import { AppLayout } from "@/components/layout/AppLayout";
import { VaultGate } from "@/components/layout/VaultGate";
import { VaultProvider } from "@/hooks/useVault";
import Dashboard from "@/pages/Dashboard";
import KeyManager from "@/pages/KeyManager";
import ProviderSettings from "@/pages/ProviderSettings";
import Settings from "@/pages/Settings";
import Usage from "@/pages/Usage";

export default function App() {
  return (
    <VaultProvider>
      <VaultGate>
        <BrowserRouter>
          <Routes>
            <Route element={<AppLayout />}>
              <Route index element={<Dashboard />} />
              <Route path="/keys" element={<KeyManager />} />
              <Route path="/providers" element={<ProviderSettings />} />
              <Route path="/usage" element={<Usage />} />
              <Route path="/settings" element={<Settings />} />
            </Route>
          </Routes>
        </BrowserRouter>
      </VaultGate>
    </VaultProvider>
  );
}
