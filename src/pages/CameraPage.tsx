import { useEffect, useRef, useState } from "react";
import { useParams } from "react-router";
import { api } from "../lib/api";
import { isUsableSessionId, parseSessionLookup } from "../lib/sessionContract";
import { SignalingClient } from "../lib/signalingClient";
import {
  videoConstraintsForViewport,
  viewportOrientation,
  type ViewportOrientation,
} from "../lib/cameraOrientation";
import { addRemoteIce, attachLocalStream, createOffer, newPeer, wireIce } from "../lib/webrtc";
import type { CameraSlot } from "../lib/api";
import { BrandMark } from "../components/BrandMark";
import { Banner, Button, Field, StatusChip } from "../components/ui";

type CameraUiState =
  | "loading"
  | "invalid-session"
  | "not-found"
  | "unsupported"
  | "insecure"
  | "ready"
  | "init-error";

function hasMediaDevices(): boolean {
  return Boolean(navigator.mediaDevices?.getUserMedia);
}

function isSecureEnough(): boolean {
  if (window.isSecureContext) return true;
  const host = location.hostname;
  return host === "localhost" || host === "127.0.0.1";
}

function statusTone(status: string): "ready" | "connecting" | "offline" | "warning" {
  const value = status.toLowerCase();
  if (value.includes("connected") || value === "idle") return value === "idle" ? "offline" : "ready";
  if (value.includes("reconnect") || value.includes("connecting")) return "connecting";
  if (value.includes("error") || value.includes("fail")) return "warning";
  return "offline";
}

