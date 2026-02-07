export interface TaskConfig {
  id: string;
  name: string;
  command: string;
  tag: string;
  auto_retry: boolean;
}

export interface TaskStatus {
  id: string;
  status: "running" | "stopped" | "error";
  pid: number | null;
  start_time: number | null;
}

export interface TaskView {
  config: TaskConfig;
  status: TaskStatus;
}
