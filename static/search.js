(function () {
  var form = document.querySelector(".site-search");
  var input = document.getElementById("site-search-input");
  var results = document.getElementById("site-search-results");

  if (!form || !input || !results) return;

  var documents = [];

  function normalize(value) {
    return (value || "").toString().toLowerCase().trim();
  }

  function localPath(id) {
    try {
      return new URL(id).pathname;
    } catch (_) {
      return id || "/";
    }
  }

  function loadDocuments() {
    if (documents.length) return documents;
    var index = window.searchIndex;
    var store = index && index.documentStore && index.documentStore.docs;
    if (!store) return documents;

    documents = Object.keys(store)
      .map(function (key) {
        return store[key];
      })
      .filter(function (doc) {
        return doc && doc.title && doc.id && localPath(doc.id) !== "/";
      });

    return documents;
  }

  function score(doc, query, terms) {
    var title = normalize(doc.title);
    var body = normalize(doc.body);
    var total = 0;

    if (title.indexOf(query) !== -1) total += 12;
    if (body.indexOf(query) !== -1) total += 4;

    terms.forEach(function (term) {
      if (title.indexOf(term) !== -1) total += 5;
      if (body.indexOf(term) !== -1) total += 1;
    });

    return total;
  }

  function clear() {
    results.replaceChildren();
    results.hidden = true;
    input.setAttribute("aria-expanded", "false");
  }

  function render(matches, query) {
    results.replaceChildren();

    if (!query) {
      clear();
      return;
    }

    if (!matches.length) {
      var empty = document.createElement("div");
      empty.className = "search-empty";
      empty.textContent = "No matches";
      results.appendChild(empty);
    }

    matches.slice(0, 8).forEach(function (doc) {
      var link = document.createElement("a");
      var title = document.createElement("strong");
      var path = document.createElement("span");

      link.className = "search-result";
      link.href = localPath(doc.id);
      link.setAttribute("role", "option");
      title.textContent = doc.title;
      path.textContent = localPath(doc.id);

      link.append(title, path);
      results.appendChild(link);
    });

    results.hidden = false;
    input.setAttribute("aria-expanded", "true");
  }

  function search() {
    var query = normalize(input.value);
    var terms = query.split(/\s+/).filter(Boolean);

    if (query.length < 2) {
      clear();
      return;
    }

    var matches = loadDocuments()
      .map(function (doc) {
        return { doc: doc, score: score(doc, query, terms) };
      })
      .filter(function (match) {
        return match.score > 0;
      })
      .sort(function (a, b) {
        return b.score - a.score || a.doc.title.localeCompare(b.doc.title);
      })
      .map(function (match) {
        return match.doc;
      });

    render(matches, query);
  }

  input.addEventListener("input", search);

  input.addEventListener("keydown", function (event) {
    if (event.key === "Escape") {
      input.value = "";
      clear();
    }
  });

  form.addEventListener("submit", function (event) {
    event.preventDefault();
    var first = results.querySelector(".search-result");
    if (!first) return;
    first.click();
  });

  document.addEventListener("click", function (event) {
    if (!form.contains(event.target)) clear();
  });
})();
