# Link Studio (First Demo)

Local-first multi-camera director for a Windows laptop and phones on the **same LAN**.
Phones send video with WebRTC. The PC mixes Program (PGM) in the browser: switch, PIP, caption.

Supported First Demo networks:

- **Wi-Fi + Wi-Fi** — Director PC on Wi-Fi, iPhone on the same Wi-Fi LAN
- **Ethernet + Wi-Fi** — Director PC on Ethernet, iPhone on the same LAN Wi-Fi

Not in scope: phone on 4G/5G, cloud signaling, TURN, or reaching a private LAN IP from the public internet.

Cloud signaling, SFU, Supabase, AI, and ESP32 hardware are **out of scope** for this demo.
See `docs/MVP_SCOPE.md` and `docs/Link_Studio_RFP.pdf`.

## Requirements

- Node.js 24+
- Rust 1.98+ (`rustup`)
- **MSVC C++ build tools** (`link.exe`) — required to compile the Tauri/Rust server on Windows
- WebView2 (usually already on Windows 10/11)

If `cargo test` fails with `linker link.exe not found`, install
[Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
with the **Desktop development with C++** workload, then open a new terminal.

## Run (development)

```powershell
npm install
npm run tauri dev
```

This opens the Director window (Vite on `http://localhost:1420`) and starts the studio server:

| Bind | Port | Use |
|------|------|-----|
| `0.0.0.0` HTTP | 8787 | Director API, loopback PGM, OBS on the same PC |
| `0.0.0.0` HTTPS | 8443 | Phone camera pages (secure context) |

Headless server (after `npm run build` so `/dist` exists):

```powershell
npm run build
npm run server
```

Then open `http://127.0.0.1:8787/`.

## Why HTTPS for phones

`getUserMedia()` only works in a [secure context](https://developer.mozilla.org/en-US/docs/Web/Security/Secure_Contexts):

- `http://localhost` / `http://127.0.0.1` — yes (Director PC)
- `http://192.168.x.x` — **no** (phones will be blocked)
- `https://192.168.x.x` with a CA the phone trusts — yes

This app generates a **local CA** on first launch (`data/certs/ca.pem` or the Tauri app-data `certs/ca.pem`) and a server certificate whose SAN includes the current LAN IPs.

Do **not** use Chrome flags such as “insecure origins treated as secure” as the default workaround.

The CA installer is served over **HTTP port 8787**. That is intentional: the phone cannot trust HTTPS until the CA is installed, so the profile must not depend on the untrusted certificate.

### Trust the CA on iOS Safari (required for iPhone)

Use **Safari**, not Chrome. Chrome on iOS will not install a configuration profile.

1. On the iPhone, open this **HTTP** URL (same Wi-Fi as the PC):

   `http://<LAN-IP>:8787/ios/link-studio-ca.mobileconfig`

   Example: `http://192.168.0.137:8787/ios/link-studio-ca.mobileconfig`

   Or open `http://<LAN-IP>:8787/install-ca` and tap **Download iOS configuration profile**.

2. iOS should treat the file as a configuration profile. If Safari asks, Allow the download.

3. Open **Settings**. A **Profile Downloaded** row appears near the top. Tap it.

   If you missed that banner: **Settings → General → VPN & Device Management** (or **Profiles**).

4. Tap **Install**, enter the passcode, confirm Install.

5. **Settings → General → About → Certificate Trust Settings**.

6. Enable full trust for **Link Studio Local CA**. Confirm the warning. Without this step Safari still shows `NET::ERR_CERT_AUTHORITY_INVALID`.

7. Only then open the **HTTPS** camera URL:

   `https://<LAN-IP>:8443/camera/<session>`

8. Allow camera (and microphone) access, then Start Camera.

Do **not** install `ca.pem` on iOS. A raw PEM is not a configuration profile, so it never appears under Profile Downloaded.

### Trust the CA on Android Chrome

1. On the phone, open this **HTTP** URL:

   `http://<LAN-IP>:8787/ca.crt`

2. Settings → Security → Encryption & credentials → Install a certificate → **CA certificate**.

3. Open the Director **HTTPS camera URL** (`https://<LAN-IP>:8443/camera/<session>`).

## Demo procedure

1. PC and phones on the **same Wi-Fi** (client isolation off on the AP).
2. `npm run tauri dev` (or production build).
3. Create a project → **Start camera session**.
4. Phone 1: scan QR or open the camera URL → slot Camera 1 → Start Camera → allow permission.
5. Phone 2: same URL, slot Camera 2.
6. In Director, confirm two previews → TAKE CAM 1 / CAM 2.
7. Enable PIP and SHOW caption.
8. Open the PGM HTTP URL on the PC (`http://127.0.0.1:8787/program/<session>`) or in OBS Browser Source.
9. Quit and relaunch: the project is still in SQLite.

## Connection network (Wi-Fi vs Ethernet)

Director lists **live** physical adapters only. A disconnected Ethernet NIC that still has a leftover DHCP address (for example `192.168.0.160` with AddressState Deprecated) is **not** used for QR codes.

In **Camera connection** choose:

- `Wi-Fi — 192.168.0.137`
- `Ethernet — 192.168.0.160` (only when the cable is actually up)

The same selected host is used for Camera URL, QR, iOS CA install, and Android CA download. Changing the selector does **not** create a new session. PGM/OBS stays `http://127.0.0.1:8787/program/<session>`.

If you plug Ethernet in after launch, wait a few seconds (Director refreshes adapters) or restart the app so the TLS certificate SAN can include the new IP.

### Ethernet + Wi-Fi still fails after selecting a live Ethernet IP

The HTTP/HTTPS servers already bind `0.0.0.0:8787` and `0.0.0.0:8443`. If the phone cannot open the Ethernet IP:

1. Confirm the Ethernet adapter status is **Up / Connected**, not Disconnected.
2. Confirm PC Ethernet and the phone are on the **same subnet** (same router, client isolation off).
3. Set the Ethernet (and Wi-Fi) network profile to **Private**:
   - Settings → Network & internet → Ethernet (or Wi-Fi) → **Private network**
4. Add inbound allow rules **only** for TCP 8787 and 8443 (do not turn Windows Firewall off):

```powershell
netsh advfirewall firewall add rule name="Link Studio LAN HTTP 8787" dir=in action=allow protocol=TCP localport=8787 profile=private,domain
netsh advfirewall firewall add rule name="Link Studio LAN HTTPS 8443" dir=in action=allow protocol=TCP localport=8443 profile=private,domain
```

An elevated Command Prompt may be required. Director also has **Allow LAN inbound (Windows Firewall)** — it asks before changing anything.

If Windows labeled the NIC **Public**, either switch it to Private or keep the existing per-app allow rule for `link-studio.exe`. Do not disable the firewall.

## Human verification (not claimed PASS without devices)

- TEST-H1 QR/URL on a phone (Wi-Fi PC + Wi-Fi iPhone)
- TEST-H2 camera permission
- TEST-H3 phone → Director preview
- TEST-H4 two phones
- TEST-H5 LAN switching
- TEST-H6 OBS Browser Source
- TEST-H7 iOS Safari
- TEST-H8 Android Chrome
- TEST-H9 Director Ethernet + iPhone Wi-Fi (live Ethernet IP in the selector, not a stale disconnected address)

## Automated checks

```powershell
npm run typecheck
npm run test
npm run test:rust
npm run build
```

`npm run test:rust` needs `link.exe`.

## Architecture

P2P WebRTC, local WebSocket signaling, SQLite. Details: `docs/ARCHITECTURE.md`.
