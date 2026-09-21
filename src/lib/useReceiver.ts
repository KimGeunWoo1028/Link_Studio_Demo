import { useEffect, useMemo, useRef, useState } from "react";
import { emptyPgm, type CameraSlot, type PgmState, type Role } from "./api";
import { SignalingClient, type PeerInfo } from "./signalingClient";
import { acceptOffer, addRemoteIce, newPeer, wireIce } from "./webrtc";
import type { ViewportOrientation } from "./cameraOrientation";

export function useReceiver(sessionId: string | undefined, role: Role) {
  const [status, setStatus] = useState("Idle");
  const [error, setError] = useState<string | null>(null);
  const [pgm, setPgm] = useState<PgmState>(emptyPgm());
  const [peers, setPeers] = useState<PeerInfo[]>([]);
  const [streams, setStreams] = useState<Partial<Record<CameraSlot, MediaStream>>>({});
  const [orientations, setOrientations] = useState<Partial<Record<CameraSlot, ViewportOrientation>>>({});
  const pcs = useRef(new Map<string, RTCPeerConnection>());
  const clientToSlot = useRef(new Map<string, CameraSlot>());
  const signaling = useRef<SignalingClient | null>(null);

  const send = (payload: Record<string, unknown>) => signaling.current?.send(payload);

  useEffect(() => {
    if (!sessionId) return;
    const client = new SignalingClient({
      onStatus: setStatus,
      onError: (_code, message) => setError(message),
      onRegistered: (_id, nextPeers, nextPgm) => {
        setPeers(nextPeers);
        setPgm(nextPgm);
        for (const peer of nextPeers) {
          if (peer.role === "camera" && peer.slot) {
            clientToSlot.current.set(peer.clientId, peer.slot);
          }
        }
      },
      onPeerJoined: (peer) => {
        setPeers((current) => [...current.filter((item) => item.clientId !== peer.clientId), peer]);
        if (peer.role === "camera" && peer.slot) {
          clientToSlot.current.set(peer.clientId, peer.slot);
        }
      },
      onPeerLeft: (clientId, _role, slot) => {
        setPeers((current) => current.filter((item) => item.clientId !== clientId));
        const pc = pcs.current.get(clientId);
        pc?.close();
        pcs.current.delete(clientId);
        const mapped = slot ?? clientToSlot.current.get(clientId);
        clientToSlot.current.delete(clientId);
        if (mapped) {
          setStreams((current) => {
            const next = { ...current };
            delete next[mapped];
            return next;
          });
          setOrientations((current) => {
            const next = { ...current };
            delete next[mapped];
            return next;
          });
        }
      },
      onOffer: async (from, sdp) => {
        let pc = pcs.current.get(from);
        if (pc) pc.close();
        pc = newPeer();
        pcs.current.set(from, pc);
        wireIce(pc, send, from);
        pc.ontrack = (event) => {
          const slot = clientToSlot.current.get(from);
          const stream = event.streams[0] ?? new MediaStream([event.track]);
          if (slot) {
            setStreams((current) => ({ ...current, [slot]: stream }));
          }
        };
        pc.onconnectionstatechange = () => {
          if (pc && (pc.connectionState === "failed" || pc.connectionState === "disconnected")) {
            setStatus(`WebRTC ${pc.connectionState}`);
          }
        };
        try {
          await acceptOffer(pc, sdp, send, from);
          setError(null);
        } catch (err) {
          setError(err instanceof Error ? err.message : "WebRTC offer failed");
        }
      },
      onIce: async (from, candidate, sdpMid, sdpMLineIndex) => {
        const pc = pcs.current.get(from);
        if (pc) await addRemoteIce(pc, candidate, sdpMid, sdpMLineIndex);
      },
      onPgm: setPgm,
      onCameraOrientation: (from, slot, orientation) => {
        const mapped = slot ?? clientToSlot.current.get(from);
        if (!mapped) return;
        setOrientations((current) => ({ ...current, [mapped]: orientation }));
      },
    });
    signaling.current = client;
    client.connect({ sessionId, role });
    return () => {
      client.close();
      for (const pc of pcs.current.values()) pc.close();
      pcs.current.clear();
    };
  }, [sessionId, role]);

  const cameras = useMemo(
    () =>
      peers.filter((peer) => peer.role === "camera") as Array<PeerInfo & { slot: CameraSlot }>,
    [peers],
  );

  return { status, error, pgm, setPgm, cameras, streams, orientations, sendPgm: (next: PgmState) => signaling.current?.sendPgm(next) };
}
