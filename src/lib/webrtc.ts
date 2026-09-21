import { rtcConfig } from "./signalingClient";

export function attachLocalStream(pc: RTCPeerConnection, stream: MediaStream): void {
  for (const track of stream.getTracks()) {
    pc.addTrack(track, stream);
  }
}

export async function createOffer(
  pc: RTCPeerConnection,
  send: (payload: Record<string, unknown>) => void,
  to: string,
): Promise<void> {
  const offer = await pc.createOffer();
  await pc.setLocalDescription(offer);
  send({ type: "offer", to, sdp: offer.sdp });
}

export async function acceptOffer(
  pc: RTCPeerConnection,
  sdp: string,
  send: (payload: Record<string, unknown>) => void,
  to: string,
): Promise<void> {
  await pc.setRemoteDescription({ type: "offer", sdp });
  const answer = await pc.createAnswer();
  await pc.setLocalDescription(answer);
  send({ type: "answer", to, sdp: answer.sdp });
}

export function wireIce(
  pc: RTCPeerConnection,
  send: (payload: Record<string, unknown>) => void,
  to: string,
): void {
  pc.onicecandidate = (event) => {
    if (!event.candidate) return;
    send({
      type: "ice",
      to,
      candidate: event.candidate.candidate,
      sdpMid: event.candidate.sdpMid,
      sdpMLineIndex: event.candidate.sdpMLineIndex,
    });
  };
}

export async function addRemoteIce(
  pc: RTCPeerConnection,
  candidate: string,
  sdpMid: string | null,
  sdpMLineIndex: number | null,
): Promise<void> {
  if (!candidate) return;
  await pc.addIceCandidate({ candidate, sdpMid, sdpMLineIndex });
}

export function newPeer(): RTCPeerConnection {
  return new RTCPeerConnection(rtcConfig());
}
