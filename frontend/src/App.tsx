// The editor shell: viewport, timeline, layer list, inspector (PLAN.md §12
// Phase 3). State here is *derived* from the Rust engine — the UI dispatches
// commands and re-pulls summaries / re-scrubs the viewport. It never mutates a
// local source of truth.

import { useCallback, useEffect, useState } from "react";
import {
  applyEdit,
  openProject,
  redo,
  renderBackend,
  scrub,
  undo,
  type ProjectSummary,
} from "./api";

export default function App() {
  const [project, setProject] = useState<ProjectSummary | null>(null);
  const [time, setTime] = useState(0);
  const [frameUrl, setFrameUrl] = useState<string | null>(null);
  const [selected, setSelected] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [backend, setBackend] = useState<string>("");

  useEffect(() => {
    renderBackend().then(setBackend).catch(() => setBackend(""));
  }, []);

  const refreshFrame = useCallback(async (t: number) => {
    try {
      setFrameUrl(await scrub(t));
    } catch (e) {
      setError(String(e));
    }
  }, []);

  const onScrub = useCallback(
    (t: number) => {
      setTime(t);
      void refreshFrame(t);
    },
    [refreshFrame],
  );

  useEffect(() => {
    if (project) void refreshFrame(time);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [project]);

  async function onOpen() {
    try {
      // A real build wires the Tauri dialog plugin; this default path matches
      // `creator sample demo.ctor`.
      setProject(await openProject("demo.ctor"));
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }

  async function reload() {
    // Re-pull the project summary after an edit (a real build would apply the
    // patch the backend emits instead of refetching).
    if (project) void refreshFrame(time);
  }

  return (
    <div className="app">
      <header className="toolbar">
        <button onClick={onOpen}>Open</button>
        <button onClick={async () => (await undo(), reload())}>Undo</button>
        <button onClick={async () => (await redo(), reload())}>Redo</button>
        <span className="title">{project?.name ?? "No project"}</span>
        {backend && <span className="badge">{backend === "vulkan" ? "GPU · Vulkan" : "CPU"}</span>}
        {error && <span className="error">{error}</span>}
      </header>

      <div className="body">
        <aside className="layers">
          <h2>Layers</h2>
          <ul>
            {project?.layers.map((l, i) => (
              <li
                key={i}
                className={selected === i ? "selected" : ""}
                onClick={() => setSelected(i)}
              >
                <input
                  type="checkbox"
                  checked={l.enabled}
                  onChange={async (e) => {
                    await applyEdit({
                      type: "SetEnabled",
                      layer_index: i,
                      enabled: e.target.checked,
                    });
                    await reload();
                  }}
                />
                {l.name}
              </li>
            ))}
          </ul>
        </aside>

        <main className="viewport">
          {/* Tier-3 readback: the Rust-rendered frame as an image. The target
              state composites a native GPU surface here instead (PLAN.md §7). */}
          {frameUrl ? (
            <img src={frameUrl} alt="viewport" />
          ) : (
            <div className="placeholder">Open a project to render</div>
          )}
        </main>

        <aside className="inspector">
          <h2>Inspector</h2>
          {selected !== null && project ? (
            <Inspector
              index={selected}
              name={project.layers[selected]?.name ?? ""}
              onEdit={reload}
            />
          ) : (
            <p>Select a layer</p>
          )}
        </aside>
      </div>

      <footer className="timeline">
        <input
          type="range"
          min={0}
          max={project?.duration ?? 0}
          step={1 / (project?.frame_rate ?? 30)}
          value={time}
          onChange={(e) => onScrub(parseFloat(e.target.value))}
        />
        <span>
          {time.toFixed(2)}s / {project?.duration ?? 0}s
        </span>
      </footer>
    </div>
  );
}

function Inspector(props: { index: number; name: string; onEdit: () => void }) {
  const [opacity, setOpacity] = useState(1);
  return (
    <div>
      <label>
        Name
        <input
          defaultValue={props.name}
          onBlur={async (e) => {
            await applyEdit({ type: "Rename", layer_index: props.index, name: e.target.value });
            props.onEdit();
          }}
        />
      </label>
      <label>
        Opacity
        <input
          type="range"
          min={0}
          max={1}
          step={0.01}
          value={opacity}
          onChange={async (e) => {
            const v = parseFloat(e.target.value);
            setOpacity(v);
            await applyEdit({ type: "SetOpacity", layer_index: props.index, opacity: v });
            props.onEdit();
          }}
        />
      </label>
    </div>
  );
}
