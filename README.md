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

1. Connect both devices to same wifi/local network.
2. Open the Tranzit app on both devices, it will detect the devices automatically. If not detected automatically, click the refresh button
3. Select files and click on the device from the device list to send a file transfer request.
4. Accept the file transfer request on the other device to start the file transfer.

## Download / Install

<table>
  <thead>
    <tr>
      <th>Platform</th>
      <th>Package / Method</th>
      <th>Install / Download</th>
    </tr>
  </thead>
  <tbody>
    <!-- Linux -->
    <tr>
      <td rowspan="3" valign="middle">
        <img src="https://img.icons8.com/color/48/000000/linux.png" alt="Linux" title="Linux"/>
      </td>
      <td>AUR</td>
      <td><code>yay -S tranzit</code></td>
    </tr>
    <tr>
      <td>.deb</td>
      <td><a href="https://github.com/kr5hn4/tranzit/releases/latest">Download</a></td>
    </tr>
    <tr>
      <td>AppImage</td>
      <td><a href="https://github.com/kr5hn4/tranzit/releases/latest">Download</a></td>
    </tr>
    <!-- macOS -->
    <tr>
      <td rowspan="2" valign="middle">
        <img src="https://img.icons8.com/color/48/000000/mac-os.png" alt="macOS" title="macOS"/>
      </td>
      <td>Homebrew Cask</td>
      <td><code>coming soon...</code></td>
    </tr>
    <tr>
      <td>DMG</td>
      <td><a href="https://github.com/kr5hn4/tranzit/releases/latest">Download</a></td>
    </tr>
    <!-- Windows -->
    <tr>
      <td>
        <img src="https://img.icons8.com/color/48/000000/windows-xp.png" alt="Windows" title="Windows"/>
      </td>
      <td>Installer</td>
      <td><a href="https://github.com/kr5hn4/tranzit/releases/latest">Download</a></td>
    </tr>
    <!-- Android -->
    <tr>
      <td rowspan="3" valign="middle">
        <img src="https://img.icons8.com/color/48/000000/android-os.png" alt="Android" title="Android"/>
      </td>
      <td>APK (ARM64)</td>
      <td><a href="https://github.com/kr5hn4/tranzit/releases/latest">Download</a></td>
    </tr>
    <tr>
      <td>APK (x86_64)</td>
      <td><a href="https://github.com/kr5hn4/tranzit/releases/latest">Download</a></td>
    </tr>
    <tr>
      <td>Play Store</td>
      <td valign="middle">
        <a href="https://play.google.com/store/apps/details?id=org.tranzit.app" target="_blank">
          <img src="https://play.google.com/intl/en_us/badges/images/generic/en_badge_web_generic.png" alt="Get it on Google Play" width="150"/>
        </a>
      </td>
    </tr>
  </tbody>
</table>

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

## Contributors

<a href="https://github.com/kr5hn4/tranzit/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=kr5hn4/tranzit" />
</a>
