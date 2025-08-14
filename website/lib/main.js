import { windowsZip } from "@/lib/gh-assets.js";

const userAgent = window.navigator.userAgent;
if (userAgent.indexOf("Linux") !== -1) {
  document.body.classList.add("is-linux");
} else {
  document.body.classList.add("is-windows");
}

const downloadCard = document.querySelector(".download-card");

function overrideDownloadLinks() {
  if (windowsZip) {
    downloadCard
      .querySelectorAll("a.windows-only, .windows-only > a")
      .forEach((a) => a.setAttribute("href", windowsZip));
  }
}

overrideDownloadLinks();

const mutationObserver = new MutationObserver(([{ addedNodes }]) => {
  if (addedNodes.length) {
    // A tippy.js popover has appeared, we override their links
    //
    // Popover logic is in @/components/SplitButton.astro
    overrideDownloadLinks();
  }
});
mutationObserver.observe(downloadCard, {
  subtree: true,
  childList: true,
});

function restoreVisibility() {
  // KEEP THIS ORDER,
  // visibility must be restored first so that opacity transition works properly
  document.documentElement.style.visibility = "";
  document.documentElement.style.overflow = "";
  document.documentElement.style.opacity = "";
}

if (document.readyState === "complete") {
  restoreVisibility();
} else {
  window.addEventListener("load", () => restoreVisibility());
}
