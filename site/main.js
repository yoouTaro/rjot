// Content rises in once as it enters the viewport; the top bar gets a
// hairline once the page scrolls. Without JS (no "js" class, set in the
// page head) everything is simply shown.

const reveal = new IntersectionObserver(
  (entries) => {
    for (const e of entries) {
      if (e.isIntersecting) {
        e.target.classList.add("in");
        reveal.unobserve(e.target);
      }
    }
  },
  { rootMargin: "0px 0px -8% 0px" },
);
document.querySelectorAll(".reveal").forEach((el) => reveal.observe(el));

const bar = document.querySelector(".bar");
const sentinel = document.createElement("div");
sentinel.style.cssText = "position:absolute;top:0;height:1px;width:1px";
document.body.prepend(sentinel);
new IntersectionObserver(([e]) => bar.classList.toggle("scrolled", !e.isIntersecting)).observe(sentinel);
