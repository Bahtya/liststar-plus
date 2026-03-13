import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
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
      unlistenProgress.then(f => f());
      unlistenUsn.then(f => f());
      unlistenCount.then(f => f());
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
    }
  };

  const openFile = async (path: string) => {
    try {
      await invoke("plugin:opener|open", { path });
    } catch (e) {
      console.error("Failed to open file:", e);
    }
  };

  const copyPath = (path: string) => {
    navigator.clipboard.writeText(path);
    alert("Path copied to clipboard!");
  };

  const openFolder = async (path: string) => {
    try {
      // On Windows, we can use explorer.exe /select,path
      // But for simplicity with plugin-opener, we just open the parent folder
      const parentPath = path.substring(0, path.lastIndexOf("\\"));
      await invoke("plugin:opener|open", { path: parentPath });
    } catch (e) {
      console.error("Failed to open folder:", e);
    }
  };

  const handleContextMenu = (e: React.MouseEvent, path: string) => {
    e.preventDefault();
    // Simple native-like context menu simulation or just use window.confirm for now
    // In a real app, we'd use a custom React component
    const action = window.prompt("Choose action: 1. Copy Path, 2. Open Folder", "1");
    if (action === "1") copyPath(path);
    if (action === "2") openFolder(path);
  };

  // Scroll selected item into view
  useEffect(() => {
    const selectedElement = resultsRef.current?.querySelector(".selected");
    if (selectedElement) {
      selectedElement.scrollIntoView({ block: "nearest" });
    }
  }, [selectedIndex]);

  return (
    <div className="app-container" onKeyDown={handleKeyDown}>
      <div className="search-container">
        <input
          ref={inputRef}
          className="search-input"
          value={keyword}
          onChange={(e) => handleSearch(e.target.value)}
          placeholder="Search files..."
          spellCheck={false}
        />
      </div>

      <div className="results-container" ref={resultsRef}>
        <table className="results-table">
          <thead>
            <tr>
              <th className="filename-cell">Name</th>
              <th className="path-cell">Path</th>
            </tr>
          </thead>
          <tbody>
            {results.map((res, i) => (
              <tr
                key={i}
                className={`result-item ${i === selectedIndex ? "selected" : ""}`}
                onClick={() => setSelectedIndex(i)}
                onDoubleClick={() => openFile(res.path)}
                onContextMenu={(e) => handleContextMenu(e, res.path)}
              >
                <td className="filename-cell">{res.filename}</td>
                <td className="path-cell">{res.path}</td>
              </tr>
            ))}
          </tbody>
        </table>
        {results.length === 0 && keyword && (
          <div className="no-results">No items match your search.</div>
        )}
      </div>

      <div className="status-bar">
        <div className="status-item">
          {results.length} objects
        </div>
        {searchTime > 0 && (
          <div className="status-item">
            Search took {searchTime} ms
          </div>
        )}
        <div className="status-item">
          USN: {usnStatus}
        </div>
        {indexingProgress && (
          <div className="status-item">
            Indexing: {indexingProgress}
          </div>
        )}
        {totalFiles > 0 && (
          <div className="status-item">
            Total: {totalFiles.toLocaleString()} files
          </div>
        )}
      </div>
    </div>
  );
}

export default App;
