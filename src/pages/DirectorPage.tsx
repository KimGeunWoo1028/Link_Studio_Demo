import { FormEvent, useEffect, useMemo, useRef, useState } from "react";
import { QRCodeSVG } from "qrcode.react";
import { emptyPgm, escapeCaption, type CameraSlot, type PgmState } from "../lib/api";
import { studioApi, type Camera, type Project, type RuntimeInfo, type SessionResponse } from "../lib/studioApi";
import {
  androidCaUrl,
  cameraHttpsUrl,
  iosInstallUrl,
  iosProfileUrl,
  otherSelectable,
  programLocalUrl,
  resolveSelectedIpv4,
  unavailableInterfaces,
  type LanInterface,
} from "../lib/connectionUrls";
import { useReceiver } from "../lib/useReceiver";
import { BrandMark, IconChevron, IconCopy, IconEthernet, IconWifi } from "../components/BrandMark";
import { CameraPreview } from "../components/CameraPreview";
import { ProgramStage } from "../components/ProgramStage";
import { Banner, Button, Field, Panel, StatusChip } from "../components/ui";

function signalingTone(status: string): "ready" | "connecting" | "offline" | "warning" {
  const value = status.toLowerCase();
  if (value.includes("connected") && !value.includes("disconnected")) return "ready";
  if (value.includes("reconnect") || value.includes("connecting")) return "connecting";
  if (value.includes("error")) return "warning";
  return "offline";
}

