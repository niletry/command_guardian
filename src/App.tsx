import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Play, Square, Trash2, Terminal as TerminalIcon, Plus, LayoutGrid, Activity, Edit2 } from "lucide-react";
import type { TaskView } from "./types";
import { TerminalView } from "./components/TerminalView";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

function formatDuration(seconds: number | null) {
  if (!seconds) return "—";
  const now = Math.floor(Date.now() / 1000);
  const diff = now - seconds;
  if (diff < 0) return "0s";
  if (diff < 60) return `${diff}s`;
  const m = Math.floor(diff / 60);
  if (m < 60) return `${m}m ${diff % 60}s`;
  const h = Math.floor(m / 60);
  return `${h}h ${m % 60}m`;
}

export default function App() {
  const [tasks, setTasks] = useState<TaskView[]>([]);
  const [filter, setFilter] = useState("All");
  const [terminalTask, setTerminalTask] = useState<TaskView | null>(null);
  
  // Add Task Form State
  const [isAddOpen, setIsAddOpen] = useState(false);
  const [editingTask, setEditingTask] = useState<TaskView | null>(null);
  const [newName, setNewName] = useState("");
  const [newCmd, setNewCmd] = useState("");
  const [newTag, setNewTag] = useState("Default");
  const [newEnv, setNewEnv] = useState("");
  const [autoRetry, setAutoRetry] = useState(false);

  const refreshTasks = async () => {
    try {
      const data = await invoke<TaskView[]>("get_tasks");
      setTasks(data);
    } catch (err) {
      console.error(">>> UI: Failed to fetch tasks:", err);
    }
  };

  useEffect(() => {
    refreshTasks();
    const unlisten = listen("task-updated", (e) => {
        console.log(">>> UI: Received task-updated event for:", e.payload);
        refreshTasks();
    });
    
    const interval = setInterval(() => {
        setTasks(prev => [...prev]);
    }, 1000);

    return () => {
      unlisten.then(f => f());
      clearInterval(interval);
    };
  }, []);

  const handleStart = async (id: string) => {
    try {
      await invoke("start_task", { id });
    } catch (err) {
      console.error(">>> UI: Start failed:", err);
      alert("Failed to start task: " + err);
    }
  };

  const handleStop = async (id: string) => {
    try {
      await invoke("stop_task", { id });
    } catch (err) {
      console.error(">>> UI: Stop failed:", err);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm("Are you sure you want to delete this task? This will also delete its logs.")) return;
    try {
      await invoke("delete_task", { id });
      await refreshTasks();
    } catch (err) {
      console.error(">>> UI: Delete failed:", err);
    }
  };

  const handleCreateOrUpdate = async () => {
    if (!newName || !newCmd) return;

    const env_vars: Record<string, string> = {};
    newEnv.split('\n').forEach(line => {
      const parts = line.split('=');
      if (parts.length >= 2) {
        const key = parts[0].trim();
        const value = parts.slice(1).join('=').trim();
        if (key) env_vars[key] = value;
      }
    });

    try {
      if (editingTask) {
          await invoke("update_task", {
            id: editingTask.config.id,
            name: newName,
            command: newCmd,
            tag: newTag,
            autoRetry: autoRetry,
            envVars: Object.keys(env_vars).length > 0 ? env_vars : null
          });
      } else {
          await invoke("create_task", { 
            name: newName, 
            command: newCmd, 
            tag: newTag, 
            autoRetry: autoRetry,
            envVars: Object.keys(env_vars).length > 0 ? env_vars : null
          });
        }
        
        setIsAddOpen(false);
        setEditingTask(null);
        setNewName("");
        setNewCmd("");
        setNewEnv("");
        setAutoRetry(false);
        await refreshTasks();
    } catch (err) {
        console.error(">>> UI: Save failed:", err);
        alert("Save failed: " + err);
    }
  };

  const startEdit = (task: TaskView) => {
    setEditingTask(task);
    setNewName(task.config.name);
    setNewCmd(task.config.command);
    setNewTag(task.config.tag);
    setAutoRetry(task.config.auto_retry);
    const envStr = Object.entries(task.config.env_vars || {})
      .map(([k, v]) => `${k}=${v}`)
      .join('\n');
    setNewEnv(envStr);
    setIsAddOpen(true);
  };

  const closeAddModal = () => {
    setIsAddOpen(false);
    setEditingTask(null);
    setNewName("");
    setNewCmd("");
    setNewEnv("");
    setAutoRetry(false);
  };

  const uniqueTags = ["All", ...Array.from(new Set(tasks.map(t => t.config.tag))).sort()];
  const filteredTasks = filter === "All" ? tasks : tasks.filter(t => t.config.tag === filter);

  return (
    <div className="flex h-screen bg-neutral-900 text-neutral-100 font-sans selection:bg-blue-500/30 overflow-hidden">
      {/* Sidebar */}
      <div className="w-64 border-r border-neutral-800 p-4 flex flex-col gap-6 bg-neutral-950/50 backdrop-blur-xl">
        <div className="flex items-center gap-3 px-2">
          <div className="w-8 h-8 rounded-lg bg-blue-600 flex items-center justify-center shadow-lg shadow-blue-900/20">
            <Activity className="w-5 h-5 text-white" />
          </div>
          <div>
            <h1 className="font-bold text-lg tracking-tight">Guardian</h1>
            <p className="text-xs text-neutral-500">Process Manager</p>
          </div>
        </div>

        <div className="flex flex-col gap-1 overflow-y-auto pr-2">
          <p className="text-xs font-medium text-neutral-500 px-2 mb-2 uppercase tracking-wider">Filters</p>
          {uniqueTags.map(tag => (
            <button
              key={tag}
              onClick={() => setFilter(tag)}
              className={cn(
                "flex items-center gap-2 px-3 py-2 rounded-md text-sm transition-all duration-200",
                filter === tag 
                  ? "bg-blue-600/10 text-blue-400 font-medium" 
                  : "text-neutral-400 hover:bg-neutral-800 hover:text-neutral-200"
              )}
            >
              <LayoutGrid className="w-4 h-4 opacity-70" />
              <span className="truncate flex-1 text-left">{tag}</span>
              {tag !== "All" && (
                <span className="text-xs bg-neutral-800 px-1.5 py-0.5 rounded-full text-neutral-500">
                  {tasks.filter(t => t.config.tag === tag).length}
                </span>
              )}
            </button>
          ))}
        </div>

        <div className="mt-auto pt-4">
            <button 
                onClick={() => setIsAddOpen(true)}
                className="w-full flex items-center justify-center gap-2 bg-neutral-100 text-neutral-900 hover:bg-white py-2.5 rounded-lg font-medium transition-colors shadow-lg shadow-white/5"
            >
                <Plus className="w-4 h-4" />
                New Task
            </button>
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 flex flex-col h-full overflow-hidden bg-neutral-900">
        <div className="p-8 overflow-y-auto h-full pb-32">
          <div className="flex items-center justify-between mb-8">
            <h2 className="text-2xl font-bold">Dashboard <span className="text-neutral-500 text-lg font-normal ml-2">/ {filter}</span></h2>
            <div className="text-sm text-neutral-500 font-mono">
                {tasks.filter(t => t.status.status === 'running').length} Running
            </div>
          </div>

          <div className="grid grid-cols-1 xl:grid-cols-2 gap-4">
            {filteredTasks.map(task => (
              <div key={task.config.id} className="group bg-neutral-800/40 border border-neutral-700/50 rounded-xl p-5 hover:border-neutral-600/50 transition-all duration-300 hover:bg-neutral-800/60 shadow-sm">
                <div className="flex justify-between items-start mb-4">
                  <div className="overflow-hidden flex-1 mr-4">
                    <div className="flex items-center gap-2 mb-1 overflow-hidden">
                        <h3 className="font-semibold text-lg truncate">{task.config.name}</h3>
                        <span className="shrink-0 text-[10px] px-2 py-0.5 rounded-full bg-neutral-700 text-neutral-400 font-medium border border-neutral-600">
                            {task.config.tag}
                        </span>
                    </div>
                    <code className="text-xs text-neutral-500 font-mono block truncate" title={task.config.command}>
                      $ {task.config.command}
                    </code>
                  </div>
                  <div className={cn(
                    "shrink-0 px-2.5 py-1 rounded-full text-xs font-bold tracking-wide flex items-center gap-1.5 border",
                    task.status.status === "running" 
                      ? "bg-emerald-500/10 text-emerald-400 border-emerald-500/20" 
                      : "bg-neutral-700/30 text-neutral-400 border-neutral-700"
                  )}>
                    <div className={cn("w-1.5 h-1.5 rounded-full", task.status.status === "running" ? "bg-emerald-400 animate-pulse" : "bg-neutral-500")} />
                    {task.status.status.toUpperCase()}
                  </div>
                </div>

                <div className="grid grid-cols-3 gap-2 mb-5">
                    <div className="bg-neutral-950/30 rounded-lg p-2 border border-neutral-800">
                        <span className="text-xs text-neutral-500 block mb-0.5">PID</span>
                        <span className="font-mono text-sm text-neutral-300">{task.status.pid || "—"}</span>
                    </div>
                    <div className="bg-neutral-950/30 rounded-lg p-2 border border-neutral-800">
                        <span className="text-xs text-neutral-500 block mb-0.5">Uptime</span>
                        <span className="font-mono text-sm text-neutral-300">{formatDuration(task.status.start_time)}</span>
                    </div>
                    <div className="bg-neutral-950/30 rounded-lg p-2 border border-neutral-800">
                        <span className="text-xs text-neutral-500 block mb-0.5">Status</span>
                        <span className="font-mono text-[10px] uppercase text-neutral-400">{task.status.status}</span>
                    </div>
                </div>

                <div className="flex items-center gap-2">
                  {task.status.status === "running" ? (
                    <button onClick={() => handleStop(task.config.id)} className="flex-1 flex items-center justify-center gap-2 bg-red-500/10 text-red-400 hover:bg-red-500/20 py-2 rounded-lg text-sm font-medium transition-colors border border-red-500/10">
                      <Square className="w-4 h-4" /> Stop
                    </button>
                  ) : (
                    <button onClick={() => handleStart(task.config.id)} className="flex-1 flex items-center justify-center gap-2 bg-emerald-500/10 text-emerald-400 hover:bg-emerald-500/20 py-2 rounded-lg text-sm font-medium transition-colors border border-emerald-500/10">
                      <Play className="w-4 h-4" /> Start
                    </button>
                  )}
                  
                  <button onClick={() => setTerminalTask(task)} className="p-2 text-neutral-400 hover:text-blue-400 hover:bg-blue-500/10 rounded-lg transition-colors border border-transparent hover:border-blue-500/10">
                    <TerminalIcon className="w-5 h-5" />
                  </button>

                  <button onClick={() => startEdit(task)} className="p-2 text-neutral-400 hover:text-amber-400 hover:bg-amber-500/10 rounded-lg transition-colors border border-transparent hover:border-amber-500/10">
                    <Edit2 className="w-5 h-5" />
                  </button>
                  
                  <button
                    onClick={() => handleDelete(task.config.id)}
                    className="p-2 text-neutral-400 hover:text-red-400 hover:bg-red-500/10 rounded-lg transition-colors border border-transparent hover:border-red-500/10"
                    title="Delete Task"
                  >
                    <Trash2 className="w-5 h-5" />
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Terminal Modal */}
      {terminalTask && (
        <div className="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center p-8">
            <div className="w-full max-w-5xl h-[80vh] bg-neutral-900 rounded-xl shadow-2xl flex flex-col border border-neutral-800 relative">
                <button 
                    onClick={() => setTerminalTask(null)}
                    className="absolute -top-3 -right-3 w-8 h-8 bg-neutral-700 text-white rounded-full flex items-center justify-center hover:bg-neutral-600 shadow-lg border border-neutral-600 z-10 text-xl font-bold"
                >
                    &times;
                </button>
                <div className="flex-1 p-2">
                    <TerminalView taskId={terminalTask.config.id} taskName={terminalTask.config.name} />
                </div>
            </div>
        </div>
      )}

      {/* Add/Edit Task Modal */}
      {isAddOpen && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div className="w-full max-w-md bg-neutral-900 border border-neutral-800 rounded-xl shadow-2xl p-6">
            <h2 className="text-xl font-bold mb-4">{editingTask ? "Edit Task" : "New Task"}</h2>
            <div className="flex flex-col gap-4">
              <div>
                <label className="block text-sm font-medium text-neutral-400 mb-1">Name</label>
                <input 
                    value={newName}
                    onChange={e => setNewName(e.target.value)}
                    className="w-full bg-neutral-950 border border-neutral-800 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-blue-500 transition-colors text-white"
                    placeholder="e.g. API Server"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-neutral-400 mb-1">Command</label>
                <textarea 
                    value={newCmd}
                    onChange={e => setNewCmd(e.target.value)}
                    className="w-full bg-neutral-950 border border-neutral-800 rounded-lg px-3 py-2 text-sm font-mono focus:outline-none focus:border-blue-500 transition-colors h-24 text-white"
                    placeholder="npm run dev"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-neutral-400 mb-1">Tag</label>
                <input 
                    value={newTag}
                    onChange={e => setNewTag(e.target.value)}
                    className="w-full bg-neutral-950 border border-neutral-800 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-blue-500 transition-colors text-white"
                    placeholder="Default"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-neutral-400 mb-1">Environment Variables (KEY=VALUE)</label>
                <textarea 
                    value={newEnv}
                    onChange={e => setNewEnv(e.target.value)}
                    className="w-full bg-neutral-950 border border-neutral-800 rounded-lg px-3 py-2 text-sm font-mono focus:outline-none focus:border-blue-500 transition-colors h-20 text-white"
                    placeholder="NODE_ENV=production&#10;PORT=3000"
                />
              </div>
              <div className="flex items-center gap-2">
                <input 
                    type="checkbox"
                    id="autoRetry"
                    checked={autoRetry}
                    onChange={e => setAutoRetry(e.target.checked)}
                    className="w-4 h-4 rounded border-neutral-800 bg-neutral-950 text-blue-600 focus:ring-blue-500 focus:ring-offset-neutral-900"
                />
                <label htmlFor="autoRetry" className="text-sm font-medium text-neutral-400 cursor-pointer">
                    Auto Restart on Exit
                </label>
              </div>
              <div className="flex justify-end gap-3 mt-4">
                <button onClick={closeAddModal} className="px-4 py-2 text-sm text-neutral-400 hover:text-white">Cancel</button>
                <button onClick={handleCreateOrUpdate} className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg text-sm font-medium">
                  {editingTask ? "Save Changes" : "Create Task"}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
