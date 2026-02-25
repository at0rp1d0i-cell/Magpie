import { test, expect } from '@playwright/test';
import { SettingsPageObject } from './SettingsPageObject';

test.describe('Settings Page E2E', () => {
    test.beforeEach(async ({ page }) => {
        // Inject mock for Tauri IPC so the React app doesn't crash in a pure browser
        await page.addInitScript(() => {
            Object.defineProperty(window, '__TAURI_INTERNALS__', {
                value: {
                    invoke: async (cmd: string, args: any) => {
                        if (cmd === 'get_app_config') {
                            return {
                                deepseek_api_key: '',
                                deepseek_base_url: '',
                                deepseek_model: '',
                                variflight_api_key: '',
                                pushplus_token: '',
                                wxpusher_uid: '',
                            };
                        }
                        if (cmd === 'save_app_config') {
                            return "Success";
                        }
                        return null;
                    }
                }
            });
        });
    });

    test('should allow saving API configuration', async ({ page }) => {
        const settings = new SettingsPageObject(page);

        // Use the Vite dev server default port
        // We assume `npm run dev` is running before tests
        await settings.goto();

        // Verify header loads
        await expect(page.getByText('系统配置与聚合模型中心')).toBeVisible();

        // Select SiliconFlow
        await settings.selectProvider(1);

        // Check if Base URL is filled automatically 
        // It should fill 'https://api.siliconflow.cn/v1'
        const baseUrlInput = page.locator('input[placeholder="https://api.deepseek.com"]');
        await expect(baseUrlInput).toHaveValue('https://api.siliconflow.cn/v1');

        // Fill an API key
        await settings.fillApiKey('sk-playwright-test-key');

        // Save configuration
        await settings.saveConfig();

        // Expect the button state to change to "已系统同步生效"
        await settings.expectSavedSuccessfully();
    });
});
