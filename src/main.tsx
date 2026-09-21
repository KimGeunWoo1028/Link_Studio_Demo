import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter, Route, Routes } from "react-router";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { DirectorPage } from "./pages/DirectorPage";
import { CameraPage } from "./pages/CameraPage";
import { ProgramPage } from "./pages/ProgramPage";
import "./styles.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ErrorBoundary>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<DirectorPage />} />
          <Route path="/camera" element={<CameraPage />} />
          <Route path="/camera/:sessionId" element={<CameraPage />} />
          <Route path="/program/:sessionId" element={<ProgramPage />} />
        </Routes>
      </BrowserRouter>
    </ErrorBoundary>
  </React.StrictMode>,
);
