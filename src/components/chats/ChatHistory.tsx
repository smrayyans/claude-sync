import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MessageSquare, FolderOpen, ChevronRight, ArrowLeft, Bot, User, Wrench, Trash2, CheckSquare, Square, Calendar, Cloud, HardDrive, X } from "lucide-react";
import { formatRelativeTime, cn } from "../../lib/utils";

interface ChatSession {
  id: string;
  project_slug: string;
  project_display: string;
  timestamp: string;
  message_count: number;
  first_user_message: string;
  path: string;
  file_size_bytes: number;
  line_count: number;
  is_synced: boolean;
}

interface ChatMessage {
  role: string;
  content: string;
  timestamp: string;
  is_tool_use: boolean;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function getDateGroup(timestamp: string): string {
  if (!timestamp) return "Unknown";
  const date = new Date(timestamp);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

  if (diffDays === 0) return "Today";
  if (diffDays === 1) return "Yesterday";
  if (diffDays < 7) return "This week";
  if (diffDays < 30) return "This month";
  return "Older";
}

export default function ChatHistory() {
  const [sessions, setSessions] = useState<ChatSession[]>([]);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [selected, setSelected] = useState<ChatSession | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadingMsgs, setLoadingMsgs] = useState(false);
  const [search, setSearch] = useState("");
  const [groupBy, setGroupBy] = useState<"project" | "date">("project");
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [isSelecting, setIsSelecting] = useState(false);
  const [showDeleteDialog, setShowDeleteDialog] = useState(false);
  const [deleteFromSync, setDeleteFromSync] = useState(false);

  useEffect(() => {
    loadSessions();
  }, []);

