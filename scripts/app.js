/* ==========================================================================
   ref42 · client enhancements
   Progressive enhancement only: the site is fully readable and navigable
   with JavaScript disabled.
   ========================================================================== */

(function () {
  "use strict";

  var doc = document.documentElement;
  doc.classList.add("js");

  var reduceMotion = window.matchMedia
    ? window.matchMedia("(prefers-reduced-motion: reduce)").matches
    : false;

  function $(sel, root) {
    return (root || document).querySelector(sel);
  }

  function $$(sel, root) {
    return Array.prototype.slice.call((root || document).querySelectorAll(sel));
  }

  /* Status text that only assistive tech should hear. `role="status"` already
     implies `aria-live="polite"`, but both are set because the pairing is what
     older screen readers actually honour. */
  function visuallyHiddenLive(className) {
    var el = document.createElement("div");
    el.className = className;
    el.setAttribute("role", "status");
    el.setAttribute("aria-live", "polite");
    return el;
  }

  /* Shared by the `/` jump, the shortcuts overlay and `j`/`k`: a keystroke is
     ours to act on only when it is not going into a field the reader is
     typing in. `isContentEditable` covers `contenteditable=""` and the bare
     attribute, and is false in browsers that never implemented it, so the
     check costs nothing where it does not exist. */
  function isEditable(target) {
    return !!(
      target &&
      (target.tagName === "INPUT" ||
        target.tagName === "TEXTAREA" ||
        target.isContentEditable)
    );
  }

  /* A modifier means the keystroke belongs to the browser or the OS —
     Cmd+? on macOS, Ctrl+j in some terminals — not to this script. */
  function hasModifier(event) {
    return event.ctrlKey || event.metaKey || event.altKey;
  }

  /* ---- scrolling ------------------------------------------------------- */

  /* Fixed duration, on purpose.
   *
   * `window.scrollTo({ behavior: "smooth" })` is distance-based: the native
   * animation scales its duration with how far it has to travel, so jumping
   * back to the top of a long note takes well over a second and feels like it
   * has stalled. It also runs on its own schedule, which fights the reading
   * progress bar and the contents highlight that listen for scroll events.
   *
   * Animating here instead gives one predictable duration regardless of
   * distance or page length. A new call cancels the one in flight, so
   * interrupting a scroll does not leave two animations competing. */
  var SCROLL_MS = 380;
  var scrollFrame = 0;

  function easeOutCubic(t) {
    return 1 - Math.pow(1 - t, 3);
  }

  function scrollToY(target) {
    if (scrollFrame) {
      window.cancelAnimationFrame(scrollFrame);
      scrollFrame = 0;
    }

    var start = window.scrollY || doc.scrollTop || 0;
    var delta = target - start;

    if (reduceMotion || delta === 0 || !window.requestAnimationFrame) {
      window.scrollTo(0, target);
      return;
    }

    var started = 0;

    function step(now) {
      if (!started) started = now;
      var t = Math.min(1, (now - started) / SCROLL_MS);
      window.scrollTo(0, start + delta * easeOutCubic(t));
      scrollFrame = t < 1 ? window.requestAnimationFrame(step) : 0;
    }

    scrollFrame = window.requestAnimationFrame(step);
  }

  /* Ties the animation to the user: a wheel tick or a keypress mid-flight
     hands control straight back instead of yanking the page on. */
  ["wheel", "touchstart", "keydown"].forEach(function (event) {
    window.addEventListener(
      event,
      function () {
        if (!scrollFrame) return;
        window.cancelAnimationFrame(scrollFrame);
        scrollFrame = 0;
      },
      { passive: true }
    );
  });

  /* In-page anchors — the contents rail and the per-heading anchors — are
     routed through the same animation so every jump on the site lasts the same
     380ms.
     
     The offset is measured from the real header element rather than parsed out
     of `--header-h`. That token is `5rem`, and `parseFloat("5rem")` is 5, not
     80: the unit is silently dropped and the guard against NaN never fires
     because the parse "succeeds". The result was an offset of 21px instead of
     96px, which native `scroll-padding-top` happened to mask on plain anchor
     jumps while leaving programmatic jumps ~75px off. Measuring the rendered
     element is correct at any root font size. */
  function anchorOffset() {
    var header = $(".site-header");
    var height = header ? header.getBoundingClientRect().height : 0;
    if (!height) height = 80;
    return height + 16;
  }

  function initAnchors() {
    document.addEventListener("click", function (event) {
      if (event.defaultPrevented || event.button !== 0) return;
      if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;

      var link = event.target.closest ? event.target.closest('a[href^="#"]') : null;
      if (!link) return;

      var hash = link.getAttribute("href");
      if (!hash || hash === "#") return;

      var el = document.getElementById(decodeURIComponent(hash.slice(1)));
      if (!el) return;

      event.preventDefault();
      scrollToY(Math.max(0, el.getBoundingClientRect().top + window.scrollY - anchorOffset()));

      /* Keep the URL and the back button honest without a second jump: the
         history entry is pushed, and the native anchor behaviour is suppressed
         by the `hashchange` that would otherwise re-scroll. */
      if (window.history && history.pushState) history.pushState(null, "", hash);
    });
  }

  /* ---- giscus theme ---------------------------------------------------- */

  /* The giscus theme is set declaratively, on the script tag in the page:
     `data-theme="dark_dimmed"`. There is deliberately nothing here to change it
     at runtime.

     There used to be: an observer that waited for the frame and then posted
     `setConfig`. It could not work. `postMessage` with a target origin is
     checked against the *recipient's* origin, and the moment the iframe element
     exists its `contentWindow` is still the initial `about:blank` — the parent's
     own origin — so every page load that reached this logged

       Failed to execute 'postMessage': The target origin provided
       ('https://giscus.app') does not match the recipient window's origin

     and the theme was never actually sent. Waiting for the frame's `load` event
     would fix the timing, but the site has one palette and never toggles, so the
     whole mechanism only ever duplicated what the attribute already says. */

  /* ---- mobile navigation ---------------------------------------------- */

  function initNav() {
    var burger = $(".nav-burger");
    var nav = $(".site-nav");
    if (!burger || !nav) return;

    function setOpen(open) {
      nav.classList.toggle("is-open", open);
      burger.setAttribute("aria-expanded", open ? "true" : "false");
    }

    burger.addEventListener("click", function () {
      setOpen(!nav.classList.contains("is-open"));
    });

    document.addEventListener("keydown", function (event) {
      if (event.key === "Escape") setOpen(false);
    });

    document.addEventListener("click", function (event) {
      if (!nav.contains(event.target) && !burger.contains(event.target)) {
        setOpen(false);
      }
    });
  }

  /* ---- reading progress + back to top ---------------------------------- */

  function initScrollChrome() {
    var bar = $(".progress__bar");
    var toTop = $(".to-top");
    var ticking = false;

    function sync() {
      ticking = false;
      var y = window.scrollY || doc.scrollTop;
      if (bar) {
        var max = doc.scrollHeight - window.innerHeight;
        var pct = max > 0 ? Math.min(100, (y / max) * 100) : 0;
        bar.style.width = pct.toFixed(2) + "%";
      }
      if (toTop) toTop.classList.toggle("is-visible", y > 560);
    }

    function onScroll() {
      if (ticking) return;
      ticking = true;
      window.requestAnimationFrame(sync);
    }

    window.addEventListener("scroll", onScroll, { passive: true });
    window.addEventListener("resize", onScroll, { passive: true });
    sync();

    if (toTop) {
      toTop.addEventListener("click", function () {
        scrollToY(0);
      });
    }
  }

  /* ---- reveal on scroll ------------------------------------------------ */

  function initReveal() {
    var items = $$("[data-reveal]");
    if (!items.length) return;

    function showAll() {
      items.forEach(function (el) {
        el.classList.add("is-in");
      });
    }

    // Anything the observer cannot help with — reduced motion, no observer
    // support, or an element that never intersects — still has to be visible.
    if (reduceMotion || !("IntersectionObserver" in window)) {
      showAll();
      return;
    }

    var observer = new IntersectionObserver(
      function (entries) {
        entries.forEach(function (entry) {
          if (!entry.isIntersecting) return;
          entry.target.classList.add("is-in");
          observer.unobserve(entry.target);
        });
      },
      { rootMargin: "0px 0px -8% 0px", threshold: 0.05 }
    );

    items.forEach(function (el, i) {
      el.style.setProperty("--reveal-delay", Math.min(i, 8) * 55 + "ms");
      observer.observe(el);
    });

    // Belt and braces: reveal everything shortly after load regardless, so a
    // missed observer callback can never leave the page looking empty.
    window.setTimeout(showAll, 2500);
  }

  /* ---- code blocks ----------------------------------------------------- */

  var COPY_ICON =
    '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M8 7.5V6a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2h-1.5" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M5 7.5h8a2 2 0 0 1 2 2v8.5a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V9.5a2 2 0 0 1 2-2Z" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>';
  var CHECK_ICON =
    '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="m5 12 4.5 4.5L19 7" stroke-width="2.3" stroke-linecap="round" stroke-linejoin="round"/></svg>';

  function fallbackCopy(text, done) {
    var area = document.createElement("textarea");
    area.value = text;
    area.setAttribute("readonly", "");
    area.style.position = "fixed";
    area.style.top = "-9999px";
    document.body.appendChild(area);
    area.select();
    try {
      document.execCommand("copy");
      done();
    } catch (_) {
      /* ignore */
    }
    document.body.removeChild(area);
  }

  function initCodeBlocks() {
    $$(".prose pre").forEach(function (pre) {
      var code = pre.querySelector("code");
      if (!code) return;

      var lang = pre.getAttribute("data-lang") || "";
      if (!lang) {
        var cls = code.className || "";
        var match = cls.match(/language-([\w+#-]+)/);
        if (match) lang = match[1];
      }

      var wrapper = document.createElement("div");
      wrapper.className = "code-block";
      pre.parentNode.insertBefore(wrapper, pre);

      var bar = document.createElement("div");
      bar.className = "code-block__bar";

      var dots = document.createElement("span");
      dots.className = "code-block__dots";
      dots.innerHTML = "<i></i><i></i><i></i>";

      var label = document.createElement("span");
      label.className = "code-block__lang";
      label.textContent = lang || "text";

      var button = document.createElement("button");
      button.type = "button";
      button.className = "code-copy";
      button.innerHTML = COPY_ICON;
      button.setAttribute("aria-label", "Copy code");
      button.setAttribute("title", "Copy code");

      button.addEventListener("click", function () {
        var text = code.innerText.replace(/\n$/, "");
        var done = function () {
          button.innerHTML = CHECK_ICON;
          button.classList.add("is-copied");
          button.setAttribute("aria-label", "Copied");
          window.setTimeout(function () {
            button.innerHTML = COPY_ICON;
            button.classList.remove("is-copied");
            button.setAttribute("aria-label", "Copy code");
          }, 1600);
        };
        if (navigator.clipboard && window.isSecureContext) {
          navigator.clipboard.writeText(text).then(done).catch(function () {
            fallbackCopy(text, done);
          });
        } else {
          fallbackCopy(text, done);
        }
      });

      bar.appendChild(dots);
      bar.appendChild(label);
      bar.appendChild(button);
      wrapper.appendChild(bar);
      wrapper.appendChild(pre);
    });
  }

  /* ---- lazy search index ----------------------------------------------- */

  /* The index is a separate JSON file, so a page that is never searched never
   * pays for it.
   *
   * Nothing is fetched while the page loads: the request happens on the first
   * `focus` or `input` on the field, or on the first press of the random-note
   * button. `indexDocs` holds the rows once they are here and `indexRequested`
   * is what keeps that to a single request for the life of the page — a failed
   * fetch is not retried, so an offline reader gets a search box that quietly
   * finds nothing instead of one that hammers the network.
   *
   * Both callers share this one loader rather than each fetching for
   * themselves: `indexRequested` is only set when a request really goes out,
   * so a random-note click in the middle of a search's request joins that
   * request instead of starting a second one, and one fetch serves both.
   *
   * The old inline `window.__REF42_INDEX__` is still honoured if the HTML
   * happens to carry it, so a cached page from before this change keeps
   * working. It holds the rows themselves, which is why it can stand in for
   * `indexDocs` directly. */
  var indexDocs = null;
  var indexRequested = false;

  function loadIndex(done) {
    if (indexRequested || indexDocs !== null) return;

    var legacy = window.__REF42_INDEX__;
    if (Array.isArray(legacy)) {
      indexDocs = legacy;
      if (typeof done === "function") done();
      return;
    }

    indexRequested = true;

    /* Root-absolute rather than resolved against the document. The site is
       served from the domain root, and `document.baseURI` on
       `/toolchain/cortex-m/` would ask for
       `/toolchain/cortex-m/search-index.json` — which would mean either a
       copy of the index in every directory or a 404 on every page except the
       home page. Serving under a subpath would change this one string. */
    var url = "/search-index.json";

    fetch(url, { credentials: "same-origin" })
      .then(function (response) {
        if (!response.ok) return null;
        return response.json();
      })
      .then(function (value) {
        /* The server's ETag means a repeat fetch is usually a 304, which the
           browser resolves from cache as a normal response, body and all. If
           the shape is ever anything else this is a miss, not a crash. */
        if (!value || !Array.isArray(value.docs)) return;
        indexDocs = value.docs;
        /* The loader is shared, so the caller decides what a fresh index
           means: the search re-runs the pending query, the random button picks
           a note. */
        done();
      })
      .catch(function () {
        /* Offline, a 404, or a body that is not JSON: search just stays
           empty. No logging, and no retry — `indexRequested` is already
           true, so a broken index is asked for exactly once. */
      });
  }

  /* ---- search ---------------------------------------------------------- */

  function initSearch() {
    var form = $(".site-search");
    var input = $("#site-search-input");
    var panel = $("#site-search-results");
    if (!form || !input || !panel) return;

    var optionSeq = 0;

    function normalize(value) {
      return (value || "").toString().toLowerCase().trim();
    }

    function escapeHtml(value) {
      return value
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;");
    }

    function highlight(text, terms) {
      var safe = escapeHtml(text);
      terms.forEach(function (term) {
        if (term.length < 2) return;
        var re = new RegExp("(" + term.replace(/[.*+?^${}()|[\]\\]/g, "\\$&") + ")", "gi");
        safe = safe.replace(re, "<mark>$1</mark>");
      });
      return safe;
    }

    function score(doc, query, terms) {
      var title = normalize(doc[2]);
      var body = normalize(doc[3]);
      var total = 0;
      if (title.indexOf(query) !== -1) total += 14;
      if (body.indexOf(query) !== -1) total += 4;
      terms.forEach(function (term) {
        if (title.indexOf(term) !== -1) total += 6;
        if (body.indexOf(term) !== -1) total += 1;
      });
      return total;
    }

    /* ---- combobox plumbing -------------------------------------------- */

    /* Everything ARIA-shaped is added here rather than in the markup, so
       without JavaScript the field stays a plain search input instead of
       claiming a listbox it cannot drive. */
    input.setAttribute("role", "combobox");
    input.setAttribute("aria-autocomplete", "list");
    input.setAttribute("aria-expanded", "false");
    input.setAttribute("aria-controls", panel.id);
    panel.setAttribute("aria-label", "Search results");

    var options = [];
    var activeId = "";

    var live = visuallyHiddenLive("visually-hidden");
    form.appendChild(live);

    function announce(message) {
      live.textContent = message;
    }

    /* The option ids have to be unique on the page, and they change on every
       render, so they are minted from a counter rather than reused. */
    function nextOptionId() {
      optionSeq += 1;
      return "site-search-option-" + optionSeq;
    }

    /* `aria-activedescendant` has to name an option that is really rendered:
       leaving it pointing at a row from the previous query tells assistive tech
       the wrong thing, so one that no longer matches an option is dropped. */
    function commit() {
      if (!activeId) {
        input.removeAttribute("aria-activedescendant");
        return;
      }
      var hit = options.find(function (option) {
        return option.id === activeId;
      });
      if (hit) input.setAttribute("aria-activedescendant", activeId);
      else input.removeAttribute("aria-activedescendant");
    }

    /* `-1` when the tracked row is not among the ones now rendered, which is
       how "nothing is highlighted" and "the highlight is stale" become the same
       answer. */
    function activeIndex() {
      for (var i = 0; i < options.length; i++) {
        if (options[i].id === activeId) return i;
      }
      return -1;
    }

    function highlight(index) {
      /* `-1` means "no row", which happens for an empty panel; it still has to
         clear the old state rather than leave the previous query's selection on
         screen. */
      if (index < 0) index = -1;

      activeId = "";
      options.forEach(function (option, i) {
        var on = i === index;
        option.classList.toggle("is-active", on);
        if (on) {
          activeId = option.id;
          option.setAttribute("aria-selected", "true");
        } else {
          /* Removed rather than set to "false": only the highlighted option
             should carry the attribute at all. */
          option.removeAttribute("aria-selected");
        }
      });

      var option = options[index];
      if (option && option.scrollIntoView) {
        /* `block: "nearest"` scrolls the panel but leaves the page alone, and
           reduced motion gets the jump without the animation. */
        option.scrollIntoView(
          reduceMotion ? { block: "nearest" } : { block: "nearest", behavior: "smooth" }
        );
      }

      commit();
    }

    /* Moving the highlight on hover keeps the two input methods agreeing: a
       click or an Enter should act on the row the reader is pointing at.
       `mouseenter` is bound per row because it does not bubble, and does not
       need to: it fires once on entry rather than again for each child the
       pointer crosses, so the highlight is set once per row. */
    function onHover() {
      var index = options.indexOf(this);
      if (index !== -1) highlight(index);
    }

    function move(step, wrap) {
      if (!options.length) return;
      var from = activeIndex();
      var next = from === -1 ? 0 : from + step;
      if (wrap) next = (next + options.length) % options.length;
      else next = Math.max(0, Math.min(options.length - 1, next));
      highlight(next);
    }

    function openPanel() {
      panel.hidden = false;
      input.setAttribute("aria-expanded", "true");
    }

    function clear() {
      panel.replaceChildren();
      panel.hidden = true;
      input.setAttribute("aria-expanded", "false");
      input.removeAttribute("aria-activedescendant");

      options = [];
      activeId = "";
      /* Announced even for an empty query, so tabbing away and back does not
         leave the last query's count standing. */
      announce("");
    }

    /* ---- near-miss suggestions ----------------------------------------- */

    /* Two rows and no allocation, which is plenty for the tokens on this site:
       the strings being compared are one word long, and the length guard below
       keeps the inner loop short. */
    function levenshtein(a, b) {
      if (a === b) return 0;
      if (!a.length) return b.length;
      if (!b.length) return a.length;

      var previous = [];
      var i;
      var j;
      for (j = 0; j <= b.length; j++) previous[j] = j;

      for (i = 1; i <= a.length; i++) {
        var current = [i];
        for (j = 1; j <= b.length; j++) {
          var cost = a.charCodeAt(i - 1) === b.charCodeAt(j - 1) ? 0 : 1;
          current[j] = Math.min(
            previous[j] + 1,
            current[j - 1] + 1,
            previous[j - 1] + cost
          );
        }
        previous = current;
      }

      return previous[b.length];
    }

    /* The suggestion is used as the look of a word, so punctuation and case are
       noise: `ESP32-C3` and `esp32` have to come out as the same token, and the
       space in `Cortex-M + Rust` has to end a token rather than join two. */
    function titleWords(title) {
      return normalize(title)
        .split(/[^\w\u00c0-\uffff]+/)
        .filter(Boolean);
    }

    /* One candidate token against one query term, cheap test first: the terms
       come from the reader, so a four-letter query must not be compared against
       every token in the corpus. A prefix hit is what catches a term the reader
       stopped typing early, and distance catches the typo. */
    function closeEnough(candidate, term) {
      if (Math.abs(candidate.length - term.length) > 2) return false;
      if (candidate.indexOf(term) === 0) return true;
      return levenshtein(candidate, term) <= 2;
    }

    function nearMissTitles(docs, query, terms) {
      var found = [];
      var seen = {};

      docs.some(function (doc) {
        if (found.length >= 3) return true;

        var title = doc[2];
        if (seen[normalize(title)]) return false;

        var hit = titleWords(title).some(function (word) {
          return terms.some(function (term) {
            return closeEnough(word, term);
          });
        });

        /* A reader who typed the whole title with one word misspelled produces
           a term no single token is close to, so the title is also tried as a
           prefix of the query: "read dht11" is a prefix of "read dht11
           temperature". */
        if (!hit) hit = normalize(title).indexOf(query) === 0;

        if (!hit) return false;
        seen[normalize(title)] = true;
        found.push(title);
        return false;
      });

      return found;
    }

    /* The fallback when nothing is even close: the sections themselves, which
       are few and always a true answer. `docs` is walked in index order and the
       labels de-duplicated, so the panel offers each section once. */
    function sectionSuggestions(docs, terms) {
      var found = [];
      var seen = {};

      docs.some(function (doc) {
        if (found.length >= 4) return true;

        var label = normalize(doc[1]);
        if (!label || seen[label]) return false;
        /* A section already named in the query is not a suggestion. */
        if (terms.some(function (term) {
          return label.indexOf(term) !== -1;
        })) {
          return false;
        }

        seen[label] = true;
        found.push(doc[1]);
        return false;
      });

      return found;
    }

    /* Clicking a suggestion is a shortcut for typing it: the field takes the
       text, the caret goes to the end where it would be after typing, and the
       search runs again so the panel shows the new answer immediately. Focus
       stays in the field, because the reader is mid-query, not reading the
       panel. */
    function resetInput(value) {
      input.value = value;
      input.focus();
      if (input.setSelectionRange) input.setSelectionRange(value.length, value.length);
      search();
    }

    /* ---- rendering ----------------------------------------------------- */

    function render(matches, query, terms) {
      panel.replaceChildren();
      options = [];
      activeId = "";

      /* A fresh panel holds no highlighted row, whatever the old one did. */
      input.removeAttribute("aria-activedescendant");

      if (!query) return clear();

      var frag = document.createDocumentFragment();
      var shown = matches.slice(0, 8);

      /* Restored here rather than at the top of the function, so the empty
         branch below can take it away again for its own content. */
      panel.setAttribute("role", "listbox");

      shown.forEach(function (doc) {
        var link = document.createElement("a");
        link.className = "search-result";
        link.href = doc[0];
        link.id = nextOptionId();
        link.setAttribute("role", "option");
        link.addEventListener("mouseenter", onHover);

        var title = document.createElement("strong");
        title.innerHTML = highlight(doc[2], terms);

        var path = document.createElement("span");
        path.textContent = doc[1] + " · " + doc[0];

        link.append(title, path);
        frag.appendChild(link);
      });

      if (!matches.length) {
        /* A listbox may only contain options, and this branch is about to put a
           paragraph and some buttons in it. Dropping the role for the empty
           state is what lets a screen reader reach those buttons at all; the
           listbox role goes back on above, where the options are. */
        panel.removeAttribute("role");

        /* The failure is dressed as a compiler diagnostic, because that is the
           voice of the rest of the site and a bare "no results" tells the
           reader nothing they cannot already see. The suggestion pass runs only
           here: on a query that found something there is nothing to suggest and
           no reason to pay for the distance calculations. */
        var suggestions = nearMissTitles(indexDocs, query, terms).map(function (title) {
          return { text: title, title: true };
        });

        if (!suggestions.length) {
          suggestions = sectionSuggestions(indexDocs, terms).map(function (label) {
            return { text: label, title: false };
          });
        }

        var empty = document.createElement("div");
        empty.className = "search-empty";

        var error = document.createElement("p");
        error.className = "search-empty__error";
        /* Built through `escapeHtml`, so a query full of angle brackets is
           printed as typed instead of becoming markup. */
        error.innerHTML = 'error[E0425]: no note matches "' + escapeHtml(query) + '"';

        var help = document.createElement("p");
        help.className = "search-empty__help";
        help.appendChild(document.createTextNode("help: "));

        if (suggestions.length) {
          /* One candidate reads as a question the compiler could have asked;
             several read better as a list of things to try. */
          help.appendChild(
            document.createTextNode(
              suggestions.length === 1 ? "did you mean " : "try "
            )
          );

          suggestions.forEach(function (suggestion, i) {
            if (i) help.appendChild(document.createTextNode(" · "));

            var button = document.createElement("button");
            button.type = "button";
            button.className = "search-suggest";
            /* The label is the full title, because that is what the field
               takes on click: the reader can then narrow it. The value is
               carried separately so the label can be styled — quoted, marked
               up — without those decorations leaking into the query. */
            button.textContent = suggestion.title
              ? '"' + suggestion.text + '"'
              : suggestion.text;
            button.setAttribute("data-suggest", suggestion.text);
            button.addEventListener("click", function () {
              resetInput(button.getAttribute("data-suggest"));
            });

            help.appendChild(button);
          });
        } else {
          help.appendChild(document.createTextNode("no note matches this query"));
        }

        empty.appendChild(error);
        empty.appendChild(help);
        frag.appendChild(empty);
      }

      panel.appendChild(frag);
      options = $$(".search-result", panel);
      openPanel();

      /* The first result is highlighted by default, so Enter always does the
         obvious thing without a trip through the arrow keys first. */
      if (options.length) highlight(0);
      else commit();

      announce(matches.length ? matches.length + " results" : "No matching notes");
    }

    function search() {
      var query = normalize(input.value);
      var terms = query.split(/\s+/).filter(Boolean);
      if (query.length < 2) return clear();

      /* Nothing to search yet: show the panel empty rather than "No matching
         notes", because that answer is not known to be true. The pending query
         is re-run through the loader's callback the moment the rows arrive. */
      if (indexDocs === null) return openPanel();

      var matches = indexDocs
        .map(function (doc) {
          return { doc: doc, score: score(doc, query, terms) };
        })
        .filter(function (m) {
          return m.score > 0;
        })
        .sort(function (a, b) {
          return b.score - a.score || a.doc[2].localeCompare(b.doc[2]);
        })
        .map(function (m) {
          return m.doc;
        });

      render(matches, query, terms);
    }

    /* Both listeners ask for the index; the loader has already seen the request
       on the first of them, so the second is a no-op. What the callback does is
       the caller's business — a query typed while the request was in flight is
       re-run here, which is the first honest answer it can get. */
    input.addEventListener("focus", function () {
      loadIndex(search);
    });
    input.addEventListener("input", function () {
      loadIndex(search);
    });
    input.addEventListener("input", search);
    input.addEventListener("focus", search);

    input.addEventListener("keydown", function (event) {
      if (event.key === "Escape") {
        input.value = "";
        clear();
        input.blur();
        return;
      }

      /* Alt is in here because Alt+Arrow is a word-jump in some browsers and on
         macOS, and neither should be mistaken for list navigation. */
      if (event.altKey || event.ctrlKey || event.metaKey) return;

      if (event.key === "ArrowDown") {
        event.preventDefault();
        move(1, true);
      } else if (event.key === "ArrowUp") {
        event.preventDefault();
        move(-1, true);
      } else if (event.key === "Home") {
        if (!options.length) return;
        event.preventDefault();
        highlight(0);
      } else if (event.key === "End") {
        if (!options.length) return;
        event.preventDefault();
        highlight(options.length - 1);
      } else if (event.key === "Enter") {
        /* The form has no `action`, so leaving this alone would reload the
           page; it is always ours to handle. */
        event.preventDefault();
        var index = activeIndex();
        if (index === -1) index = 0;
        var hit = options[index];
        if (hit) window.location.href = hit.href;
      }
    });

    form.addEventListener("submit", function (event) {
      event.preventDefault();
      var hit = panel.querySelector(".search-result");
      if (hit) window.location.href = hit.href;
    });

    document.addEventListener("click", function (event) {
      if (!form.contains(event.target)) clear();
    });

    document.addEventListener("keydown", function (event) {
      var typing =
        event.target &&
        (event.target.tagName === "INPUT" ||
          event.target.tagName === "TEXTAREA" ||
          event.target.isContentEditable);
      if (event.key === "/" && !typing) {
        event.preventDefault();
        input.focus();
      }
    });
  }

  /* ---- random note ----------------------------------------------------- */

  /* The button ships hidden because it cannot work without JavaScript: it is
     revealed here, which is the whole reason the markup does not render it
     visible for a reader who has no script. It stays a `<button>` rather than
     becoming a link to nowhere, and the markup keeps `data-random-note`.
   *
   * Picking a note needs the index, so the shared loader fetches it on the
   * first click if a search has not already done so. A click that cannot get
   * the rows does nothing at all: navigating to `undefined` would be a
   * broken page dressed up as a feature. */
  function initRandomNote() {
    var button = $("[data-random-note]");
    if (!button) return;

    button.hidden = false;

    button.addEventListener("click", function () {
      function go() {
        if (!indexDocs || !indexDocs.length) return;
        var pick = indexDocs[Math.floor(Math.random() * indexDocs.length)];
        if (pick && pick[0]) window.location.href = pick[0];
      }

      /* Already loaded, or the loader is mid-request: `indexRequested` is set
         only when a request really went out, so this joins the one in flight
         rather than starting a second. */
      if (indexDocs !== null) go();
      else loadIndex(go);
    });
  }

  /* ---- table of contents scroll spy ------------------------------------ */

  function initToc() {
    var toc = $(".toc");
    if (!toc) return;

    var links = $$("a", toc);
    if (!links.length) return;

    var marker = document.createElement("span");
    marker.className = "toc__marker";
    toc.appendChild(marker);

    var targets = links
      .map(function (link) {
        var id = decodeURIComponent(link.getAttribute("href").slice(1));
        var el = document.getElementById(id);
        return el ? { link: link, el: el } : null;
      })
      .filter(Boolean);

    if (!targets.length || typeof IntersectionObserver === "undefined") return;

    function mark(link) {
      links.forEach(function (l) {
        l.classList.toggle("is-active", l === link);
      });
      if (link) {
        marker.style.height = link.offsetHeight + "px";
        marker.style.transform = "translateY(" + link.offsetTop + "px)";
      }
    }

    var observer = new IntersectionObserver(
      function (entries) {
        var visible = entries
          .filter(function (e) {
            return e.isIntersecting;
          })
          .sort(function (a, b) {
            return a.boundingClientRect.top - b.boundingClientRect.top;
          });
        if (!visible.length) return;
        var hit = targets.find(function (t) {
          return t.el === visible[0].target;
        });
        if (hit) mark(hit.link);
      },
      { rootMargin: "-18% 0px -70% 0px", threshold: 0 }
    );

    targets.forEach(function (t) {
      observer.observe(t.el);
    });

    mark(links[0]);
  }

  /* ---- keyboard shortcuts ---------------------------------------------- */

  /* Built here rather than in the markup so a reader without JavaScript never
     sees a dialog they cannot open, which is the same reasoning as the ARIA on
     the search field.
   *
   * The overlay owns three things at once and they have to agree: focus, which
   * is moved in and trapped, `hidden`, and whether `focusTarget` still refers
   * to anything. `open` is the single source of truth for all three, so a
   * double open or a stray Escape cannot leave the page scrolled-locked with
   * the focus stranded in a hidden panel. */
  function initShortcuts() {
    if (!document.body) return;

    var shortcutRows = [
      { keys: "/", label: "Search notes" },
      { keys: "?", label: "This help" },
      { keys: "j", label: "Next note" },
      { keys: "k", label: "Previous note" },
      { keys: "Esc", label: "Close / clear" }
    ];

    var overlay = document.createElement("div");
    overlay.className = "shortcut-overlay";
    overlay.hidden = true;

    var panel = document.createElement("div");
    panel.className = "shortcut-panel";
    panel.setAttribute("role", "dialog");
    panel.setAttribute("aria-modal", "true");
    panel.setAttribute("aria-labelledby", "shortcut-title");

    var title = document.createElement("h2");
    title.id = "shortcut-title";
    title.textContent = "Keyboard shortcuts";

    var close = document.createElement("button");
    close.type = "button";
    close.className = "shortcut-close";
    close.setAttribute("aria-label", "Close");
    close.textContent = "×";

    var list = document.createElement("dl");
    list.className = "shortcut-list";

    shortcutRows.forEach(function (row) {
      var dt = document.createElement("dt");
      var kbd = document.createElement("kbd");
      /* `textContent` for the label and a `<kbd>` for the key: the pair is what
         a definition list is for, and the key has to be marked up so it can be
         styled as one. */
      kbd.textContent = row.keys;
      dt.appendChild(kbd);

      var dd = document.createElement("dd");
      dd.textContent = row.label;

      list.appendChild(dt);
      list.appendChild(dd);
    });

    panel.appendChild(title);
    panel.appendChild(close);
    panel.appendChild(list);
    overlay.appendChild(panel);
    document.body.appendChild(overlay);

    var open = false;
    var focusTarget = null;
    var bodyOverflow = "";

    function focusables() {
      return $$("button, [href], input, select, textarea, [tabindex]", panel).filter(
        function (el) {
          return (
            !el.hasAttribute("disabled") &&
            el.getAttribute("tabindex") !== "-1" &&
            el.getAttribute("aria-hidden") !== "true"
          );
        }
      );
    }

    function openOverlay() {
      if (open) return;
      open = true;

      /* Whatever had focus keeps the right to it back, but only if it is still
         on the page: an element that has been replaced cannot take focus, and
         leaving focus on `<body>` is the honest fallback. */
      focusTarget = document.activeElement;

      overlay.hidden = false;
      /* Opening by keyboard must land on the close button, which is also the
         only control in the panel, so one Tab reaches the rest of the page and
         Escape is always one keystroke away. */
      close.focus();

      /* The page behind a modal must not scroll under it. The previous value is
         kept exactly, including the empty string a browser reports for "not
         set inline", so closing restores whatever the page had. */
      bodyOverflow = document.body.style.overflow;
      document.body.style.overflow = "hidden";
    }

    function closeOverlay() {
      if (!open) return;
      open = false;

      overlay.hidden = true;
      document.body.style.overflow = bodyOverflow;

      var previous = focusTarget;
      focusTarget = null;
      if (previous && previous.isConnected !== false && previous.focus) previous.focus();
    }

    overlay.addEventListener("click", function (event) {
      if (event.target === overlay) closeOverlay();
    });

    close.addEventListener("click", closeOverlay);

    overlay.addEventListener("keydown", function (event) {
      if (event.key === "Escape") {
        event.preventDefault();
        closeOverlay();
        return;
      }

      /* Tab is trapped rather than ignored: the page behind is inert while the
         dialog is up, so focus must not wander out of it. The panel's own
         controls are queried on demand because the close button is the only
         one today, but a row added later is picked up without changes here. */
      if (event.key !== "Tab") return;

      var items = focusables();
      if (!items.length) return;

      var first = items[0];
      var last = items[items.length - 1];
      var active = document.activeElement;

      event.preventDefault();

      if (event.shiftKey) {
        var back = active === first || items.indexOf(active) === -1 ? last : items[items.indexOf(active) - 1];
        back.focus();
      } else {
        var forward = active === last || items.indexOf(active) === -1 ? first : items[items.indexOf(active) + 1];
        forward.focus();
      }
    });

    /* One document-level keydown for everything that is not the search field,
       so the three shortcuts cannot disagree about what "typing" means. The
       search field stops its own keys from bubbling into the `j`/`k` branch
       only by virtue of being an input, which `isEditable` catches. */
    document.addEventListener("keydown", function (event) {
      if (open) {
        /* The panel's own handler has already decided what Tab means inside
           it; Escape is handled there too, so anything arriving here while the
           overlay is up is meant for it. */
        if (event.key === "Escape") {
          event.preventDefault();
          closeOverlay();
        }
        return;
      }

      if (hasModifier(event) || isEditable(event.target)) return;

      if (event.key === "?") {
        event.preventDefault();
        openOverlay();
        return;
      }

      /* A held key sends a fresh keydown per repeat, which would walk the
         reader through several notes; `repeat` is the browser's own way of
         saying this is the same press. */
      if (event.repeat) return;

      if (event.key === "j" || event.key === "k") {
        var nav = $(".doc-nav");
        var link = nav && $(event.key === "j" ? ".next" : ".prev", nav);
        var href = link && link.getAttribute("href");
        /* No link on this page, or a bare "#": there is nowhere to go, and
           doing nothing is the correct answer. */
        if (!href || href === "#") return;
        event.preventDefault();
        window.location.href = href;
      }
    });
  }

  /* ---- console --------------------------------------------------------- */

  /* The only piece of this file that is purely for the reader who opens the
     console, so it is the only piece that is allowed to be cut: `console` can
     be missing or `undefined` in old or embedded browsers, and a `TypeError` at
     boot would take the rest of the page's enhancements down with it. */
  function initConsole() {
    if (typeof console === "undefined" || !console.log) return;

    console.log(
      "%cref42%c — a Rust embedded notebook",
      "color:#ff2d2d;font-weight:bold",
      ""
    );
    console.log("/ search · ? shortcuts · j/k next/previous note");
  }

  /* ---- boot ------------------------------------------------------------ */

  function boot() {
    initNav();
    initScrollChrome();
    initAnchors();
    initReveal();
    initCodeBlocks();
    initSearch();
    initRandomNote();
    initShortcuts();
    initToc();
    initConsole();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }
})();
