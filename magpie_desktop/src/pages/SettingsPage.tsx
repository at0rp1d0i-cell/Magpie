import { useState, useEffect } from "react";
import { invoke } from "../utils/tauri";
import { Save, ChevronDown, CheckCircle2, Eye, EyeOff } from "lucide-react";

interface AppConfig {
  deepseek_api_key: string;
  deepseek_base_url: string;
  deepseek_model: string;
  variflight_api_key: string;
  pushplus_token: string;
  wxpusher_uid: string;
}

const PROVIDERS = [
  { name: "DeepSeek 官方", url: "https://api.deepseek.com", model: "deepseek-chat" },
  { name: "硅基流动 (SiliconFlow)", url: "https://api.siliconflow.cn/v1", model: "Pro/deepseek-ai/DeepSeek-V3" },
  { name: "阿里云百炼", url: "https://dashscope.aliyuncs.com/compatible-mode/v1", model: "qwen-plus" },
  { name: "智谱清言", url: "https://open.bigmodel.cn/api/paas/v4", model: "glm-4-plus" },
  { name: "自定义组合", url: "", model: "" },
];

const SecretInput = ({ value, onChange, placeholder }: { value: string; onChange: (v: string) => void; placeholder: string }) => {
  const [show, setShow] = useState(false);
  return (
    <div className="relative w-full">
      <input
        type={show ? "text" : "password"}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className="w-full rounded-xl border border-zinc-200 bg-white px-4 py-2.5 pr-10 text-sm text-zinc-800 shadow-sm placeholder-zinc-400 outline-none transition-colors focus:border-zinc-400 focus:ring-1 focus:ring-zinc-400"
      />
      <button
        type="button"
        title={show ? "隐藏明文" : "查看明文"}
        onClick={() => setShow(!show)}
        className="absolute right-3 top-2.5 flex h-5 w-5 items-center justify-center text-zinc-400 outline-none transition-colors hover:text-zinc-600"
      >
        {show ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
      </button>
    </div>
  );
};

export default function SettingsPage() {
  const [config, setConfig] = useState<AppConfig>({
    deepseek_api_key: "",
    deepseek_base_url: "",
    deepseek_model: "",
    variflight_api_key: "",
    pushplus_token: "",
    wxpusher_uid: "",
  });
  const [saved, setSaved] = useState(false);
  const [loading, setLoading] = useState(true);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<{status: 'idle' | 'success' | 'error', msg: string}>({status: 'idle', msg: ''});

  // Load from backend
  useEffect(() => {
    invoke<AppConfig>("get_app_config")
      .then((data) => {
        setConfig(data);
        setLoading(false);
      })
      .catch((e) => {
        console.error("加载配置失败:", e);
        setLoading(false);
      });
  }, []);

  const handleProviderSelect = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const idx = Number(e.target.value);
    if (idx < PROVIDERS.length - 1) {
      const p = PROVIDERS[idx];
      setConfig((prev) => ({
        ...prev,
        deepseek_base_url: p.url,
        deepseek_model: p.model,
      }));
    }
  };

  const handleSave = async () => {
    try {
      setTesting(true);
      setTestResult({ status: 'idle', msg: '正在嗅探原生网络连通性...' });
      
      // 1. Verify connection first
      const pingResult = await invoke<string>("test_llm_connection", { config });
      
      // 2. If valid, proceed to save
      await invoke("save_app_config", { config });
      
      setTestResult({ status: 'success', msg: pingResult });
      setSaved(true);
      setTimeout(() => {
        setSaved(false);
        setTestResult({ status: 'idle', msg: '' });
      }, 3000);
    } catch (e: any) {
      console.error("嗅探或保存失败", e);
      setTestResult({ status: 'error', msg: e.toString() });
      setSaved(false);
    } finally {
      setTesting(false);
    }
  };

  const handleChange = (k: keyof AppConfig, v: string) => {
    setConfig((prev) => ({ ...prev, [k]: v }));
  };

  if (loading) return <div className="p-8 text-zinc-500">正在与系统底层同步配置...</div>;

  return (
    <div className="h-full overflow-y-auto bg-white">
      <header className="border-b border-zinc-100 px-8 py-6">
        <h1 className="text-[20px] font-bold tracking-tight text-zinc-900">系统配置与聚合模型中心</h1>
        <p className="mt-1 text-sm text-zinc-500">所有修改将通过 Rust 原生引擎直写磁盘，彻底保存并热重载。</p>
      </header>

      <div className="max-w-3xl space-y-10 p-8">
        {/* Core AI Setting */}
        <section>
          <div className="mb-5 flex items-center justify-between">
            <h2 className="text-xs font-semibold uppercase tracking-widest text-zinc-400">大语言模型调度枢纽 (LLM Base)</h2>
          </div>
          <div className="space-y-4 rounded-2xl border border-zinc-200/60 bg-slate-50/50 p-6 shadow-sm">
            
            <div className="grid grid-cols-2 gap-4">
               <div>
                 <label className="mb-1.5 block text-xs font-medium text-zinc-600">服务提供商聚合器预设</label>
                 <div className="relative">
                   <select 
                     onChange={handleProviderSelect}
                     className="w-full appearance-none rounded-xl border border-zinc-200 bg-white px-4 py-2.5 text-sm font-medium text-zinc-800 shadow-sm outline-none transition-colors focus:border-zinc-400 focus:ring-1 focus:ring-zinc-400"
                    >
                     <option value={PROVIDERS.length - 1}>按下方参数直接使用 (默认)</option>
                     {PROVIDERS.map((p, i) => (
                       <option key={i} value={i}>{p.name}</option>
                     ))}
                   </select>
                   <ChevronDown className="pointer-events-none absolute right-3 top-3 h-4 w-4 text-zinc-400" />
                 </div>
               </div>
               <div>
                  <label className="mb-1.5 flex items-center justify-between text-xs font-medium text-zinc-600">
                    <span>API Key (令牌)</span>
                    {config.deepseek_api_key && <span className="text-[10px] text-emerald-500">✓ 已配置</span>}
                  </label>
                  <SecretInput
                    value={config.deepseek_api_key}
                    onChange={(v) => handleChange("deepseek_api_key", v)}
                    placeholder="sk-..."
                  />
               </div>
            </div>

            <div className="grid grid-cols-2 gap-4 pt-1">
                <div>
                  <label className="mb-1.5 block text-xs font-medium text-zinc-600">Base URL (基地址)</label>
                  <input
                    type="text"
                    value={config.deepseek_base_url}
                    onChange={(e) => handleChange("deepseek_base_url", e.target.value)}
                    placeholder="https://api.deepseek.com"
                    className="w-full rounded-xl border border-zinc-200 bg-white px-4 py-2.5 text-sm text-zinc-800 shadow-sm placeholder-zinc-400 outline-none transition-colors focus:border-zinc-400 focus:ring-1 focus:ring-zinc-400"
                  />
               </div>
                <div>
                  <label className="mb-1.5 block text-xs font-medium text-zinc-600">Model (具体调用模型)</label>
                  <input
                    type="text"
                    value={config.deepseek_model}
                    onChange={(e) => handleChange("deepseek_model", e.target.value)}
                    placeholder="deepseek-chat"
                    className="w-full rounded-xl border border-zinc-200 bg-white px-4 py-2.5 text-sm text-zinc-800 shadow-sm placeholder-zinc-400 outline-none transition-colors focus:border-zinc-400 focus:ring-1 focus:ring-zinc-400"
                  />
               </div>
            </div>

          </div>
        </section>

        {/* Function Interfaces */}
        <section>
          <h2 className="mb-5 text-xs font-semibold uppercase tracking-widest text-zinc-400">外部资源服务流 (Fetchers)</h2>
          <div className="space-y-4 rounded-2xl border border-zinc-200/60 bg-white p-6 shadow-sm">
             <div>
                <label className="mb-1.5 flex max-w-md items-center justify-between text-xs font-medium text-zinc-600">
                  <span>Variflight (飞常准) API Key</span>
                  {config.variflight_api_key && <span className="text-[10px] text-emerald-500">✓ 已配置</span>}
                </label>
                <div className="max-w-md">
                  <SecretInput
                    value={config.variflight_api_key}
                    onChange={(v) => handleChange("variflight_api_key", v)}
                    placeholder="用于读取准点率和底价机运"
                  />
                </div>
             </div>
          </div>
        </section>

        {/* Notification Options */}
        <section>
          <h2 className="mb-5 text-xs font-semibold uppercase tracking-widest text-zinc-400">事件分发与推送 (Notifications)</h2>
          <div className="grid grid-cols-2 gap-6">
             <div className="rounded-2xl border border-zinc-200/60 bg-white p-6 shadow-sm">
                <div className="mb-1.5 flex items-center justify-between">
                  <label className="text-sm font-semibold text-zinc-800">PushPlus Token</label>
                  {config.pushplus_token && <span className="text-[10px] font-medium text-emerald-500">✓ 已接入</span>}
                </div>
                <p className="mb-4 text-xs text-zinc-500">通过公众号即时接收购票建议</p>
                <SecretInput
                  value={config.pushplus_token}
                  onChange={(v) => handleChange("pushplus_token", v)}
                  placeholder="输入 Token 开启通道"
                />
             </div>
             <div className="rounded-2xl border border-zinc-200/60 bg-white p-6 shadow-sm">
                <div className="mb-1.5 flex items-center justify-between">
                  <label className="text-sm font-semibold text-zinc-800">WXPusher UID</label>
                  {config.wxpusher_uid && <span className="text-[10px] font-medium text-emerald-500">✓ 已接入</span>}
                </div>
                <p className="mb-4 text-xs text-zinc-500">双冗余通道备份推送路由</p>
                <SecretInput
                  value={config.wxpusher_uid}
                  onChange={(v) => handleChange("wxpusher_uid", v)}
                  placeholder="输入 UID (可选)"
                />
             </div>
          </div>
        </section>

        {/* Submit */}
        <div className="flex flex-col items-end pb-8">
           {testResult.status !== 'idle' && (
             <div className={`mb-4 max-w-md rounded-xl p-4 text-xs font-medium shadow-sm border ${
               testResult.status === 'success' ? 'border-emerald-200 bg-emerald-50 text-emerald-700' : 'border-rose-200 bg-rose-50 text-rose-700'
             }`}>
               {testResult.msg}
             </div>
           )}
           <button
             onClick={handleSave}
             disabled={testing}
             className="flex items-center gap-2 rounded-xl bg-zinc-900 px-8 py-3.5 text-sm font-medium text-white shadow-lg shadow-zinc-900/20 transition-all hover:bg-zinc-800 active:scale-95 disabled:opacity-50"
           >
             {testing ? (
               <>
                 <span className="h-4 w-4 animate-spin rounded-full border-2 border-white border-t-transparent" />
                 验证并下发配置...
               </>
             ) : saved ? (
               <>
                 <CheckCircle2 className="h-4 w-4" /> 已系统同步生效
               </>
             ) : (
               <>
                 <Save className="h-4 w-4" /> 验证并持久化网络配置
               </>
             )}
           </button>
        </div>

      </div>
    </div>
  );
}
