window.programmaticUpdate = false;

window.addEventListener("load", () => {
  console.log("Cytoscape version:", cytoscape.version);
  console.log("Cytoscape loaded successfully");

  // Set up graph theme immediately - don't wait for CodeMirror
  window.graphTheme = {
    defaultNodeColor: "#00d3bb",
    activeNodeColor: "#fdcf2b",
    edgeColor: "#ccc",
    edgeHighlightColor: "#ffaf80",
    nodeTextColor: "white",
    edgeTextBackgroundColor: "white",
  };

  console.log("Graph theme initialized");

  // Initialize CodeMirror-based syntax highlighting
  if (typeof CodeMirror !== "undefined") {
    console.log("CodeMirror loaded successfully");

    // Define Turing machine mode for CodeMirror
    CodeMirror.defineMode("turing", function () {
      return {
        startState: function () {
          return {
            inName: false,
            inTape: false,
          };
        },
        token: function (stream, state) {
          // Reset state at the beginning of a new line
          if (stream.sol()) {
            state.inName = false;
            state.inTape = false;
          }

          // Comments
          if (stream.match(/^#.*/)) {
            return "comment";
          }

          // Consume any leading whitespace before other tokens
          if (stream.eatSpace()) {
            return null;
          }

          // Section headers
          if (
            stream.match(
              /^(name|head|heads|blank|tape|tapes|states|rules):/,
            )
          ) {
            const matched = stream.current();
            if (matched.startsWith("name:")) {
              state.inName = true;
            } else if (
              matched.startsWith("tape:") ||
              matched.startsWith("tapes:")
            ) {
              state.inTape = true;
            }
            return "keyword";
          }

          // Handle content after 'name:'
          if (state.inName) {
            if (stream.eatSpace()) {
              return null;
            }
            stream.skipToEnd();
            return "string";
          }

          // Handle content after 'tape:' or 'tapes:'
          if (state.inTape) {
            if (stream.match(/'[^']*'/)) {
              // Quoted symbols
              return "string";
            }
            // Unquoted symbols: match any single character that is not a reserved character
            // Reserved characters: #, space, comma, >, <
            if (stream.match(/^[^#\s,><]/)) {
              return "number"; // Treat as number as per user request
            }
            if (stream.match(/[,|]/)) {
              // Separators
              return "punctuation";
            }
            // If nothing matched, consume character and continue
            stream.next();
            return null;
          }

          // Arrows
          if (stream.match(/->/)) {
            return "operator";
          }

          // Directions
          if (stream.match(/\b[LRS<>]\b/)) {
            return "atom";
          }

          // State names (definitions or within transitions)
          // Match an identifier that starts with non-digit and contains alphanumeric/underscore.
          // This needs to be before general symbol matching to prioritize state names.
          if (stream.match(/^[a-zA-Z_][a-zA-Z0-9_]*/)) {
            // Check if it's a state definition (ends with ':')
            if (stream.peek() === ":") {
              stream.next(); // Consume the colon
              return "def"; // State definition
            }
            return "def"; // State name within a transition
          }

          // Quoted symbols (general, e.g., 'a', '$')
          if (stream.match(/'[^']*'/)) {
            return "string";
          }

          // Numbers
          if (stream.match(/\b\d+\b/)) {
            return "number";
          }

          // Brackets
          if (stream.match(/[\[\]]/)) {
            return "bracket";
          }

          // Comma (as punctuation)
          if (stream.match(/,/)) {
            return "punctuation";
          }

          // Unquoted symbols (general)
          // Match a single character that is not a reserved character or part of other tokens.
          // Reserved characters from grammar.pest: #, space, comma, >, <
          // Also exclude: [, ] (handled by other rules)
          // So, match any character NOT in: #, \s, ,, >, <, [, ]
          if (stream.match(/^[^#\s,><\[\]]/)) {
            return "number"; // User requested to treat symbols as "number"
          }

          // Default: consume character and return null (no special styling)
          stream.next();
          return null;
        },
      };
    });

    // Function to initialize CodeMirror
    const initCodeMirror = () => {
      const textarea = document.getElementById("turing-program-editor");
      if (textarea && !textarea.dataset.codemirrorInitialized) {
        console.log("Initializing CodeMirror for textarea");

        const editor = CodeMirror.fromTextArea(textarea, {
          mode: "turing",
          theme: "cobalt",
          lineNumbers: false,
          lineWrapping: true,
          indentUnit: 2,
          tabSize: 2,
          extraKeys: {
            Tab: function (cm) {
              cm.replaceSelection("  ");
            },
          },
        });

        window.codeMirrorEditor = editor;

        // Sync with Yew component
        editor.on("change", function () {
          if (window.programmaticUpdate) {
            window.programmaticUpdate = false; // Reset flag after programmatic update is handled
            return; // Do not dispatch input event for programmatic changes
          }
          textarea.value = editor.getValue();
          textarea.dispatchEvent(new Event("input", { bubbles: true }));
        });

        // Ensure the editor is properly synced with the textarea's initial value
        // This prevents cursor jumping on first edit after initialization
        const initialValue = textarea.value;
        if (initialValue && editor.getValue() !== initialValue) {
          window.programmaticUpdate = true;
          editor.setValue(initialValue);
        }

        textarea.dataset.codemirrorInitialized = "true";
        console.log("CodeMirror initialized successfully");
      }
    };

    window.updateCodeMirrorValue = (newValue) => {
      if (window.codeMirrorEditor) {
        // Only update if the value is actually different
        if (window.codeMirrorEditor.getValue() !== newValue) {
          window.programmaticUpdate = true; // Set flag before programmatic update
          window.codeMirrorEditor.setValue(newValue);
        }
      } else {
        // If CodeMirror isn't initialized yet, update the textarea directly
        // This ensures the value is there when CodeMirror initializes
        const textarea = document.getElementById("turing-program-editor");
        if (textarea) {
          textarea.value = newValue;
        }
      }
    };

    // Try to initialize immediately
    setTimeout(initCodeMirror, 500);

    // Also watch for DOM changes
    const observer = new MutationObserver(() => {
      initCodeMirror();
    });

    observer.observe(document.body, {
      childList: true,
      subtree: true,
    });
  } else {
    console.error("CodeMirror failed to load");
  }
});
