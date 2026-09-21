export interface SessionResponse {
  sessionId: string;
  cameraUrl: string;
  programUrl: string;
  programHttpUrl: string;
}

export interface SessionCamera {
  role: string;
  name: string;
}

export interface SessionLookup {
  sessionId: string;
  projectId: string;
  cameras: SessionCamera[];
}

function readString(data: Record<string, unknown>, ...keys: string[]): string | undefined {
  for (const key of keys) {
    const value = data[key];
    if (typeof value === "string" && value.trim().length > 0 && value !== "undefined") {
      return value.trim();
    }
  }
  return undefined;
}

function asObject(data: unknown, label: string): Record<string, unknown> {
  if (!data || typeof data !== "object" || Array.isArray(data)) {
    throw new Error(`${label} was not a JSON object.`);
  }
  return data as Record<string, unknown>;
}

export function parseSessionResponse(data: unknown): SessionResponse {
  const object = asObject(data, "Session response");
  const sessionId = readString(object, "sessionId", "session_id");
  if (!sessionId) {
    throw new Error("Session response missing session ID.");
  }
  return {
    sessionId,
    cameraUrl: readString(object, "cameraUrl", "camera_url") ?? "",
    programUrl: readString(object, "programUrl", "program_url") ?? "",
    programHttpUrl: readString(object, "programHttpUrl", "program_http_url") ?? "",
  };
}

function parseCameras(object: Record<string, unknown>): SessionCamera[] {
  const defaults: SessionCamera[] = [
    { role: "cam1", name: "Camera 1" },
    { role: "cam2", name: "Camera 2" },
  ];
  const raw = object.cameras;
  if (!Array.isArray(raw)) return defaults;
  const cameras = raw.flatMap((item) => {
    if (!item || typeof item !== "object" || Array.isArray(item)) return [];
    const record = item as Record<string, unknown>;
    const role = readString(record, "role");
    const name = readString(record, "name");
    if (!role || !name) return [];
    return [{ role, name }];
  });
  return cameras.length > 0 ? cameras : defaults;
}

export function parseSessionLookup(data: unknown): SessionLookup {
  const object = asObject(data, "Session lookup");
  const sessionId = readString(object, "sessionId", "session_id", "id");
  const projectId = readString(object, "projectId", "project_id");
  if (!sessionId) {
    throw new Error("Session lookup missing session ID.");
  }
  if (!projectId) {
    throw new Error("Session lookup missing project ID.");
  }
  return { sessionId, projectId, cameras: parseCameras(object) };
}

export function isUsableSessionId(value: string | undefined): value is string {
  return Boolean(value && value.trim() && value !== "undefined" && value !== "null");
}
