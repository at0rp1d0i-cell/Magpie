import { useState } from "react";

interface SettingField {
  key: string;
  label: string;
  placeholder: string;
  type: "text" | "password" | "number";
}

const API_FIELDS: SettingField[] = [
  { key: "deepseek_key", label: "DeepSeek API Key", placeholder: "sk-...", type: "password" },
  { key: "deepseek_url", label: "DeepSeek Base URL", placeholder: "https://api.deepseek.com/v1", type: "text" },
  { key: "variflight_key", label: "飞常准 API Key", placeholder: "Your Variflight Key", type: "password" },
  { key: "pushplus_token", label: "PushPlus Token", placeholder: "微信推送令牌", type: "password" },
];

export default function SettingsPage() {
  const [saved, setSaved] = useState(false);

  const handleSave = () => {
    // TODO: invoke("save_settings", { ... })
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  return (
    <div className="h-full overflow-y-auto">
      <header className="border-b border-white/[0.06] px-8 py-5">
        <h1 className="text-lg font-bold tracking-tight text-white/90">系统设置</h1>
        <p className="text-xs text-white/40">API 密钥、推送偏好与主题</p>
      </header>

      <div className="max-w-xl space-y-8 p-8">
        {/* API Keys */}
        <section>
          <h2 className="mb-4 text-[11px] font-semibold uppercase tracking-widest text-white/35">API 密钥配置</h2>
          <div className="space-y-3">
            {API_FIELDS.map((field) => (
              <div key={field.key}>
                <label className="mb-1.5 block text-xs font-medium text-white/50">{field.label}</label>
                <input
                  type={field.type}
                  placeholder={field.placeholder}
                  className="w-full rounded-xl border border-white/[0.08] bg-white/[0.03] px-4 py-2.5 text-sm text-white/80 placeholder-white/20 outline-none transition-colors focus:border-violet-500/50 focus:bg-white/[0.06]"
                />
              </div>
            ))}
          </div>
        </section>

        {/* Push preferences */}
        <section>
          <h2 className="mb-4 text-[11px] font-semibold uppercase tracking-widest text-white/35">推送偏好</h2>
          <div className="space-y-3">
            {[
              { label: "微信推送 (PushPlus)", desc: "通过微信公众号接收 AI 决策报文" },
              { label: "App 内通知", desc: "在通知中心页面记录所有决策" },
            ].map((item) => (
              <div key={item.label} className="flex items-center justify-between rounded-xl border border-white/[0.06] bg-white/[0.03] px-5 py-3.5">
                <div>
                  <p className="text-sm font-medium text-white/80">{item.label}</p>
                  <p className="text-xs text-white/30">{item.desc}</p>
                </div>
                <div className="relative h-6 w-11 cursor-pointer rounded-full bg-violet-600 transition-colors">
                  <span className="absolute right-0.5 top-0.5 h-5 w-5 rounded-full bg-white shadow-sm transition-transform" />
                </div>
              </div>
            ))}
          </div>
        </section>

        {/* Save button */}
        <button
          onClick={handleSave}
          className="w-full rounded-xl bg-gradient-to-r from-violet-600 to-fuchsia-600 py-3 text-sm font-semibold text-white shadow-lg shadow-violet-500/20 transition-all hover:shadow-violet-500/40"
        >
          {saved ? "✅ 已保存" : "保存设置"}
        </button>
      </div>
    </div>
  );
}
