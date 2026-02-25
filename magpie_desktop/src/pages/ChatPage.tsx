import { useState, useRef, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";

interface Message {
  id: string;
  role: "user" | "assistant";
  content: string;
  timestamp: Date;
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
  const scrollRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);

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
    <div className="flex h-full flex-col">
      {/* Header */}
      <header className="flex items-center justify-between border-b border-white/[0.06] px-8 py-5">
        <div>
          <h1 className="text-lg font-bold tracking-tight text-white/90">AI 出行顾问</h1>
          <p className="text-xs text-white/40">和 Magpie 聊聊你的出行计划</p>
        </div>
        <div className="flex items-center gap-2 rounded-full border border-white/10 bg-white/[0.04] px-3 py-1.5">
          <span className="text-[10px] font-bold uppercase tracking-widest text-white/50">DeepSeek V3</span>
          <span className="h-1.5 w-1.5 rounded-full bg-emerald-400" />
        </div>
      </header>

      {/* Messages */}
      <div ref={scrollRef} className="flex-1 overflow-y-auto px-8 py-6">
        <div className="mx-auto max-w-2xl space-y-6">
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
                    "max-w-[80%] rounded-2xl px-5 py-3.5 text-[14px] leading-relaxed",
                    msg.role === "user"
                      ? "bg-gradient-to-br from-violet-600 to-fuchsia-600 text-white"
                      : "border border-white/10 bg-white/[0.05] text-white/85 backdrop-blur-md",
                  ].join(" ")}
                >
                  <p className="whitespace-pre-wrap">{msg.content}</p>
                  <p className={`mt-1.5 text-[10px] ${msg.role === "user" ? "text-white/50" : "text-white/25"}`}>
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
              <div className="flex items-center gap-1.5 rounded-2xl border border-white/10 bg-white/[0.05] px-5 py-3.5 backdrop-blur-md">
                <span className="h-2 w-2 animate-bounce rounded-full bg-violet-400 [animation-delay:0ms]" />
                <span className="h-2 w-2 animate-bounce rounded-full bg-violet-400 [animation-delay:150ms]" />
                <span className="h-2 w-2 animate-bounce rounded-full bg-violet-400 [animation-delay:300ms]" />
              </div>
            </motion.div>
          )}
        </div>
      </div>

      {/* Input area */}
      <div className="border-t border-white/[0.06] px-8 py-4">
        <div className="mx-auto flex max-w-2xl items-end gap-3 rounded-2xl border border-white/10 bg-white/[0.04] px-4 py-3 backdrop-blur-md transition-colors focus-within:border-violet-500/40 focus-within:bg-white/[0.06]">
          <textarea
            ref={inputRef}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="告诉 Magpie 你的出行计划..."
            rows={1}
            className="flex-1 resize-none bg-transparent text-sm text-white/90 placeholder-white/25 outline-none"
          />
          <button
            onClick={handleSend}
            disabled={!input.trim() || isLoading}
            className="flex h-8 w-8 flex-shrink-0 items-center justify-center rounded-lg bg-gradient-to-r from-violet-600 to-fuchsia-600 text-white transition-all hover:shadow-lg hover:shadow-violet-500/30 disabled:opacity-30"
          >
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" className="h-4 w-4">
              <path d="M3.105 2.288a.75.75 0 0 0-.826.95l1.414 4.926A1.5 1.5 0 0 0 5.135 9.25h6.115a.75.75 0 0 1 0 1.5H5.135a1.5 1.5 0 0 0-1.442 1.086l-1.414 4.926a.75.75 0 0 0 .826.95 28.897 28.897 0 0 0 15.293-7.155.75.75 0 0 0 0-1.114A28.897 28.897 0 0 0 3.105 2.288Z" />
            </svg>
          </button>
        </div>
        <p className="mt-2 text-center text-[10px] text-white/20">
          Enter 发送 · Shift + Enter 换行
        </p>
      </div>
    </div>
  );
}
