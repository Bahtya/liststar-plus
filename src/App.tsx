import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./App.css";

interface SearchResult {
  filename: string;
  path: string;
}

function App() {
  const [keyword, setKeyword] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [searchTime, setSearchTime] = useState(0);
  const [totalFiles, setTotalFiles] = useState(0);
  const [usnStatus, setUsnStatus] = useState("Idle");
  const [indexingProgress, setIndexingProgress] = useState<string | null>(null);

  const inputRef = useRef<HTMLInputElement>(null);
  const resultsRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    inputRef.current?.focus();

    // Listen for backend events
    const unlistenProgress = listen<string>("indexing-progress", (event) => {
      setIndexingProgress(event.payload);
    });

    const unlistenUsn = listen<string>("usn-status", (event) => {
      setUsnStatus(event.payload);
    });

    const unlistenCount = listen<number>("file-count", (event) => {
      setTotalFiles(event.payload);
    });

    // Hide window on blur
    const unlistenBlur = getCurrentWindow().listen("tauri://blur", () => {
      getCurrentWindow().hide();
    });

    // Initial index build if needed (or just get status)
    const init = async () => {
      try {
        // Trigger a build for C: on startup
        await invoke("build_index", { roots: ["C:\\"] });
        await invoke("start_usn_monitoring", { driveLetter: 'C' });
      } catch (e) {
        console.error("Failed to initialize:", e);
      }
    };
    init();

    return () => {
      unlistenProgress.then((f: UnlistenFn) => f());
      unlistenUsn.then((f: UnlistenFn) => f());
      unlistenCount.then((f: UnlistenFn) => f());
      unlistenBlur.then((f: UnlistenFn) => f());
    };
  }, []);

  const handleSearch = async (val: string) => {
    setKeyword(val);
    setSelectedIndex(0);
    if (val.trim().length === 0) {
      setResults([]);
      setSearchTime(0);
      return;
    }

    const start = performance.now();
    try {
      const res: SearchResult[] = await invoke("search", { keyword: val, limit: 100 });
      setResults(res);
      setSearchTime(Math.round(performance.now() - start));
    } catch (e) {
      console.error(e);
    }
  };

  const handleKeyDown = async (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIndex((prev) => Math.min(prev + 1, results.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIndex((prev) => Math.max(prev - 1, 0));
    } else if (e.key === "Enter") {
      if (results[selectedIndex]) {
        openFile(results[selectedIndex].path);
      }
    } else if (e.key === "Escape") {
      getCurrentWindow().hide();
    }
  };

  const openFile = async (path: string) => {
    try {
      await invoke("plugin:opener|open", { path });
      getCurrentWindow().hide();
    } catch (e) {
      console.error("Failed to open file:", e);
    }
  };

  // Scroll selected item into view
  useEffect(() => {
    const selectedElement = resultsRef.current?.querySelector(".selected-item");
    if (selectedElement) {
      selectedElement.scrollIntoView({ block: "nearest" });
    }
  }, [selectedIndex]);

  return (
    <div className="flex items-center justify-center w-screen h-screen bg-transparent overflow-hidden" onKeyDown={handleKeyDown}>
      <div className="w-[700px] max-h-[500px] flex flex-col bg-slate-50/95 backdrop-blur-xl rounded-2xl shadow-2xl border border-white/20 overflow-hidden">
        {/* Search Input */}
        <div className="flex items-center px-6 py-4 border-b border-slate-200/50">
          <svg className="w-6 h-6 text-slate-400 mr-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <input
            ref={inputRef}
            className="flex-1 bg-transparent text-2xl text-slate-800 placeholder-slate-400 outline-none border-none"
            value={keyword}
            onChange={(e) => handleSearch(e.target.value)}
            placeholder="Search files..."
            spellCheck={false}
          />
        </div>

        {/* Results List */}
        <div className="flex-1 overflow-y-auto custom-scrollbar" ref={resultsRef}>
          {results.length > 0 ? (
            <div className="py-2">
              {results.map((res, i) => (
                <div
                  key={i}
                  className={`flex items-center px-6 py-3 cursor-pointer transition-colors ${
                    i === selectedIndex ? "bg-blue-500/10 selected-item" : "hover:bg-slate-100/50"
                  }`}
                  onClick={() => setSelectedIndex(i)}
                  onDoubleClick={() => openFile(res.path)}
                >
                  <div className="w-10 h-10 flex items-center justify-center bg-white rounded-lg shadow-sm border border-slate-200 mr-4">
                    <svg className="w-6 h-6 text-slate-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z" />
                    </svg>
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className={`text-base font-medium truncate ${i === selectedIndex ? "text-blue-600" : "text-slate-700"}`}>
                      {res.filename}
                    </div>
                    <div className="text-xs text-slate-400 truncate">
                      {res.path}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          ) : keyword && (
            <div className="flex flex-col items-center justify-center py-12 text-slate-400">
              <svg className="w-12 h-12 mb-4 opacity-20" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.172 9.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <p>No items match your search.</p>
            </div>
          )}
        </div>

        {/* Status Bar */}
        <div className="px-4 py-1.5 bg-slate-100/50 border-t border-slate-200/50 flex items-center justify-between text-[10px] text-slate-400 uppercase tracking-wider font-semibold">
          <div className="flex items-center space-x-4">
            <span>{results.length} Objects</span>
            {searchTime > 0 && <span>{searchTime}ms</span>}
            <span>USN: {usnStatus}</span>
          </div>
          <div className="flex items-center space-x-4">
            {indexingProgress && <span>Indexing: {indexingProgress}</span>}
            {totalFiles > 0 && <span>Total: {totalFiles.toLocaleString()}</span>}
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
