# Contributing Guidelines

Thanks for your interest in contributing! 🎉

## How to Contribute

1. **Fork the repo** and create your branch from `main`.
2. Make your changes (code, docs, tests, or UI improvements).
3. Run and test the app to make sure it works.
4. Commit with a clear message (e.g., `chore: update contributing guidelines`).
5. Push your branch and open a Pull Request (PR).

## How to run it locally

1. Clone the repository
2. Install system level dependencies:
   - Install the required dependencies for your OS as per [Tauri docs](https://v2.tauri.app/start/prerequisites/#system-dependencies)
   - Install [rust](https://rust-lang.org/tools/install/) (preferrably the latest LTS version)
   - Install [node.js](https://nodejs.org/en/download) (preferrably the latest LTS version)
3. Install project level dependencies: `npm install`
4. Run the project locally
   - `npm run tauri dev` (for windows, mac, linux)
   - `npm run tauri android dev` (for android)

   (please ignore the postcss error by clicking anywhere on the screen for now, as this is a known issue in [tauri](https://github.com/tauri-apps/tauri/issues/5839))
   <img width="2255" height="1268" alt="image" src="https://github.com/user-attachments/assets/9e76c729-092d-49ed-870e-939b27b8cb57" />

## Build the project

- `npm run tauri build` (for windows, mac, linux)
- `npm run tauri android` build (for android)

## Reporting Issues

- Use the **Issues tab** to report bugs or suggest features.
- Check for duplicates before creating a new issue.

## Code Style

- Follow existing code style in the project.
- Add comments where needed.

## Non-Code Contributions

- Improvements to documentation.
- Adding screenshots, icons, or updating assets, creating videos.
- Adding translations.
