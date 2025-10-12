/**
 * Check if mp3 playback is supported on the browser/webview
 * @param type     - MIME type of the file to check for playback support
 * @returns          True if supported, else false
 */
function isMp3PlaybackSupported(type = "audio/mpeg"): boolean {
  const audio = document.createElement("audio");
  return !!audio.canPlayType && audio.canPlayType(type) !== "";
}

type sfx = {
  [key: string]: HTMLAudioElement;
};

let { popSound, successSound }: sfx = loadSoundFiles();

/**
 * Load all the sfx files
 * @returns  An object containing all the supported sfx
 */
function loadSoundFiles(): sfx {
  let popSound: HTMLAudioElement;
  let successSound: HTMLAudioElement;

  // load sounds only once
  if (isMp3PlaybackSupported()) {
    //pre-load pop sound
    popSound = new Audio("/sfx/pop.mp3");
    popSound.preload = "auto";
    popSound.load(); // starts loading immediately

    //pre-load success sound
    successSound = new Audio("/sfx/success.mp3");
    successSound.preload = "auto";
    successSound.load();
    return { popSound, successSound };
  } else {
    popSound = new Audio();
    successSound = new Audio();
    return { popSound, successSound };
  }
}

/**
 * Play the given sfx if sounds are enabled in settings
 * @param name     - name of the sfx to be played (there are 2 currently supported i.e "pop" and "success")
 * @returns          void
 */
export function playSfx(name: string): void {
  const isSfxEnabled = localStorage.getItem("isSfxEnabled") ?? "false";
  if (isMp3PlaybackSupported() && isSfxEnabled === "true") {
    const pop = popSound.cloneNode() as HTMLAudioElement;
    const success = successSound.cloneNode() as HTMLAudioElement;
    switch (name) {
      case "pop":
        pop.currentTime = 0;
        pop.play();
        break;
      case "success":
        success.currentTime = 0;
        success.play();
        break;
      default:
        pop.currentTime = 0;
        pop.play();
    }
  }
}
