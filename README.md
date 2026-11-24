# Atoll Island

![Platform](https://img.shields.io/badge/Windows-10%20%2F%2011-0078D6?logo=windows&logoColor=white)
![Built with Rust](https://img.shields.io/badge/Rust-Tauri%202-orange?logo=rust&logoColor=white)
![Languages](https://img.shields.io/badge/Languages-12-30d158)
![License](https://img.shields.io/badge/License-GPL--3.0-blue)

An animated **Dynamic Island / Now Bar** for Windows 10 and 11, written in pure **Rust + [Tauri 2](https://tauri.app)**. A compact pill sits at the top center of your screen and springs open for media, lyrics, notifications, volume, a file shelf, downloads and more. **No Electron, no PowerShell.**

> **Beta:** Features are tested in daily use. If you find a bug or have an idea, please open an issue.

---

## Installation

### 1. MSI installer (recommended)
Prebuilt `.msi` installers (`en-US` and `tr-TR`) are published on the [Releases](../../releases) page when available.
- Creates a Start Menu shortcut and an Add/Remove Programs entry.
- Installs WebView2 automatically if it is missing (it ships with Windows 11).

### 2. Build from source
Requirements: Windows 10/11, [Rust](https://rustup.rs) (stable), Visual Studio C++ Build Tools, Node.js (only for packaging the `.msi`).

```powershell
git clone https://github.com/UgurYorulmaz1/Atoll-Island.git
cd Atoll-Island\src-tauri

# Build the executable only:
cargo build --release

# Build the MSI installer:
npx @tauri-apps/cli@2 build --bundles msi
```

---

## Features

### Media, music and Spotify
- **Live media controls (SMTC):** Album art, background that follows the cover color, animated equalizer bars, play/pause, previous/next, shuffle/repeat and a draggable progress bar for Spotify, Apple Music, YouTube Music and browsers. Apple Music seeking works through UI Automation.
- **Two sources (Now Bar):** When two players run at once, the main pill shows one and a small bubble shows the other; click the bubble to swap them.
- **Single source mode (optional):** Pauses the previous player when a new one starts.
- **Synced lyrics ([LRCLIB](https://lrclib.net)):** Apple Music style word-by-word or line-by-line lyrics. Click a line to jump there; fine-tune the timing with `− / +`.
- **Spotify Web API (optional; basic media controls work without it):**
  - Like (heart), view the play queue and jump straight to any track in it
  - Search Spotify from the island, add to queue (`+`) and an **`∞ Similar`** autoplay toggle
  - **Spotify Connect:** One-click switching between PC, phone and speakers

### Notifications, calls and volume
- **Expandable Windows notifications:** Notifications appear in the island; **click to expand** and read long messages, **Open app** brings the right window to the front, or dismiss.
- **Incoming call card:** Live call card for WhatsApp, Telegram, Teams and Discord voice/video calls.
- **Volume:** Windows volume keys show in the island instead of the standard Windows OSD. Scroll to change volume, drag the volume bar, and right-click the speaker icon for a quick **audio output switcher** (speakers ↔ headphones).

### Shelf and clipboard
- **Drag-and-drop file shelf:** Drag files from the desktop or Explorer onto the island, keep them there, then drag them anywhere (`SHDoDragDrop`). Stored on disk or in RAM with a configurable size limit.
- **Clipboard history:** Text, screenshots and files you copy with `Ctrl+C` land on the shelf automatically (password managers are excluded).

### Downloads, weather and stats
- **System-wide download tracker:**
  - **Chrome / Edge / Brave / Opera** (real `%` and speed from the local `History` SQLite database), plus **Steam** and **Epic Games**.
  - A progress ring next to the compact pill, a list of up to **5** downloads when expanded, and an open / add-to-shelf notification when finished.
- **Weather and rain alert:** Temperature next to the clock while idle, and a warning if rain starts within the hour ([Open-Meteo](https://open-meteo.com), no API key).
- **Listening history and weekly summary:** Daily and weekly listening time, a 7-day chart, and your top artists and tracks.

### Hardware, system and privacy
- **Live system and gaming monitor:** CPU, NVIDIA GPU temperature and load, RAM, and download/upload speed.
- **Wireless device batteries:** Headphone, keyboard and mouse battery levels over Bluetooth Classic and **BLE GATT (`0x180F`)**, plus a headphones-connected alert.
- **Privacy and recording dots:** Microphone (orange) and camera (green) indicators, and a screen-recording indicator for OBS and Xbox Game Bar.

### Customization, languages and performance
- **Zero idle cost and GPU acceleration:** Background features that are turned off never wake up and use **0 CPU / 0 disk / 0 network**. WebView2 GPU rasterization and hardware acceleration are supported.
- **12 languages:** Turkish, English, German, French, Spanish, Italian, Portuguese, Russian, Arabic, Japanese, Korean and Chinese (including context and tray menus and localized dates).
- **Simple / Advanced mode and customization:** A one-switch simple Settings view, category tabs and setting search. Background style (Black / Glass / Dark Gray), accent color, top offset (`0–48 px`), idle opacity and Auto-Dim, Windows-matched or custom scaling (`75–115%`), animation speed, `24h / 12h` clock, multi-monitor selection and one-click reset.

---

## Connecting Spotify (optional)

1. Create an app at [developer.spotify.com/dashboard](https://developer.spotify.com/dashboard).
2. Add `http://127.0.0.1:8989/login` as a **Redirect URI**.
3. Copy the app's **Client ID**, paste it into **Atoll Island → Settings → Spotify** and click **Connect with Spotify**.

> Note: Spotify requires the account owner to have **Premium** for apps in development mode. No `client secret` is needed; sign-in uses PKCE.

---

## Tech stack

- **Backend:** Rust (2021), Tauri 2, `windows` 0.62 (Win32 / WinRT: SMTC, Core Audio, UI Automation, SetupAPI, UserNotificationListener, Shell/OLE), `tokio`, `reqwest`, `rusqlite`
- **Frontend:** Plain HTML5 / CSS3 / vanilla JavaScript (no framework)

## Privacy

- Settings, the shelf and statistics are stored only on your computer (`%APPDATA%\com.island.dynamic`).
- For lyrics, only the track title, artist and duration are sent to LRCLIB.
- When weather is on, `ipwho.is` is used for approximate location and `Open-Meteo` for the forecast.
- Spotify features talk to the Spotify Web API only after you connect your account.

## License

[GPL-3.0](LICENSE)