export function CameraPage() {
  const { sessionId: rawSessionId } = useParams();
  const sessionId = isUsableSessionId(rawSessionId) ? rawSessionId.trim() : "";
  const [pageState, setPageState] = useState<CameraUiState>(sessionId ? "loading" : "invalid-session");
  const [slot, setSlot] = useState<CameraSlot>("cam1");
  const [devices, setDevices] = useState<MediaDeviceInfo[]>([]);
  const [deviceId, setDeviceId] = useState("");
  const [status, setStatus] = useState("Idle");
  const [error, setError] = useState<string | null>(null);
  const [preview, setPreview] = useState<MediaStream | null>(null);
  const [orientation, setOrientation] = useState<ViewportOrientation>(() => viewportOrientation());
  const [slotNames, setSlotNames] = useState<Record<CameraSlot, string>>({
    cam1: "Camera 1",
    cam2: "Camera 2",
  });
  const videoRef = useRef<HTMLVideoElement>(null);
  const streamRef = useRef<MediaStream | null>(null);
  const pcs = useRef(new Map<string, RTCPeerConnection>());
  const signaling = useRef<SignalingClient | null>(null);
  const wakeLock = useRef<WakeLockSentinel | null>(null);
  const deviceIdRef = useRef(deviceId);
  deviceIdRef.current = deviceId;

  useEffect(() => {
    if (videoRef.current) videoRef.current.srcObject = preview;
  }, [preview]);

  useEffect(() => {
    const sync = () => setOrientation(viewportOrientation());
    sync();
    const media = window.matchMedia("(orientation: landscape)");
    const supportsChange = typeof media.addEventListener === "function";
    if (supportsChange) media.addEventListener("change", sync);
    else media.addListener(sync);
    window.addEventListener("orientationchange", sync);
    window.addEventListener("resize", sync);
    return () => {
      if (supportsChange) media.removeEventListener("change", sync);
      else media.removeListener(sync);
      window.removeEventListener("orientationchange", sync);
      window.removeEventListener("resize", sync);
    };
  }, []);

  useEffect(() => {
    if (!sessionId) {
      setPageState("invalid-session");
      return;
    }
    if (!isSecureEnough()) {
      setPageState("insecure");
      return;
    }
    if (!hasMediaDevices()) {
      setPageState("unsupported");
      return;
    }

    let cancelled = false;
    setPageState("loading");
    setError(null);
    void (async () => {
      try {
        const data = await api<unknown>(`/api/sessions/${encodeURIComponent(sessionId)}`);
        const lookup = parseSessionLookup(data);
        if (cancelled) return;
        if (lookup.sessionId !== sessionId) {
          setPageState("init-error");
          setError("Server returned a different session ID than the camera URL.");
          return;
        }
        const nextNames: Record<CameraSlot, string> = { cam1: "Camera 1", cam2: "Camera 2" };
        for (const camera of lookup.cameras) {
          if (camera.role === "cam1" || camera.role === "cam2") {
            nextNames[camera.role] = camera.name;
          }
        }
        setSlotNames(nextNames);
        setPageState("ready");
      } catch (err) {
        if (cancelled) return;
        const message = err instanceof Error ? err.message : String(err);
        console.error("Camera session lookup failed", err);
        if (message.toLowerCase().includes("not found")) {
          setPageState("not-found");
        } else {
          setPageState("init-error");
          setError(message);
        }
      }
    })();

    return () => {
      cancelled = true;
      stopTracks();
      signaling.current?.close();
      for (const pc of pcs.current.values()) pc.close();
      void wakeLock.current?.release();
    };
  }, [sessionId]);

  useEffect(() => {
    signaling.current?.send({ type: "cameraOrientation", orientation });
  }, [orientation]);

  function stopTracks() {
    streamRef.current?.getTracks().forEach((track) => track.stop());
    streamRef.current = null;
    setPreview(null);
  }

  async function listDevices() {
    const all = await navigator.mediaDevices.enumerateDevices();
    setDevices(all.filter((device) => device.kind === "videoinput"));
  }

  async function openStream(nextOrientation: ViewportOrientation): Promise<MediaStream> {
    const video = videoConstraintsForViewport(deviceIdRef.current, nextOrientation);
    try {
      return await navigator.mediaDevices.getUserMedia({ audio: true, video });
    } catch (err) {
      if (!(err instanceof DOMException) || err.name !== "OverconstrainedError") throw err;
      return navigator.mediaDevices.getUserMedia({
        audio: true,
        video: deviceIdRef.current
          ? { deviceId: { exact: deviceIdRef.current } }
          : { facingMode: { ideal: "environment" } },
      });
    }
  }

  async function startCamera() {
    setError(null);
    stopTracks();
    try {
      const stream = await openStream(viewportOrientation());
      streamRef.current = stream;
      setPreview(stream);
      await listDevices();
      connect();
      if ("wakeLock" in navigator) {
        try {
          wakeLock.current = await navigator.wakeLock.request("screen");
        } catch {
          /* optional */
        }
      }
    } catch (err) {
      const name = err instanceof DOMException ? err.name : "Error";
      console.error("Camera start failed", err);
      if (name === "NotAllowedError" || name === "PermissionDeniedError") {
        setError("Camera permission denied. Enable camera and microphone access for this site, then tap Start Camera.");
      } else if (name === "NotFoundError" || name === "OverconstrainedError") {
        setError("Camera unavailable. No matching camera was found on this device.");
      } else if (!isSecureEnough()) {
        setPageState("insecure");
      } else {
        setError(err instanceof Error ? err.message : "Could not start the camera.");
      }
    }
  }

  function connect() {
    if (!sessionId) {
      setPageState("invalid-session");
      return;
    }
    signaling.current?.close();
    const send = (payload: Record<string, unknown>) => signaling.current?.send(payload);
    const client = new SignalingClient({
      onStatus: setStatus,
      onError: (code, message) => {
        console.error("Signaling error", code, message);
        if (code === "unknown-session") {
          setPageState("not-found");
          setError(message);
          return;
        }
        setError(code === "bad-message" ? message : `Signaling connection failed: ${message}`);
      },
      onRegistered: () => {
        signaling.current?.send({ type: "cameraOrientation", orientation: viewportOrientation() });
      },
      onMakeOffer: async (to) => {
        const stream = streamRef.current;
        if (!stream) return;
        pcs.current.get(to)?.close();
        const pc = newPeer();
        pcs.current.set(to, pc);
        attachLocalStream(pc, stream);
        wireIce(pc, send, to);
        pc.onconnectionstatechange = () => setStatus(`WebRTC ${pc.connectionState}`);
        try {
          await createOffer(pc, send, to);
        } catch (err) {
          console.error("Failed to create offer", err);
          setError(err instanceof Error ? err.message : "Failed to create offer");
        }
      },
      onAnswer: async (from, sdp) => {
        const pc = pcs.current.get(from);
        if (pc) await pc.setRemoteDescription({ type: "answer", sdp });
      },
      onIce: async (from, candidate, sdpMid, sdpMLineIndex) => {
        const pc = pcs.current.get(from);
        if (pc) await addRemoteIce(pc, candidate, sdpMid, sdpMLineIndex);
      },
      onPeerLeft: (clientId) => {
        pcs.current.get(clientId)?.close();
        pcs.current.delete(clientId);
      },
    });
    signaling.current = client;
    client.connect({ sessionId, role: "camera", slot });
  }

  const title =
    pageState === "loading"
      ? "Loading"
      : pageState === "invalid-session"
        ? "Invalid session"
        : pageState === "not-found"
          ? "Session not found"
          : pageState === "unsupported"
            ? "Browser unsupported"
            : pageState === "insecure"
              ? "HTTPS/secure-context problem"
              : pageState === "init-error"
                ? "Unexpected initialization error"
                : "Link Studio Camera";

  const detail =
    pageState === "loading"
      ? "Checking this camera session…"
      : pageState === "invalid-session"
        ? "This camera URL has no session ID. Ask the director to start a camera session and scan the new QR code."
        : pageState === "not-found"
          ? "This session does not exist on the studio server. Start a camera session in Director, then scan the QR again."
          : pageState === "unsupported"
            ? "This browser cannot access the camera (getUserMedia is missing). Use Safari on iPhone after trusting the Link Studio CA."
            : pageState === "insecure"
              ? "Camera capture needs a secure context. Open the HTTPS LAN camera URL after installing and fully trusting the local CA."
              : pageState === "init-error"
                ? `${(error ?? "The camera page failed while starting.").replace(/\.$/, "")}. Confirm the studio server is running, then reload this page.`
                : "Choose a camera slot, allow camera access, then start sending to Director.";

  return (
    <main
      className="camera-page"
      data-testid="camera-state"
      data-state={pageState}
      data-orientation={orientation}
    >
      {pageState === "ready" ? (
        <div className="camera-stage">
          <video ref={videoRef} className="local-preview" autoPlay playsInline muted />
        </div>
      ) : null}
      <div className="camera-controls">
        <header>
          <BrandMark size={32} />
          <div>
            <h1>{title}</h1>
          </div>
        </header>
        {detail ? <p className="camera-lede">{detail}</p> : null}
        {pageState === "ready" ? (
          <>
            <StatusChip tone={statusTone(status)}>{status}</StatusChip>
            <Field label="Camera slot">
              <select value={slot} onChange={(event) => setSlot(event.target.value as CameraSlot)}>
                <option value="cam1">{slotNames.cam1}</option>
                <option value="cam2">{slotNames.cam2}</option>
              </select>
            </Field>
            <Field label="Device">
              <select value={deviceId} onChange={(event) => setDeviceId(event.target.value)}>
                <option value="">Back camera (default)</option>
                {devices.map((device) => (
                  <option key={device.deviceId} value={device.deviceId}>
                    {device.label || device.deviceId.slice(0, 8)}
                  </option>
                ))}
              </select>
            </Field>
            {error ? <Banner>{error}</Banner> : null}
            <Button onClick={() => void startCamera()}>Start Camera</Button>
            <p className="hint">Keep this tab in the foreground. Rotate anytime — Director follows landscape and portrait. Wake Lock is requested when the browser supports it.</p>
          </>
        ) : error && pageState !== "init-error" ? (
          <Banner>{error}</Banner>
        ) : null}
      </div>
    </main>
  );
}
