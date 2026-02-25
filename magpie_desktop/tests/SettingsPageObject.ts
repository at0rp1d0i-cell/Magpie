import { Page, expect } from '@playwright/test';

export class SettingsPageObject {
    readonly page: Page;

    constructor(page: Page) {
        this.page = page;
    }

    async goto() {
        await this.page.goto('http://localhost:1420/settings');
    }

    async selectProvider(index: number) {
        const selector = this.page.locator('select');
        await selector.selectOption({ index });
    }

    async fillApiKey(key: string) {
        const input = this.page.locator('input[placeholder="sk-..."]');
        await input.fill(key);
    }

    async saveConfig() {
        const saveBtn = this.page.getByRole('button', { name: /应用并持久化配置|已系统同步生效/ });
        await saveBtn.click();
    }

    async expectSavedSuccessfully() {
        const saveBtn = this.page.getByRole('button', { name: /已系统同步生效/ });
        await expect(saveBtn).toBeVisible({ timeout: 5000 });
    }
}
