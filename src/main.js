const { invoke } = window.__TAURI__.tauri;

let playing = false;

async function play() {
  playing = !playing;
  await invoke("play", { playing });
}

async function tick() {
  const time = await invoke("current");
  document.querySelector("#play").textContent = time;
}

window.addEventListener("DOMContentLoaded", () => {
  setInterval(async () => {
    await tick();
  }, 500);

  document.querySelector("#play").addEventListener("click", (e) => {
    e.preventDefault();
    play();
  });
});
