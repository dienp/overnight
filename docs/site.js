// The bar is a running head: as the reader moves through the night, it shows
// the time and the chapter they're in, and takes on that chapter's light.

const bar = document.getElementById("bar");
const time = document.getElementById("phase-time");
const name = document.getElementById("phase-name");
const links = [...bar.querySelectorAll("nav a[href^='#']")];
const sections = [...document.querySelectorAll("[data-phase]")];

let shown = null;

function show(section) {
  if (section === shown) return;
  shown = section;
  bar.dataset.phase = section.dataset.phase;
  time.textContent = section.dataset.time;
  name.textContent = section.dataset.name;
  for (const a of links) {
    if (a.getAttribute("href") === `#${section.id}`) a.setAttribute("aria-current", "true");
    else a.removeAttribute("aria-current");
  }
}

// The chapter under the bar is the last one whose top has passed it. Reading
// six positions per scroll event is cheap, and unlike an intersection
// observer it gives the right answer after a jump to an anchor.
function update() {
  // Past the 12px of scroll padding that anchor jumps stop at, with room to spare.
  const line = bar.offsetHeight + 24;
  let current = sections[0];
  for (const s of sections) if (s.getBoundingClientRect().top <= line) current = s;
  show(current);
}

addEventListener("scroll", update, { passive: true });
addEventListener("resize", update);
update();

// The download button always fetches the newest release; say which one it is.
// Without the network, the text stays "The latest version".
fetch("https://api.github.com/repos/dienp/overnight/releases/latest")
  .then((r) => (r.ok ? r.json() : null))
  .then((release) => {
    const tag = release?.tag_name;
    if (!tag) return;
    for (const el of document.querySelectorAll("[data-version]")) el.textContent = `Version ${tag.replace(/^v/, "")}`;
  })
  .catch(() => {});
