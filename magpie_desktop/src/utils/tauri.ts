import { invoke as tauriInvoke } from "@tauri-apps/api/core";

export async function invoke<T>(cmd: string, args?: any): Promise<T> {
    if (!('__TAURI_INTERNALS__' in window)) {
        console.warn(`[Mock Tauri] invoke('${cmd}') in browser environment.`);

        // Fallbacks for web browser preview
        if (cmd === "chat_send_message") {
            await new Promise(r => setTimeout(r, 1000));
            return "这是网页端预览模式，我能完美渲染这些对话气泡！\n\n(抱歉，由于缺少 Tauri 本地内核，我暂时无法直连外部大模型网络。编译为桌面版后即刻生效 ✨)" as any;
        }
        if (cmd === "get_app_config") {
            return {
                deepseek_api_key: "",
                deepseek_base_url: "https://api.deepseek.com",
                deepseek_model: "deepseek-chat",
                variflight_api_key: "",
                pushplus_token: "",
                wxpusher_uid: ""
            } as any;
        }
        if (cmd === "get_latest_tickets") {
            return [] as any;
        }
        if (cmd === "get_daemon_status") {
            return "web-preview" as any;
        }
        if (cmd === "save_app_config") {
            await new Promise(r => setTimeout(r, 600));
            return "Mock Saved" as any;
        }

        return null as any;
    }

    return tauriInvoke<T>(cmd, args);
}