  const loadSessions = async () => {
    setLoading(true);
    try {
      const data = await invoke<ChatSession[]>("list_chat_sessions");
      setSessions(data);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  };

  const openSession = async (session: ChatSession) => {
    if (isSelecting) {
      toggleSelect(session.path);
      return;
    }
    setSelected(session);
    setLoadingMsgs(true);
    try {
      const msgs = await invoke<ChatMessage[]>("get_chat_messages", {
        sessionPath: session.path,
      });
      setMessages(msgs);
    } catch (e) {
      setMessages([]);
    } finally {
      setLoadingMsgs(false);
    }
  };

  const toggleSelect = (path: string) => {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });
  };

  const handleDelete = async () => {
    const paths = Array.from(selectedIds);
    if (paths.length === 0) return;

    try {
      await invoke("delete_chat_sessions", { paths, deleteFromSync: deleteFromSync });
      setSessions((prev) => prev.filter((s) => !selectedIds.has(s.path)));
      setSelectedIds(new Set());
      setIsSelecting(false);
    } catch (e) {
      console.error("Delete failed:", e);
    }
    setShowDeleteDialog(false);
    setDeleteFromSync(false);
  };

  const handleDeleteSingle = async (session: ChatSession) => {
    setSelectedIds(new Set([session.path]));
    setShowDeleteDialog(true);
  };

  const filtered = sessions.filter(
    (s) =>
      search === "" ||
      s.first_user_message.toLowerCase().includes(search.toLowerCase()) ||
      s.project_display.toLowerCase().includes(search.toLowerCase())
  );

  // Group sessions
  const groups: Record<string, ChatSession[]> = {};
  for (const s of filtered) {
    const key = groupBy === "project" ? s.project_display : getDateGroup(s.timestamp);
    if (!groups[key]) groups[key] = [];
    groups[key].push(s);
  }

  // Sort date groups in order
  const groupOrder = groupBy === "date"
    ? ["Today", "Yesterday", "This week", "This month", "Older"]
    : Object.keys(groups).sort();

  const sortedGroups = groupOrder.filter((k) => groups[k]?.length > 0);

  // Delete confirmation dialog
  if (showDeleteDialog) {
    return (
      <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
        <div className="bg-surface rounded-xl border border-border p-6 max-w-sm w-full shadow-xl">
          <h3 className="text-base font-semibold text-text mb-2">
            Delete {selectedIds.size} session{selectedIds.size > 1 ? "s" : ""}?
          </h3>
          <p className="text-sm text-text-muted mb-4">
            This cannot be undone.
          </p>
          <label className="flex items-center gap-2 text-sm text-text-muted mb-5 cursor-pointer">
            <input
              type="checkbox"
              checked={deleteFromSync}
              onChange={(e) => setDeleteFromSync(e.target.checked)}
              className="rounded"
            />
            Also remove from sync repository
          </label>
          <div className="flex gap-2 justify-end">
            <button
              onClick={() => { setShowDeleteDialog(false); setDeleteFromSync(false); }}
              className="btn-ghost px-4 py-2 text-sm"
            >
              Cancel
            </button>
            <button
              onClick={handleDelete}
              className="px-4 py-2 text-sm rounded-lg bg-red-500/20 text-red-400 hover:bg-red-500/30 transition-colors"
            >
              Delete
            </button>
          </div>
        </div>
      </div>
    );
  }

  // Session detail view
  if (selected) {
    return (
      <div className="flex flex-col h-full">
        <div className="flex items-center gap-3 p-4 border-b border-border bg-surface flex-shrink-0">
          <button onClick={() => { setSelected(null); setMessages([]); }} className="btn-ghost p-1.5">
            <ArrowLeft size={16} />
          </button>
          <div className="flex-1 min-w-0">
            <div className="text-sm font-medium text-text truncate">
              {selected.first_user_message || "Chat session"}
            </div>
            <div className="text-xs text-text-muted mt-0.5 flex items-center gap-2">
              <FolderOpen size={10} />
              {selected.project_display}
              <span>-</span>
              {formatRelativeTime(selected.timestamp)}
              <span>-</span>
              {selected.message_count} messages
              <span>-</span>
              {formatBytes(selected.file_size_bytes)}
              {selected.is_synced ? (
                <Cloud size={10} className="text-green-400" />
              ) : (
                <HardDrive size={10} className="text-text-dim" />
              )}
            </div>
          </div>
          <button
            onClick={() => handleDeleteSingle(selected)}
            className="btn-ghost p-1.5 text-text-dim hover:text-red-400"
            title="Delete session"
          >
            <Trash2 size={14} />
          </button>
        </div>

        <div className="flex-1 overflow-y-auto p-4 space-y-4">
          {loadingMsgs ? (
            <div className="text-sm text-text-muted">Loading messages...</div>
          ) : messages.length === 0 ? (
            <div className="text-sm text-text-muted">No messages found</div>
          ) : (
            messages.map((msg, i) => (
              <MessageBubble key={i} msg={msg} />
            ))
          )}
        </div>
      </div>
    );
  }

  // List view
  return (
    <div className="p-6">
      <div className="mb-5">
        <h1 className="text-xl font-semibold text-text">Chat History</h1>
        <p className="text-sm text-text-muted mt-0.5">
          All your Claude Code conversations, synced across machines
        </p>
      </div>

      {/* Toolbar */}
      <div className="flex items-center gap-3 mb-4">
        <input
          type="text"
          className="input flex-1 text-sm"
          placeholder="Search chats..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />

        {/* Group by toggle */}
        <div className="flex rounded-lg border border-border overflow-hidden text-xs">
          <button
            onClick={() => setGroupBy("project")}
            className={cn(
              "px-3 py-1.5 transition-colors",
              groupBy === "project" ? "bg-accent/20 text-accent" : "text-text-muted hover:bg-surface-2"
            )}
          >
            <FolderOpen size={12} className="inline mr-1" />
            Project
          </button>
          <button
            onClick={() => setGroupBy("date")}
            className={cn(
              "px-3 py-1.5 transition-colors",
              groupBy === "date" ? "bg-accent/20 text-accent" : "text-text-muted hover:bg-surface-2"
            )}
          >
            <Calendar size={12} className="inline mr-1" />
            Date
          </button>
        </div>

        {/* Select mode toggle */}
        <button
          onClick={() => {
            setIsSelecting(!isSelecting);
            if (isSelecting) setSelectedIds(new Set());
          }}
          className={cn(
            "btn-ghost p-2 text-xs",
            isSelecting && "text-accent"
          )}
          title={isSelecting ? "Cancel selection" : "Select sessions"}
        >
          {isSelecting ? <X size={14} /> : <CheckSquare size={14} />}
        </button>
      </div>

      {/* Selection toolbar */}
      {isSelecting && selectedIds.size > 0 && (
        <div className="flex items-center gap-3 mb-4 p-3 rounded-lg bg-surface-2 border border-border">
          <span className="text-xs text-text-muted flex-1">
            {selectedIds.size} selected
          </span>
          <button
            onClick={() => {
              const allPaths = filtered.map((s) => s.path);
              setSelectedIds(new Set(allPaths));
            }}
            className="text-xs text-accent hover:underline"
          >
            Select all
          </button>
          <button
            onClick={() => setShowDeleteDialog(true)}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg bg-red-500/20 text-red-400 hover:bg-red-500/30 transition-colors"
          >
            <Trash2 size={12} />
            Delete {selectedIds.size}
          </button>
        </div>
      )}

      {loading ? (
        <div className="text-sm text-text-muted">Loading chat history...</div>
      ) : sessions.length === 0 ? (
        <div className="text-center py-16">
          <MessageSquare size={40} className="text-text-dim mx-auto mb-3" />
          <p className="text-text-muted text-sm">No chat history found</p>
          <p className="text-text-dim text-xs mt-1">
            Start chatting in Claude Code and history will appear here
          </p>
        </div>
      ) : (
        <div className="space-y-4">
          {sortedGroups.map((groupKey) => (
            <div key={groupKey}>
              <div className="flex items-center gap-1.5 mb-2">
                {groupBy === "project" ? (
                  <FolderOpen size={12} className="text-accent" />
                ) : (
                  <Calendar size={12} className="text-accent" />
                )}
                <span className="text-xs font-medium text-text-muted">{groupKey}</span>
                <span className="text-xs text-text-dim">({groups[groupKey].length})</span>
              </div>
              <div className="space-y-1.5">
                {groups[groupKey].map((session) => (
                  <div
                    key={session.id}
                    className={cn(
                      "card cursor-pointer hover:border-accent/40 transition-colors group",
                      selectedIds.has(session.path) && "border-accent/60 bg-accent/5"
                    )}
                    onClick={() => openSession(session)}
                  >
                    <div className="flex items-start gap-3">
                      {isSelecting && (
                        <div className="mt-0.5 flex-shrink-0">
                          {selectedIds.has(session.path) ? (
                            <CheckSquare size={14} className="text-accent" />
                          ) : (
                            <Square size={14} className="text-text-dim" />
                          )}
                        </div>
                      )}
                      <MessageSquare size={14} className="text-accent mt-0.5 flex-shrink-0" />
                      <div className="flex-1 min-w-0">
                        <p className="text-sm text-text line-clamp-2 leading-snug">
                          {session.first_user_message || "Empty session"}
                        </p>
                        <div className="flex items-center gap-3 mt-1.5 text-xs text-text-dim">
                          <span>{formatRelativeTime(session.timestamp)}</span>
                          <span>{session.message_count} msgs</span>
                          <span>{formatBytes(session.file_size_bytes)}</span>
                          {session.is_synced ? (
                            <span className="flex items-center gap-1 text-green-400">
                              <Cloud size={10} /> synced
                            </span>
                          ) : (
                            <span className="flex items-center gap-1">
                              <HardDrive size={10} /> local
                            </span>
                          )}
                        </div>
                      </div>
                      {!isSelecting && (
                        <div className="flex items-center gap-1 flex-shrink-0">
                          <button
                            onClick={(e) => { e.stopPropagation(); handleDeleteSingle(session); }}
                            className="p-1 opacity-0 group-hover:opacity-100 text-text-dim hover:text-red-400 transition-all"
                            title="Delete"
                          >
                            <Trash2 size={12} />
                          </button>
                          <ChevronRight size={14} className="text-text-dim opacity-0 group-hover:opacity-100 mt-0.5" />
                        </div>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function MessageBubble({ msg }: { msg: ChatMessage }) {
  const isUser = msg.role === "user";
  const isTool = msg.is_tool_use;

  if (isTool) {
    return (
      <div className="flex items-center gap-2 text-xs text-text-dim py-1">
        <Wrench size={11} />
        <span className="font-mono">{msg.content}</span>
      </div>
    );
  }

  return (
    <div className={cn("flex gap-3", isUser ? "justify-end" : "justify-start")}>
      {!isUser && (
        <div className="w-7 h-7 rounded-full bg-accent/20 flex items-center justify-center flex-shrink-0 mt-0.5">
          <Bot size={13} className="text-accent" />
        </div>
      )}
      <div
        className={cn(
          "max-w-[75%] rounded-xl px-4 py-2.5 text-sm leading-relaxed",
          isUser
            ? "bg-accent/20 text-text rounded-tr-sm"
            : "bg-surface-2 text-text rounded-tl-sm border border-border"
        )}
      >
        <MessageContent content={msg.content} />
        {msg.timestamp && (
          <div className="text-xs text-text-dim mt-1.5 opacity-60">
            {formatRelativeTime(msg.timestamp)}
          </div>
        )}
      </div>
      {isUser && (
        <div className="w-7 h-7 rounded-full bg-surface-3 flex items-center justify-center flex-shrink-0 mt-0.5">
          <User size={13} className="text-text-muted" />
        </div>
      )}
    </div>
  );
}

function MessageContent({ content }: { content: string }) {
  const lines = content.split("\n");

  return (
    <div className="space-y-1 whitespace-pre-wrap break-words font-sans">
      {lines.map((line, i) => {
        if (line.startsWith("## ")) {
          return <div key={i} className="font-semibold text-text mt-2">{line.slice(3)}</div>;
        }
        if (line.startsWith("# ")) {
          return <div key={i} className="font-bold text-text mt-2">{line.slice(2)}</div>;
        }
        if (line.startsWith("- ") || line.startsWith("* ")) {
          return <div key={i} className="flex gap-2"><span className="text-accent mt-0.5">-</span><span>{line.slice(2)}</span></div>;
        }
        if (line.startsWith("```")) {
          return <div key={i} className="font-mono text-xs bg-background rounded px-2 py-1 text-text-muted">{line}</div>;
        }
        return <span key={i}>{line || "\u00A0"}</span>;
      })}
    </div>
  );
}
