export type Role = "director" | "camera" | "program";
export type CameraSlot = "cam1" | "cam2";

export interface PgmState {
  activeSlot: CameraSlot | null;
  pipEnabled: boolean;
  pipSlot: CameraSlot | null;
  captionText: string;
  captionVisible: boolean;
}

export const emptyPgm = (): PgmState => ({
  activeSlot: null,
  pipEnabled: false,
  pipSlot: null,
  captionText: "",
  captionVisible: false,
});

export function apiBase(): string {
  if (location.port === "1420" || location.port === "5173") {
    return "http://127.0.0.1:8787";
  }
  return "";
}

export function wsUrl(): string {
  if (location.port === "1420" || location.port === "5173") {
    return "ws://127.0.0.1:8787/ws";
  }
  const protocol = location.protocol === "https:" ? "wss:" : "ws:";
  return `${protocol}//${location.host}/ws`;
}

export async function api<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${apiBase()}${path}`, {
    headers: { "Content-Type": "application/json", ...(init?.headers ?? {}) },
    ...init,
  });
  if (!response.ok) {
    let message = `${response.status} ${response.statusText}`;
    try {
      const body = (await response.json()) as { error?: string };
      if (body.error) message = body.error;
    } catch {
      /* ignore */
    }
    throw new Error(message);
  }
  if (response.status === 204) {
    return undefined as T;
  }
  return (await response.json()) as T;
}

export function escapeCaption(text: string): string {
  return text.replace(/\s+/g, " ").trim().slice(0, 120);
}
