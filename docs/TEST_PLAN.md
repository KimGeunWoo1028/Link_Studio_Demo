# Test plan

## Automated (executed when the toolchain is present)

- Vitest: caption helpers
- `tsc --noEmit`
- `vite build`
- `cargo test`: SQLite + signaling hub

## Not a real-device PASS

Playwright fake-camera and `cargo test` do not prove Android/iOS WebRTC.

## Manual

See README TEST-H1–H8.
