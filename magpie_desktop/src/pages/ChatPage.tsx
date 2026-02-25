import { useState, useRef, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { invoke } from "../utils/tauri";
import { SendHorizontal, Rocket, MessageSquare, MapPin, Calendar, Wallet, User } from "lucide-react";
import { useNavigate } from "react-router-dom";

interface Message {
  id: string;
  role: "user" | "assistant";
  content: string;
  timestamp: Date;
}

interface PendingPlan {
  persona?: string;
  time_window_start?: string;
  time_window_end?: string;
  departure?: { city?: string };
  destinations?: { city?: string }[];
  budget_cap?: number;
}

const WELCOME_MSG: Message = {
  id: "welcome",
  role: "assistant",
  content:
    `你好！我是 Magpie 🐦 你的智能出行管家。\n\n告诉我你的出行计划吧！比如：\n• "下周末想去北京，预算一千"\n• "三月初从杭州出发，两个人"\n\n我会帮你找到最完美的出行时机 ✨`,
  timestamp: new Date(),
};

export default function ChatPage() {
  const [messages, setMessages] = useState<Message[]>([WELCOME_MSG]);
  const [input, setInput] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [pendingPlan, setPendingPlan] = useState<PendingPlan | null>(null);
  const scrollRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const navigate = useNavigate();

  useEffect(() => {
    // Load chat history from backend on component mount
    invoke<any[]>("get_chat_history")
      .then((hist) => {
        if (hist && hist.length > 1) {
          const loadedMsg = hist
            .filter((m) => m.role !== "system")
            .map((m, idx) => ({
              id: `history-${idx}`,
              role: m.role as "user" | "assistant",
              content: m.content,
              timestamp: new Date(),
            }));
          setMessages(loadedMsg);
        }
      })
      .catch((e) => console.error("Failed to load history:", e));
  }, []);

  useEffect(() => {
    scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight, behavior: "smooth" });
  }, [messages]);

  const handleSend = async () => {
    const text = input.trim();
    if (!text || isLoading) return;

    const userMsg: Message = {
      id: Date.now().toString(),
      role: "user",
      content: text,
      timestamp: new Date(),
    };
    setMessages((prev) => [...prev, userMsg]);
    setInput("");
    setIsLoading(true);

    try {
      const reply = await invoke<string>("chat_send_message", { msg: text });

      const aiMsg: Message = {
        id: (Date.now() + 1).toString(),
        role: "assistant",
        content: reply,
        timestamp: new Date(),
      };
      setMessages((prev) => [...prev, aiMsg]);
      
      // Detect JSON config — show confirmation card instead of auto-jumping
      if (reply.includes("```json")) {
        try {
          const jsonMatch = reply.match(/```json\s*([\s\S]*?)```/);
          if (jsonMatch?.[1]) {
            const parsed = JSON.parse(jsonMatch[1].trim());
            setPendingPlan(parsed);
          }
        } catch (parseErr) {
          console.error("Failed to parse plan JSON:", parseErr);
        }
      }
    } catch (e: any) {
      console.error("Chat error:", e);
      const errorMsg: Message = {
        id: (Date.now() + 1).toString(),
        role: "assistant",
        content: `⚠️ API 连接失败：${e?.toString() || "请检查 Settings 中的密钥配置"}`,
        timestamp: new Date(),
      };
      setMessages((prev) => [...prev, errorMsg]);
    } finally {
      setIsLoading(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div className="flex h-full flex-col bg-white">
      {/* Header */}
      <header className="flex items-center justify-between border-b border-zinc-100 px-8 py-5">
        <div>
          <h1 className="text-lg font-bold tracking-tight text-zinc-900">AI 出行顾问</h1>
          <p className="text-xs text-zinc-500">和 Magpie 聊聊你的出行计划</p>
        </div>
        <div className="flex items-center gap-3">
          <button
            onClick={async () => {
              try {
                await invoke("clear_chat_history");
                setMessages([WELCOME_MSG]);
              } catch (e) {
                console.error("Failed to clear:", e);
              }
            }}
            className="text-[10px] font-semibold uppercase tracking-widest text-zinc-400 hover:text-zinc-600 transition-colors"
          >
            清空对话
          </button>
          <div className="flex items-center gap-2 rounded-full border border-zinc-200 bg-slate-50 px-3 py-1.5 shadow-sm">
            <span className="text-[10px] font-bold uppercase tracking-widest text-zinc-500">DeepSeek Core</span>
            <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-emerald-500 shadow-[0_0_6px_rgba(16,185,129,0.4)]" />
          </div>
        </div>
      </header>

      {/* Messages */}
      <div ref={scrollRef} className="flex-1 overflow-y-auto px-8 py-6">
        <div className="mx-auto max-w-4xl space-y-6">
          <AnimatePresence>
            {messages.map((msg) => (
              <motion.div
                key={msg.id}
                initial={{ opacity: 0, y: 12 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.3 }}
                className={`flex ${msg.role === "user" ? "justify-end" : "justify-start"}`}
              >
                <div
                  className={[
                    "max-w-[80%] rounded-[20px] px-5 py-3.5 text-[14px] leading-relaxed shadow-sm",
                    msg.role === "user"
                      ? "bg-zinc-900 text-white rounded-tr-sm"
                      : "border border-zinc-100 bg-slate-50 text-zinc-800 rounded-tl-sm",
                  ].join(" ")}
                >
                  <p className="whitespace-pre-wrap leading-7">{msg.content}</p>
                  <p className={`mt-2 text-[10px] font-medium tracking-wide ${msg.role === "user" ? "text-zinc-400" : "text-zinc-400"}`}>
                    {msg.timestamp.toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" })}
                  </p>
                </div>
              </motion.div>
            ))}
          </AnimatePresence>

          {isLoading && (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              className="flex justify-start"
            >
              <div className="flex items-center gap-1.5 rounded-[20px] rounded-tl-sm border border-zinc-100 bg-slate-50 px-5 py-4 shadow-sm">
                <span className="h-1.5 w-1.5 animate-bounce rounded-full bg-zinc-400 [animation-delay:0ms]" />
                <span className="h-1.5 w-1.5 animate-bounce rounded-full bg-zinc-400 [animation-delay:150ms]" />
                <span className="h-1.5 w-1.5 animate-bounce rounded-full bg-zinc-400 [animation-delay:300ms]" />
              </div>
            </motion.div>
          )}

          {/* Plan confirmation card */}
          {pendingPlan && (
            <motion.div
              initial={{ opacity: 0, y: 16, scale: 0.97 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              transition={{ duration: 0.4, ease: "easeOut" }}
              className="mx-auto w-full max-w-lg"
            >
              <div className="overflow-hidden rounded-2xl border border-violet-200 bg-gradient-to-br from-violet-50 to-white shadow-lg shadow-violet-100/40">
                <div className="border-b border-violet-100 bg-violet-50/80 px-6 py-4">
                  <h3 className="text-sm font-bold text-violet-900 flex items-center gap-2">
                    <Rocket className="h-4 w-4" />
                    出行方案确认
                  </h3>
                  <p className="mt-1 text-[11px] text-violet-600">请审阅以下监控参数，确认无误后启动雷达</p>
                </div>
                <div className="space-y-3 px-6 py-5">
                  <div className="flex items-center gap-3">
                    <MapPin className="h-4 w-4 text-zinc-400 flex-shrink-0" />
                    <span className="text-sm text-zinc-600">路线</span>
                    <span className="ml-auto text-sm font-bold text-zinc-900">
                      {pendingPlan.departure?.city ?? "?"} → {pendingPlan.destinations?.[0]?.city ?? "?"}
                    </span>
                  </div>
                  <div className="flex items-center gap-3">
                    <Calendar className="h-4 w-4 text-zinc-400 flex-shrink-0" />
                    <span className="text-sm text-zinc-600">日期</span>
                    <span className="ml-auto text-sm font-bold text-zinc-900">
                      {pendingPlan.time_window_start ?? "?"} ~ {pendingPlan.time_window_end ?? "?"}
                    </span>
                  </div>
                  <div className="flex items-center gap-3">
                    <Wallet className="h-4 w-4 text-zinc-400 flex-shrink-0" />
                    <span className="text-sm text-zinc-600">预算上限</span>
                    <span className="ml-auto text-sm font-bold text-zinc-900">
                      ￥{pendingPlan.budget_cap ?? "未设定"}
                    </span>
                  </div>
                  <div className="flex items-center gap-3">
                    <User className="h-4 w-4 text-zinc-400 flex-shrink-0" />
                    <span className="text-sm text-zinc-600">出行画像</span>
                    <span className="ml-auto text-sm font-bold text-zinc-900">
                      {pendingPlan.persona === "leisure" ? "🏖 休闲度假" : pendingPlan.persona === "business" ? "💼 商务差旅" : pendingPlan.persona ?? "?"}
                    </span>
                  </div>
                </div>
                <div className="flex gap-3 border-t border-violet-100 bg-violet-50/40 px-6 py-4">
                  <button
                    onClick={() => setPendingPlan(null)}
                    className="flex-1 rounded-xl border border-zinc-200 bg-white px-4 py-2.5 text-xs font-semibold text-zinc-600 shadow-sm transition-all hover:bg-zinc-50 active:scale-95"
                  >
                    <MessageSquare className="mr-1.5 inline h-3.5 w-3.5" />
                    继续对话修改
                  </button>
                  <button
                    onClick={async () => {
                      try {
                        await invoke("trigger_fetch_cycle");
                        setPendingPlan(null);
                        navigate("/dashboard");
                      } catch (err) {
                        console.error(err);
                      }
                    }}
                    className="flex-1 rounded-xl bg-zinc-900 px-4 py-2.5 text-xs font-semibold text-white shadow-md shadow-zinc-900/20 transition-all hover:bg-zinc-800 active:scale-95"
                  >
                    <Rocket className="mr-1.5 inline h-3.5 w-3.5" />
                    确认并启动雷达
                  </button>
                </div>
              </div>
            </motion.div>
          )}
        </div>
      </div>

      {/* Input area */}
      <div className="border-t border-zinc-100 bg-white px-8 py-5">
        <div className="mx-auto flex max-w-4xl items-end gap-3 rounded-2xl border border-zinc-200 bg-slate-50 px-4 py-3 shadow-sm transition-colors focus-within:border-zinc-400 focus-within:ring-1 focus-within:ring-zinc-400">
          <textarea
            ref={inputRef}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="告诉 Magpie 你的出行计划..."
            rows={1}
            className="flex-1 resize-none bg-transparent py-1.5 text-[14px] text-zinc-900 placeholder-zinc-400 outline-none"
          />
          <button
            onClick={handleSend}
            disabled={!input.trim() || isLoading}
            className="flex h-[36px] w-[36px] flex-shrink-0 items-center justify-center rounded-xl bg-zinc-900 text-white shadow-md transition-all hover:bg-zinc-800 disabled:opacity-40 disabled:shadow-none"
          >
            <SendHorizontal className="h-[18px] w-[18px]" strokeWidth={2.5} />
          </button>
        </div>
        <p className="mt-2.5 text-center text-[10px] font-medium text-zinc-400">
          Enter 发送 · Shift + Enter 换行
        </p>
      </div>
    </div>
  );
}
