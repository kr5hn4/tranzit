<p align="center">
  <img src="./static/images/logo.png" alt="Tranzit logo" />
</p>

**Tranzit** is a cross-platform app to share files on your local network.

Website: [kr5hn4.github.io/tranzit/](https://kr5hn4.github.io/tranzit/)

## Screenshots

<img src="https://kr5hn4.github.io/tranzit/images/screenshot-2.png" height="300"/><img src="https://kr5hn4.github.io/tranzit/images/screenshot-3.png" height="300"/>

## How It Works

Tranzit uses mDNS and tcp heartbeats to discover devices, If a device stops responding to heartbeats, it is considered offline. All the file transfers happen via secure https connections between the devices.

## How to use

[video demo](https://youtube.com/)

1. Connect both devices to same wifi/local network.
2. Open the Tranzit app on both devices, it will detect the devices automatically. If not detected automatically, click the refresh button
3. Select files and click on the device from the device list to send a file transfer request.
4. Accept the file transfer request on the other device to start the file transfer.

## How to run it locally

1. Clone the repository

2. Make sure to have the following dependencies installed:
   - Install the required dependencies for your OS as per [Tauri docs](https://v2.tauri.app/start/prerequisites/#system-dependencies)
   - Install [rust](https://rust-lang.org/tools/install/) (preferrably the latest LTS version)
   - Install [node.js](https://nodejs.org/en/download) (preferrably the latest LTS version)

3. Install dependencies: `npm install`

4. Copy sample.env to .env: `cp sample.env .env`

5. Run the project locally
   - `npm run tauri dev` (for windows, mac, linux)
   - `npm run tauri android dev` (for android)

   (please ignore the postcss error by clicking anywhere on the screen for now, as this is a known issue in [tauri](https://github.com/tauri-apps/tauri/issues/5839))

6. Build the project
   - `npm run tauri build` (for windows, mac, linux)
   - `npm run tauri android` build (for android)

## 📥 Download

| Platform | Link                                                                                                |
| -------- | --------------------------------------------------------------------------------------------------- |
| Windows  | [Download](https://github.com/kr5hn4/tranzit/releases/download/v0.2.0/tranzit_0.1.0_x64-setup.exe)  |
| macOS    | coming soon...                                                                                      |
| Linux    | [Download](https://github.com/kr5hn4/tranzit/releases/download/v0.2.0/tranzit_0.1.0_amd64.AppImage) |
| Android  | [Download](https://play.google.com/store/apps/details?id=org.tranzit.app)                           |
| iOS      | coming soon...                                                                                      |

(Join the [discord](https://discord.gg/2eGza5XT) channel to join the closed beta testing on Android)

## 🛠️ Built With

- 🦀 Rust
- Tauri
- SvelteKit
