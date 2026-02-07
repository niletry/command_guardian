import { useEffect, useRef } from 'react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import '@xterm/xterm/css/xterm.css';

interface TerminalProps {
  taskId: string;
  taskName: string;
}

interface TaskOutputPayload {
  id: string;
  data: string;
}

export function TerminalView({ taskId, taskName }: TerminalProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const terminalRef = useRef<Terminal | null>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    const term = new Terminal({
      cursorBlink: true,
      fontSize: 14,
      fontFamily: 'Menlo, Monaco, "Courier New", monospace',
      theme: {
        background: '#1e1e1e',
      },
      convertEol: true, // Important for PTY output
    });

    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);

    term.open(containerRef.current);
    fitAddon.fit();
    terminalRef.current = term;

    // Fetch History
    invoke<string>('get_log_history', { id: taskId }).then((history) => {
      if (history) term.write(history);
    });

    // Handle Input
    term.onData((data: string) => {
      invoke('write_to_pty', { id: taskId, data });
    });

    // Handle Resize
    const handleResize = () => {
      fitAddon.fit();
      invoke('resize_pty', { id: taskId, rows: term.rows, cols: term.cols });
    };
    window.addEventListener('resize', handleResize);
    handleResize();

    // Listen for Output
    const unlistenPromise = listen<TaskOutputPayload>('task-output', (event) => {
      const payload = event.payload;
      if (payload.id === taskId) {
        term.write(payload.data);
      }
    });

    return () => {
      term.dispose();
      window.removeEventListener('resize', handleResize);
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, [taskId]);

  return (
    <div className="flex flex-col h-full bg-[#1e1e1e] rounded-lg overflow-hidden border border-gray-700">
      <div className="flex items-center justify-between px-4 py-2 bg-[#2d2d2d] border-b border-gray-700">
        <span className="text-sm font-mono text-gray-300">Terminal: {taskName}</span>
        <button 
          onClick={() => {
            terminalRef.current?.clear();
            invoke('clear_log_history', { id: taskId });
          }}
          className="text-[10px] uppercase tracking-wider bg-neutral-700 hover:bg-neutral-600 text-neutral-300 px-2 py-1 rounded transition-colors border border-neutral-600"
        >
          Clear Log
        </button>
      </div>
      <div className="flex-1 p-1 overflow-hidden" ref={containerRef} />
    </div>
  );
}
