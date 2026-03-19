import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MessageSquare, FolderOpen, ChevronRight, ArrowLeft, Bot, User, Wrench } from "lucide-react";
import { formatRelativeTime, cn } from "../../lib/utils";

interface ChatSession {
  id: string;
  project_slug: string;
  project_display: string;
  timestamp: string;
  message_count: number;
  first_user_message: string;
  path: string;
}

interface ChatMessage {
  role: string;
  content: string;
  timestamp: string;
  is_tool_use: boolean;
}

export default function ChatHistory() {
  const [sessions, setSessions] = useState<ChatSession[]>([]);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [selected, setSelected] = useState<ChatSession | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadingMsgs, setLoadingMsgs] = useState(false);
  const [search, setSearch] = useState("");

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

  const filtered = sessions.filter(
    (s) =>
      search === "" ||
      s.first_user_message.toLowerCase().includes(search.toLowerCase()) ||
      s.project_display.toLowerCase().includes(search.toLowerCase())
  );

  // Group by project
  const groups: Record<string, ChatSession[]> = {};
  for (const s of filtered) {
    if (!groups[s.project_display]) groups[s.project_display] = [];
    groups[s.project_display].push(s);
  }

  if (selected) {
    return (
      <div className="flex flex-col h-full">
        {/* Header */}
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
              <span>·</span>
              {formatRelativeTime(selected.timestamp)}
              <span>·</span>
              {selected.message_count} messages
            </div>
          </div>
        </div>

        {/* Messages */}
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

  return (
    <div className="p-6">
      <div className="mb-5">
        <h1 className="text-xl font-semibold text-text">Chat History</h1>
        <p className="text-sm text-text-muted mt-0.5">
          All your Claude Code conversations, synced across machines
        </p>
      </div>

      <input
        type="text"
        className="input mb-5 text-sm"
        placeholder="Search chats..."
        value={search}
        onChange={(e) => setSearch(e.target.value)}
      />

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
          {Object.entries(groups).map(([project, projectSessions]) => (
            <div key={project}>
              <div className="flex items-center gap-1.5 mb-2">
                <FolderOpen size={12} className="text-accent" />
                <span className="text-xs font-medium text-text-muted">{project}</span>
                <span className="text-xs text-text-dim">({projectSessions.length})</span>
              </div>
              <div className="space-y-1.5">
                {projectSessions.map((session) => (
                  <div
                    key={session.id}
                    className="card cursor-pointer hover:border-accent/40 transition-colors group"
                    onClick={() => openSession(session)}
                  >
                    <div className="flex items-start gap-3">
                      <MessageSquare size={14} className="text-accent mt-0.5 flex-shrink-0" />
                      <div className="flex-1 min-w-0">
                        <p className="text-sm text-text line-clamp-2 leading-snug">
                          {session.first_user_message || "Empty session"}
                        </p>
                        <div className="flex items-center gap-3 mt-1.5 text-xs text-text-dim">
                          <span>{formatRelativeTime(session.timestamp)}</span>
                          <span>{session.message_count} messages</span>
                        </div>
                      </div>
                      <ChevronRight size={14} className="text-text-dim opacity-0 group-hover:opacity-100 mt-0.5 flex-shrink-0" />
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
  // Simple markdown-ish rendering
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
          return <div key={i} className="flex gap-2"><span className="text-accent mt-0.5">•</span><span>{line.slice(2)}</span></div>;
        }
        if (line.startsWith("```")) {
          return <div key={i} className="font-mono text-xs bg-background rounded px-2 py-1 text-text-muted">{line}</div>;
        }
        return <span key={i}>{line || "\u00A0"}</span>;
      })}
    </div>
  );
}
