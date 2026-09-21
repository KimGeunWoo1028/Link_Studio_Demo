import { api, type CameraSlot } from "./api";
import { type LanInterface } from "./connectionUrls";
import { parseSessionLookup, parseSessionResponse, type SessionLookup, type SessionResponse } from "./sessionContract";

export type { SessionLookup, SessionResponse };

export interface Project {
  id: string;
  name: string;
  description: string;
  created_at: string;
  updated_at: string;
}

export interface Camera {
  id: string;
  project_id: string;
  name: string;
  role: CameraSlot | string;
  display_order: number;
}

export interface RuntimeInfo {
  http_base: string;
  https_base: string;
  lan_ips: string[];
  interfaces: LanInterface[];
  recommended_ipv4: string | null;
  bind_http: string;
  bind_https: string;
  ca_url: string;
  ios_profile_url: string;
  android_ca_url: string;
  http_port: number;
  https_port: number;
  active_session_id: string | null;
  last_project_id: string | null;
}

export const studioApi = {
  runtime: () => api<RuntimeInfo>("/api/runtime"),
  projects: () => api<Project[]>("/api/projects"),
  createProject: (name: string, description = "") =>
    api<Project>("/api/projects", {
      method: "POST",
      body: JSON.stringify({ name, description }),
    }),
  cameras: (projectId: string) => api<Camera[]>(`/api/projects/${projectId}/cameras`),
  renameCamera: (id: string, name: string) =>
    api<Camera>(`/api/cameras/${id}`, {
      method: "PATCH",
      body: JSON.stringify({ name }),
    }),
  startSession: async (projectId: string) =>
    parseSessionResponse(
      await api<unknown>(`/api/projects/${projectId}/session`, { method: "POST" }),
    ),
  getSession: async (sessionId: string) =>
    parseSessionLookup(await api<unknown>(`/api/sessions/${encodeURIComponent(sessionId)}`)),
  settings: () => api<Record<string, string>>("/api/settings"),
  saveSettings: (body: { caption_text?: string; pip_enabled?: string; selected_lan_ipv4?: string }) =>
    api<void>("/api/settings", { method: "PUT", body: JSON.stringify(body) }),
  allowLanFirewall: () =>
    api<{ ok: boolean; message: string }>("/api/network/firewall", {
      method: "POST",
      body: JSON.stringify({ confirm: true }),
    }),
};
