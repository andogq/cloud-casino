window.addEventListener("load", () => {
  // Ensure all icons are loaded
  lucide.createIcons();

  // Any time HTMX finishes modifying the DOM, check if any icons need to be inserted
  document.body.addEventListener("htmx:afterSwap", lucide.createIcons);
});
