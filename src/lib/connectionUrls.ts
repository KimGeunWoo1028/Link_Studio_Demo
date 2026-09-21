export type LanKind = "wifi" | "ethernet" | "other";

export interface LanInterface {
  id: string;
  name: string;
  kind: LanKind;
  ipv4: string;
  prefixLen: number | null;
  gateway: string | null;
  physical: boolean;
  up: boolean;
  selectable: boolean;
  recommended: boolean;
  reason?: string;
}

export function cameraHttpsUrl(host: string, httpsPort: number, sessionId: string): string {
  return `https://${host}:${httpsPort}/camera/${sessionId}`;
}

export function iosProfileUrl(host: string, httpPort: number): string {
  return `http://${host}:${httpPort}/ios/link-studio-ca.mobileconfig`;
}

export function iosInstallUrl(host: string, httpPort: number): string {
  return `http://${host}:${httpPort}/install-ca`;
}

export function androidCaUrl(host: string, httpPort: number): string {
  return `http://${host}:${httpPort}/ca.crt`;
}

export function programLocalUrl(httpPort: number, sessionId: string): string {
  return `http://127.0.0.1:${httpPort}/program/${sessionId}`;
}

export function resolveSelectedIpv4(
  interfaces: LanInterface[],
  preferred: string | null | undefined,
): string {
  const selectable = interfaces.filter((item) => item.selectable);
  if (preferred && selectable.some((item) => item.ipv4 === preferred)) {
    return preferred;
  }
  const recommended = selectable.find((item) => item.recommended);
  return recommended?.ipv4 ?? selectable[0]?.ipv4 ?? "";
}

export function otherSelectable(interfaces: LanInterface[], selected: string): LanInterface[] {
  return interfaces.filter((item) => item.selectable && item.ipv4 !== selected);
}

export function unavailableInterfaces(interfaces: LanInterface[]): LanInterface[] {
  return interfaces.filter((item) => !item.selectable);
}
