import { useEffect, useRef } from "react";
import type { CameraSlot, PgmState } from "../lib/api";
import { bindDisplayTurn } from "../lib/displayTurn";
import type { ViewportOrientation } from "../lib/cameraOrientation";

interface Props {
  streams: Partial<Record<CameraSlot, MediaStream>>;
  pgm: PgmState;
  orientations?: Partial<Record<CameraSlot, ViewportOrientation>>;
}

export function ProgramStage({ streams, pgm, orientations = {} }: Props) {
  const mainRef = useRef<HTMLVideoElement>(null);
  const pipRef = useRef<HTMLVideoElement>(null);
  const main = pgm.activeSlot ? streams[pgm.activeSlot] : undefined;
  const pip = pgm.pipEnabled && pgm.pipSlot ? streams[pgm.pipSlot] : undefined;
  const mainTurn = pgm.activeSlot ? orientations[pgm.activeSlot] : undefined;
  const pipTurn = pgm.pipSlot ? orientations[pgm.pipSlot] : undefined;

  useEffect(() => {
    const el = mainRef.current;
    if (el) el.srcObject = main ?? null;
    return bindDisplayTurn(el, mainTurn);
  }, [main, mainTurn]);

  useEffect(() => {
    const el = pipRef.current;
    if (el) el.srcObject = pip ?? null;
    return bindDisplayTurn(el, pipTurn);
  }, [pip, pipTurn]);

  return (
    <div className="pgm-stage" data-testid="pgm-stage">
      {main ? (
        <video ref={mainRef} className="pgm-main" autoPlay playsInline muted />
      ) : (
        <div className="pgm-empty">
          <strong>No Program source</strong>
          <span>Take a connected camera to send it to PGM.</span>
        </div>
      )}
      {pip ? <video ref={pipRef} className="pgm-pip" autoPlay playsInline muted /> : null}
      {pgm.captionVisible && pgm.captionText ? (
        <div className="pgm-caption" data-testid="pgm-caption">
          {pgm.captionText}
        </div>
      ) : null}
      {pgm.activeSlot ? <div className="pgm-badge">PGM {pgm.activeSlot.toUpperCase()}</div> : null}
    </div>
  );
}
