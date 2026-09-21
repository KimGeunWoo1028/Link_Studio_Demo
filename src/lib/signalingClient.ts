import { emptyPgm, wsUrl, type CameraSlot, type PgmState, type Role } from "./api";

export interface PeerInfo {
  clientId: string;
  role: Role;
  slot: CameraSlot | null;
}

type Handler = {
  onWelcome?: (info: Record<string, unknown>) => void;
  onRegistered?: (clientId: string, peers: PeerInfo[], pgm: PgmState) => void;
  onPeerJoined?: (peer: PeerInfo) => void;
  onPeerLeft?: (clientId: string, role: Role, slot: CameraSlot | null) => void;
  onMakeOffer?: (to: string) => void;
  onOffer?: (from: string, sdp: string) => void;
  onAnswer?: (from: string, sdp: string) => void;
  onIce?: (
    from: string,
    candidate: string,
    sdpMid: string | null,
    sdpMLineIndex: number | null,
  ) => void;
  onPgm?: (pgm: PgmState) => void;
  onCameraOrientation?: (from: string, slot: CameraSlot | null, orientation: "portrait" | "landscape") => void;
  onError?: (code: string, message: string) => void;
  onStatus?: (status: string) => void;
};

export class SignalingClient {
  private ws: WebSocket | null = null;
  private closed = false;
  private retries = 0;
  private registerPayload: Record<string, unknown> | null = null;

  constructor(private readonly handlers: Handler) {}

  connect(register: {
    sessionId: string;
    role: Role;
    slot?: CameraSlot;
  }): void {
    this.registerPayload = {
      type: "register",
      sessionId: register.sessionId,
      role: register.role,
      slot: register.slot ?? null,
    };
    this.open();
  }

  private open(): void {
    this.handlers.onStatus?.(this.retries === 0 ? "Connecting…" : "Reconnecting…");
    const ws = new WebSocket(wsUrl());
    this.ws = ws;
    ws.onopen = () => {
      this.retries = 0;
      this.handlers.onStatus?.("Signaling connected");
    };
    ws.onmessage = (event) => {
      const msg = JSON.parse(String(event.data)) as Record<string, unknown>;
      this.dispatch(msg);
    };
    ws.onclose = () => {
      this.handlers.onStatus?.("Signaling disconnected");
      if (!this.closed) {
        this.retries += 1;
        const delay = Math.min(8000, 500 * 2 ** Math.min(this.retries, 4));
        window.setTimeout(() => this.open(), delay);
      }
    };
    ws.onerror = () => {
      this.handlers.onStatus?.("Signaling error");
      this.handlers.onError?.("signaling", "Signaling connection failed");
    };
  }

  private dispatch(msg: Record<string, unknown>): void {
    const type = String(msg.type ?? "");
    if (type === "welcome") {
      this.handlers.onWelcome?.(msg);
      if (this.registerPayload) this.send(this.registerPayload);
      return;
    }
    if (type === "registered") {
      this.handlers.onRegistered?.(
        String(msg.clientId),
        (msg.peers as PeerInfo[]) ?? [],
        { ...emptyPgm(), ...(msg.pgm as PgmState) },
      );
      return;
    }
    if (type === "peerJoined") {
      this.handlers.onPeerJoined?.(msg.peer as PeerInfo);
      return;
    }
    if (type === "peerLeft") {
      this.handlers.onPeerLeft?.(
        String(msg.clientId),
        msg.role as Role,
        (msg.slot as CameraSlot | null) ?? null,
      );
      return;
    }
    if (type === "makeOffer") {
      this.handlers.onMakeOffer?.(String(msg.to));
      return;
    }
    if (type === "offer") {
      this.handlers.onOffer?.(String(msg.from), String(msg.sdp));
      return;
    }
    if (type === "answer") {
      this.handlers.onAnswer?.(String(msg.from), String(msg.sdp));
      return;
    }
    if (type === "ice") {
      this.handlers.onIce?.(
        String(msg.from),
        String(msg.candidate),
        (msg.sdpMid as string | null) ?? null,
        (msg.sdpMLineIndex as number | null) ?? null,
      );
      return;
    }
    if (type === "pgmState") {
      this.handlers.onPgm?.({ ...emptyPgm(), ...(msg.pgm as PgmState) });
      return;
    }
    if (type === "cameraOrientation") {
      const orientation = msg.orientation === "landscape" ? "landscape" : "portrait";
      this.handlers.onCameraOrientation?.(
        String(msg.from),
        (msg.slot as CameraSlot | null) ?? null,
        orientation,
      );
      return;
    }
    if (type === "error") {
      this.handlers.onError?.(String(msg.code ?? "error"), String(msg.message ?? "Unknown error"));
    }
  }

  send(payload: Record<string, unknown>): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(payload));
    }
  }

  sendPgm(pgm: PgmState): void {
    this.send({ type: "pgmState", pgm });
  }

  close(): void {
    this.closed = true;
    this.ws?.close();
  }
}

export function rtcConfig(): RTCConfiguration {
  return { iceServers: [] };
}