export function DirectorPage() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [projectId, setProjectId] = useState("");
  const [camerasMeta, setCamerasMeta] = useState<Camera[]>([]);
  const [runtime, setRuntime] = useState<RuntimeInfo | null>(null);
  const [session, setSession] = useState<SessionResponse | null>(null);
  const [name, setName] = useState("Demo Project");
  const [bootError, setBootError] = useState<string | null>(null);
  const [captionDraft, setCaptionDraft] = useState("");
  const [selectedIp, setSelectedIp] = useState("");
  const [firewallMessage, setFirewallMessage] = useState<string | null>(null);
  const [copied, setCopied] = useState<string | null>(null);
  const [controlsOpen, setControlsOpen] = useState(true);
  const [connectionOpen, setConnectionOpen] = useState(true);
  const prevLiveRef = useRef(0);

  const sessionId = session?.sessionId;
  const receiver = useReceiver(sessionId, "director");

  useEffect(() => {
    void (async () => {
      try {
        const [nextRuntime, nextProjects, settings] = await Promise.all([
          studioApi.runtime(),
          studioApi.projects(),
          studioApi.settings(),
        ]);
        setRuntime(nextRuntime);
        setProjects(nextProjects);
        const selected = settings.last_project_id || nextRuntime.last_project_id || nextProjects[0]?.id || "";
        setProjectId(selected);
        if (settings.caption_text) setCaptionDraft(settings.caption_text);
        const savedIp = settings.selected_lan_ipv4 || "";
        setSelectedIp(resolveSelectedIpv4(nextRuntime.interfaces ?? [], savedIp));
      } catch (err) {
        setBootError(err instanceof Error ? err.message : "Failed to load studio API. Is the local server running?");
      }
    })();
  }, []);

  useEffect(() => {
    if (!projectId) return;
    void studioApi.cameras(projectId).then(setCamerasMeta).catch((err: Error) => setBootError(err.message));
  }, [projectId]);

  useEffect(() => {
    const timer = window.setInterval(() => {
      void studioApi.runtime().then((next) => {
        setRuntime(next);
        setSelectedIp((current) => resolveSelectedIpv4(next.interfaces ?? [], current));
      }).catch(() => {
        /* keep last known runtime */
      });
    }, 4000);
    return () => window.clearInterval(timer);
  }, []);

  async function onCreate(event: FormEvent) {
    event.preventDefault();
    const project = await studioApi.createProject(name);
    setProjects((current) => [project, ...current]);
    setProjectId(project.id);
  }

  async function onStartSession() {
    if (!projectId) return;
    try {
      setBootError(null);
      const started = await studioApi.startSession(projectId);
      if (!started.sessionId) {
        throw new Error("Session response missing session ID.");
      }
      setSession(started);
      setConnectionOpen(true);
    } catch (err) {
      setSession(null);
      setBootError(err instanceof Error ? err.message : "Could not start camera session.");
    }
  }

  function take(slot: CameraSlot) {
    const next: PgmState = { ...receiver.pgm, activeSlot: slot };
    receiver.sendPgm(next);
  }

  function setPip(enabled: boolean, slot?: CameraSlot) {
    const next: PgmState = {
      ...receiver.pgm,
      pipEnabled: enabled,
      pipSlot: slot ?? receiver.pgm.pipSlot ?? "cam2",
    };
    receiver.sendPgm(next);
    void studioApi.saveSettings({ pip_enabled: String(enabled) });
  }

  function showCaption(visible: boolean) {
    const next: PgmState = {
      ...receiver.pgm,
      captionText: escapeCaption(captionDraft),
      captionVisible: visible,
    };
    receiver.sendPgm(next);
    void studioApi.saveSettings({ caption_text: next.captionText });
  }

  async function rename(camera: Camera, nextName: string) {
    const updated = await studioApi.renameCamera(camera.id, nextName);
    setCamerasMeta((current) => current.map((item) => (item.id === updated.id ? updated : item)));
  }

  const labels = useMemo(() => {
    const map: Record<string, string> = { cam1: "Camera 1", cam2: "Camera 2" };
    for (const camera of camerasMeta) map[camera.role] = camera.name;
    return map;
  }, [camerasMeta]);

  const pgm = sessionId ? receiver.pgm : emptyPgm();
  const interfaces = runtime?.interfaces ?? [];
  const cameraUrl =
    sessionId && selectedIp ? cameraHttpsUrl(selectedIp, runtime?.https_port ?? 8443, sessionId) : "";
  const programUrl = sessionId ? programLocalUrl(runtime?.http_port ?? 8787, sessionId) : "";
  const others = otherSelectable(interfaces, selectedIp);
  const unavailable = unavailableInterfaces(interfaces);
  const selectedIface = interfaces.find((item) => item.ipv4 === selectedIp);
  const activeProject = projects.find((item) => item.id === projectId);
  const liveCount = (["cam1", "cam2"] as CameraSlot[]).filter((slot) => receiver.streams[slot]).length;
  const cameraCountLabel =
    liveCount === 1 ? "1 camera connected" : `${liveCount} cameras connected`;

  useEffect(() => {
    if (liveCount >= 2 && prevLiveRef.current < 2) {
      setConnectionOpen(false);
    }
    prevLiveRef.current = liveCount;
  }, [liveCount]);

  function onSelectNetwork(ipv4: string) {
    setSelectedIp(ipv4);
    void studioApi.saveSettings({ selected_lan_ipv4: ipv4 });
  }

  async function onAllowFirewall() {
    const ok = window.confirm(
      "Add Windows Firewall inbound allow rules for TCP 8787 and 8443 on Private/Domain networks? This does not turn the firewall off.",
    );
    if (!ok) return;
    try {
      const result = await studioApi.allowLanFirewall();
      setFirewallMessage(result.message);
    } catch (err) {
      setFirewallMessage(err instanceof Error ? err.message : "Could not add firewall rules. See README.");
    }
  }

  function kindLabel(kind: LanInterface["kind"]): string {
    if (kind === "wifi") return "Wi-Fi";
    if (kind === "ethernet") return "Ethernet";
    return "Adapter";
  }

  async function copy(label: string, value: string) {
    if (!value) return;
    await navigator.clipboard.writeText(value);
    setCopied(label);
    window.setTimeout(() => setCopied(null), 1600);
  }

  return (
    <div className={`director ${connectionOpen ? "is-setup" : ""}`}>
      <header className="chrome">
        <div className="chrome-row">
        <div className="chrome-brand">
          <BrandMark size={24} />
          <div>
            <strong>Link Studio</strong>
            <span>Director</span>
          </div>
        </div>
        <form className="chrome-project" onSubmit={onCreate}>
          <Field label="Project">
            <select value={projectId} onChange={(event) => setProjectId(event.target.value)} aria-label="Active project">
              <option value="">Select…</option>
              {projects.map((project) => (
                <option key={project.id} value={project.id}>
                  {project.name}
                </option>
              ))}
            </select>
          </Field>
          <details className="chrome-create">
            <summary>New project</summary>
            <input
              value={name}
              onChange={(event) => setName(event.target.value)}
              placeholder="Project name"
              aria-label="Project name"
            />
            <Button type="submit" variant="ghost">
              Create
            </Button>
          </details>
          <Button type="button" variant="ghost" onClick={() => void onStartSession()} disabled={!projectId}>
            Start session
          </Button>
          <Button
            type="button"
            variant={connectionOpen ? "ghost" : "primary"}
            aria-expanded={connectionOpen}
            aria-controls="camera-connection"
            disabled={!sessionId}
            onClick={() => setConnectionOpen((open) => !open)}
          >
            + Add Camera
          </Button>
        </form>
        <div className="chrome-status">
          <div className="status-unit">
            <span className="k">Session</span>
            <span className="v">
              <StatusChip tone={sessionId ? "ready" : "offline"}>{sessionId ? "Active" : "Idle"}</StatusChip>
            </span>
          </div>
          <div className="status-unit">
            <span className="k">Network</span>
            <span className="v">
              {selectedIface?.kind === "ethernet" ? <IconEthernet /> : <IconWifi />}
              {selectedIp ? `${kindLabel(selectedIface?.kind ?? "other")} · ${selectedIp}` : "No LAN"}
            </span>
          </div>
          <div className="status-unit">
            <span className="k">Signaling</span>
            <span className="v">
              <StatusChip tone={signalingTone(receiver.status)}>{receiver.status}</StatusChip>
            </span>
          </div>
        </div>
        </div>
        {bootError || receiver.error ? (
          <Banner>
            {bootError ?? receiver.error}
            {bootError?.includes("local server") ? " Start the studio server, then reload." : ""}
          </Banner>
        ) : null}
      </header>

      <section
        id="camera-connection"
        className="connection-stage"
        aria-label="Camera connection"
        hidden={!connectionOpen}
      >
        <div className="connection-stage__head">
          <div className="connection-stage__title">
            <h2>Add camera</h2>
            <p className="hint">Trust the studio CA on the phone, pick this PC’s network, then scan the QR.</p>
          </div>
          <StatusChip tone={liveCount > 0 ? "ready" : "offline"}>{cameraCountLabel}</StatusChip>
          <Button type="button" variant="ghost" onClick={() => setConnectionOpen(false)}>
            Back to Director
          </Button>
        </div>
        <div className="connect-slots" aria-label="Camera status">
          {(["cam1", "cam2"] as CameraSlot[]).map((slot, index) => {
            const live = Boolean(receiver.streams[slot]);
            return (
              <div key={slot} className={live ? "connect-slot is-live" : "connect-slot"}>
                <span className="cam-id">CAM {index + 1}</span>
                <span className="connect-slot__name">{labels[slot]}</span>
                <StatusChip tone={live ? "live" : "offline"}>{live ? "Live" : "Waiting"}</StatusChip>
              </div>
            );
          })}
        </div>
        <div id="add-camera-body" className="connect-steps">
          <div className="step" data-testid="trust-ca">
            <span className="step-index">1. Trust this studio</span>
            <p className="hint">On the phone, install the CA over HTTP before opening the HTTPS camera URL.</p>
            <Field label="iPhone">
              <div className="url-row">
                <code className="mono" data-testid="ios-profile-url">
                  {selectedIp ? iosInstallUrl(selectedIp, runtime?.http_port ?? 8787) : "Select a live network"}
                </code>
                <Button
                  variant="ghost"
                  onClick={() => void copy("ios", selectedIp ? iosInstallUrl(selectedIp, runtime?.http_port ?? 8787) : "")}
                  disabled={!selectedIp}
                >
                  {copied === "ios" ? "Copied" : "Copy"}
                </Button>
              </div>
            </Field>
            {selectedIp ? (
              <a className="trust-link" href={iosInstallUrl(selectedIp, runtime?.http_port ?? 8787)}>
                Install iPhone profile
              </a>
            ) : null}
            <p className="hint">Then Settings → General → About → Certificate Trust Settings.</p>
            <Field label="Android">
              <div className="url-row">
                <code className="mono" data-testid="android-ca-url">
                  {selectedIp ? androidCaUrl(selectedIp, runtime?.http_port ?? 8787) : "Select a live network"}
                </code>
                <Button
                  variant="ghost"
                  onClick={() => void copy("android", selectedIp ? androidCaUrl(selectedIp, runtime?.http_port ?? 8787) : "")}
                  disabled={!selectedIp}
                >
                  {copied === "android" ? "Copied" : "Copy"}
                </Button>
              </div>
            </Field>
          </div>
          <div className="step">
            <span className="step-index">2. Select network</span>
            <select
              data-testid="connection-network"
              value={selectedIp}
              onChange={(event) => onSelectNetwork(event.target.value)}
              disabled={interfaces.filter((item) => item.selectable).length === 0}
              aria-label="Connection network"
            >
              {interfaces
                .filter((item) => item.selectable)
                .map((item) => (
                  <option key={item.id} value={item.ipv4}>
                    {kindLabel(item.kind)} — {item.ipv4}
                    {item.recommended ? " (recommended)" : ""}
                  </option>
                ))}
            </select>
            {selectedIp ? (
              <p className="hint">
                {kindLabel(selectedIface?.kind ?? "other")} {selectedIp}. Changing this does not create a new session.
              </p>
            ) : (
              <p className="error">No live LAN adapter is available. Plug in Ethernet or connect Wi-Fi, then wait a few seconds.</p>
            )}
          </div>
          <div className="step step--qr">
            <span className="step-index">3. Scan QR</span>
            {cameraUrl ? (
              <div className="qr">
                <QRCodeSVG value={cameraUrl} size={240} marginSize={3} />
              </div>
            ) : (
              <>
                <div className="qr-empty">Start a camera session to generate a QR code.</div>
                <Button
                  data-testid="qr-start-session"
                  type="button"
                  onClick={() => void onStartSession()}
                  disabled={!projectId}
                >
                  Start camera session
                </Button>
              </>
            )}
            <div className="url-row">
              <code className="mono" data-testid="camera-url">
                {cameraUrl || "Start a session to generate a URL"}
              </code>
              <Button variant="ghost" onClick={() => void copy("camera", cameraUrl)} disabled={!cameraUrl} aria-label="Copy camera URL">
                <IconCopy />
                {copied === "camera" ? "Copied" : "Copy"}
              </Button>
            </div>
            <p className="hint">Open this URL on the phone after trusting the local CA.</p>
          </div>
        </div>
        <details className="advanced">
          <summary>Advanced setup</summary>
          <div className="stack">
            {selectedIp ? (
              <p className="hint">Profile file: {iosProfileUrl(selectedIp, runtime?.http_port ?? 8787)}</p>
            ) : null}
            {others.length > 0 ? (
              <div>
                <p className="hint">Other available network:</p>
                <ul>
                  {others.map((item) => (
                    <li key={item.id}>
                      {kindLabel(item.kind)} — {item.ipv4}
                    </li>
                  ))}
                </ul>
              </div>
            ) : null}
            {unavailable.length > 0 ? (
              <div>
                <p className="hint">Not used for QR (disconnected, virtual, or link-local):</p>
                <ul>
                  {unavailable.map((item) => (
                    <li key={item.id}>
                      {item.name} — {item.ipv4}
                      {item.reason ? ` (${item.reason})` : ""}
                    </li>
                  ))}
                </ul>
              </div>
            ) : null}
            <p className="hint">PGM (same PC / OBS Browser Source) stays on localhost:</p>
            <code className="mono" data-testid="program-url">
              {programUrl || "—"}
            </code>
            <Button variant="ghost" onClick={() => void onAllowFirewall()}>
              Allow LAN inbound (Windows Firewall)
            </Button>
            {firewallMessage ? <p className="hint">{firewallMessage}</p> : null}
            <p className="hint">
              Server binds 0.0.0.0:{runtime?.http_port ?? 8787} and 0.0.0.0:{runtime?.https_port ?? 8443}. If a live Ethernet IP still fails from the phone, set that adapter to a Private network profile and add the inbound rules above — do not turn the firewall off.
            </p>
          </div>
        </details>
      </section>

      <div
        className={`workspace ${connectionOpen ? "workspace--docked" : ""}`}
        data-testid="director-workspace"
      >
        <div className="preview-grid">
          {(["cam1", "cam2"] as CameraSlot[]).map((slot, index) => {
            const camera = camerasMeta.find((item) => item.role === slot);
            const live = Boolean(receiver.streams[slot]);
            return (
              <CameraPreview
                key={slot}
                slotLabel={`CAM ${index + 1}`}
                label={labels[slot]}
                stream={receiver.streams[slot]}
                phoneOrientation={receiver.orientations[slot]}
                live={live}
                connecting={Boolean(sessionId) && !live && signalingTone(receiver.status) === "connecting"}
                selected={pgm.activeSlot === slot}
                onTake={() => take(slot)}
                onRename={camera ? (next) => void rename(camera, next) : undefined}
              />
            );
          })}
        </div>

        <div className={`pgm-row ${controlsOpen ? "" : "is-collapsed"}`}>
          <section className="pgm-well" aria-label="Program monitor">
            <div className="pgm-well__bar">
              <h2>PROGRAM</h2>
              <StatusChip tone={pgm.activeSlot ? "live" : "offline"}>
                {pgm.activeSlot ? "PGM LIVE" : "NO SOURCE"}
              </StatusChip>
            </div>
            <ProgramStage streams={receiver.streams} pgm={pgm} orientations={receiver.orientations} />
          </section>

          <aside className={`control-rail ${controlsOpen ? "" : "is-collapsed"}`}>
            <button
              type="button"
              className="collapse-btn"
              aria-expanded={controlsOpen}
              aria-controls="production-controls"
              onClick={() => setControlsOpen((open) => !open)}
            >
              <IconChevron dir={controlsOpen ? "right" : "left"} />
              <span>{controlsOpen ? "Hide controls" : "Controls"}</span>
            </button>
            <div id="production-controls" className="control-rail__body" hidden={!controlsOpen}>
            <Panel
              title="PIP"
              actions={<StatusChip tone={pgm.pipEnabled ? "ready" : "offline"}>{pgm.pipEnabled ? "On" : "Off"}</StatusChip>}
            >
              <label className="row">
                <input
                  type="checkbox"
                  checked={pgm.pipEnabled}
                  onChange={(event) => setPip(event.target.checked)}
                />
                Picture-in-picture
              </label>
              <Field label="PIP camera">
                <select
                  value={pgm.pipSlot ?? "cam2"}
                  onChange={(event) => setPip(pgm.pipEnabled, event.target.value as CameraSlot)}
                >
                  <option value="cam1">{labels.cam1}</option>
                  <option value="cam2">{labels.cam2}</option>
                </select>
              </Field>
            </Panel>
            <Panel
              title="Caption"
              actions={
                <StatusChip tone={pgm.captionVisible ? "ready" : "offline"}>
                  {pgm.captionVisible ? "Shown" : "Hidden"}
                </StatusChip>
              }
            >
              <Field label="Lower third">
                <input
                  value={captionDraft}
                  onChange={(event) => setCaptionDraft(event.target.value)}
                  placeholder="Lower-third text"
                />
              </Field>
              <div className="row">
                <Button variant="show" onClick={() => showCaption(true)}>
                  SHOW
                </Button>
                <Button variant="hide" onClick={() => showCaption(false)}>
                  HIDE
                </Button>
              </div>
            </Panel>
            <p className="hint">
              {activeProject ? `Project context: ${activeProject.name}.` : "Create or select a project, then start a session."}
            </p>
            </div>
          </aside>
        </div>
      </div>
    </div>
  );
}
