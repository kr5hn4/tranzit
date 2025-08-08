function isMp3PlaybackSupported(type = "audio/mpeg") {
  const audio = document.createElement("audio");
  return !!audio.canPlayType && audio.canPlayType(type) !== "";
}

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
}

export function playSfx(name: string) {
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
