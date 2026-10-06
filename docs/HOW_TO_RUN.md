# Link Studio 실행 방법

같은 LAN에 있는 Windows PC(Director)와 스마트폰(카메라)으로 로컬 멀티캠을 돌리는 First Demo입니다.

## 준비물

| 항목 | 비고 |
|------|------|
| Windows 10/11 PC | Director 실행용 |
| Node.js 24+ | |
| Rust (`rustup`) | |
| MSVC C++ 빌드 도구 | `link.exe` 필요 |
| WebView2 | 보통 Windows에 기본 설치됨 |
| 스마트폰 | PC와 **같은 Wi-Fi/LAN** |

클라우드·4G/5G·외부 인터넷 접속은 지원하지 않습니다.

## 1. 코드 받기

```powershell
git clone -b geunwoo-linkstudio-demo https://github.com/gwada-chulhyeol/link-studio-test.git
cd link-studio-test
```

## 2. PC에서 실행

```powershell
npm install
npm run tauri dev
```

Director 창이 열리고 서버가 함께 뜹니다.

| 포트 | 용도 |
|------|------|
| HTTP `8787` | Director API, CA 설치, OBS/PGM(PC) |
| HTTPS `8443` | 폰 카메라 페이지 |

헤드리스만 쓸 때:

```powershell
npm run build
npm run server
```

이후 브라우저에서 `http://127.0.0.1:8787/` 를 엽니다.

## 3. 데모 순서

1. PC와 폰을 같은 LAN에 연결합니다. (공유기 client isolation OFF)
2. Director에서 프로젝트 생성 → **Start camera session**
3. 폰에 **로컬 CA를 먼저 설치**합니다. (아래 참고)
4. QR 또는 카메라 URL로 접속 → 슬롯 선택 → Start Camera → 권한 허용
5. Director에서 프리뷰 확인 후 TAKE / PIP / caption 사용
6. PGM(OBS용): `http://127.0.0.1:8787/program/<session>`

## 4. 폰 CA 설치 (필수)

`getUserMedia`는 HTTPS가 필요합니다. 폰은 LAN IP라서 **로컬 CA를 신뢰**해야 합니다.  
Chrome insecure-origin 플래그는 기본 우회로 쓰지 마세요.

### iPhone (Safari만)

1. `http://<PC-LAN-IP>:8787/ios/link-studio-ca.mobileconfig` 열기
2. 설정 → 프로필 설치
3. 설정 → 일반 → 정보 → **인증서 신뢰 설정**에서 Link Studio Local CA 신뢰 ON
4. 그다음 `https://<PC-LAN-IP>:8443/camera/<session>` 접속

### Android (Chrome)

1. `http://<PC-LAN-IP>:8787/ca.crt` 받기
2. 설정 → 보안 → 인증서 설치 → **CA 인증서**
3. `https://<PC-LAN-IP>:8443/camera/<session>` 접속

## 막힐 때

- 폰이 페이지에 안 닿으면: PC 방화벽에서 TCP **8787**, **8443** 인바운드 허용, 네트워크 프로필 **개인(Private)**
- Ethernet + Wi-Fi: Director의 Camera connection에서 **실제 연결된** 어댑터 IP를 선택
- iOS에서 CA가 안 보이면: Chrome이 아니라 **Safari**로 `.mobileconfig`를 받았는지 확인

더 자세한 네트워크·방화벽·검증 항목은 저장소 루트 `README.md`를 보세요.
