// The bar is a running head: as the reader moves through the night, it shows
// the time and the chapter they're in, and takes on that chapter's light.

const bar = document.getElementById("bar");
const time = document.getElementById("phase-time");
const name = document.getElementById("phase-name");
const links = [...bar.querySelectorAll("nav a[href^='#']")];
const sections = [...document.querySelectorAll("[data-phase]")];

function show(section) {
  bar.dataset.phase = section.dataset.phase;
  time.textContent = section.dataset.time;
  name.textContent = section.dataset.name;
  for (const a of links) {
    if (a.getAttribute("href") === `#${section.id}`) a.setAttribute("aria-current", "true");
    else a.removeAttribute("aria-current");
  }
}

// The chapter under the bar is the one that counts.
const watcher = new IntersectionObserver(
  (entries) => {
    for (const e of entries) if (e.isIntersecting) show(e.target);
  },
  { rootMargin: "-60px 0px -85% 0px" },
);
for (const s of sections) watcher.observe(s);
show(sections[0]);
