import { useEffect, useRef, useState } from "react";
import { StatusChip } from "./ui";
import { bindDisplayTurn } from "../lib/displayTurn";
import type { ViewportOrientation } from "../lib/cameraOrientation";

interface Props {
  stream?: MediaStream;
  slotLabel: string;
  label: string;
  live: boolean;
  selected: boolean;
  connecting?: boolean;
  phoneOrientation?: ViewportOrientation;
  onTake: () => void;
  onRename?: (name: string) => void;
  error?: string;
}

export function CameraPreview({
  stream,
  slotLabel,
  label,
  live,
  selected,
  connecting = false,
  phoneOrientation,
  onTake,
  onRename,
  error,
}: Props) {
  const ref = useRef<HTMLVideoElement>(null);
  const [name, setName] = useState(label);
  useEffect(() => setName(label), [label]);
  useEffect(() => {
    const el = ref.current;
    if (el) el.srcObject = stream ?? null;
    return bindDisplayTurn(el, phoneOrientation);
  }, [stream, phoneOrientation]);

  const tone = selected && live ? "live" : live ? "ready" : connecting ? "connecting" : "offline";
  const statusText = selected && live ? "LIVE" : live ? "READY" : connecting ? "CONNECTING" : "OFFLINE";

  return (
    <article className={`preview-card ${selected ? "is-pgm" : ""}`}>
      <header>
        <span className="cam-id">{slotLabel}</span>
        {onRename ? (
          <input
            className="cam-name"
            aria-label={`${slotLabel} name`}
            value={name}
            onChange={(event) => setName(event.target.value)}
            onBlur={() => {
              const next = name.trim();
              if (next && next !== label) onRename(next);
              else setName(label);
            }}
            onKeyDown={(event) => {
              if (event.key === "Enter") (event.target as HTMLInputElement).blur();
            }}
          />
        ) : (
          <strong className="cam-name">{label}</strong>
        )}
        <StatusChip tone={tone}>{statusText}</StatusChip>
      </header>
      <button type="button" className="preview-hit" onClick={onTake} aria-label={`Take ${label}`}>
        {stream ? (
          <video ref={ref} autoPlay playsInline muted />
        ) : (
          <div className="preview-empty">{error ?? "Waiting for camera"}</div>
        )}
      </button>
      <button type="button" className={`btn btn-take ${selected ? "is-live" : ""}`} onClick={onTake}>
        {selected ? "TAKE · ON AIR" : `TAKE ${slotLabel}`}
      </button>
    </article>
  );
}
