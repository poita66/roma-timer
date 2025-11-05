import { test, expect, BrowserContext, Browser, Page } from '@playwright/test';

test.describe('Cross-Device Synchronization E2E Tests', () => {
  let context1: BrowserContext;
  let context2: BrowserContext;
  let page1: Page;
  let page2: Page;

  test.beforeEach(async ({ browser }) => {
    // Create two separate browser contexts to simulate different devices
    context1 = await browser.newContext({
      viewport: { width: 1200, height: 800 },
      userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/91.0.4472.124'
    });

    context2 = await browser.newContext({
      viewport: { width: 375, height: 667 }, // Mobile viewport
      userAgent: 'Mozilla/5.0 (iPhone; CPU iPhone OS 14_6 like Mac OS X) AppleWebKit/605.1.15'
    });

    page1 = await context1.newPage();
    page2 = await context2.newPage();

    // Navigate both pages to the app
    await page1.goto('/');
    await page2.goto('/');
  });

  test.afterEach(async () => {
    await context1.close();
    await context2.close();
  });

  test('synchronizes timer start across devices', async () => {
    // Device 1: Start timer
    await page1.click('[data-testid="start-button"]');

    // Wait for WebSocket connection and sync
    await page1.waitForSelector('[data-testid="running-indicator"].running');
    await page2.waitForSelector('[data-testid="running-indicator"].running', { timeout: 5000 });

    // Device 2: Should show timer is running
    const runningIndicator2 = page2.locator('[data-testid="running-indicator"]');
    await expect(runningIndicator2).toHaveClass(/running/);

    // Both devices should show same timer time
    const time1 = await page1.locator('[data-testid="timer-time"]').textContent();
    const time2 = await page2.locator('[data-testid="timer-time"]').textContent();

    // Times should be very close (within 1 second)
    expect(time1).toBe(time2);

    // Both should show same session type
    const sessionType1 = await page1.locator('[data-testid="session-type"]').textContent();
    const sessionType2 = await page2.locator('[data-testid="session-type"]').textContent();
    expect(sessionType1).toBe(sessionType2);
  });

  test('synchronizes timer pause across devices', async () => {
    // Device 1: Start timer
    await page1.click('[data-testid="start-button"]');
    await page1.waitForSelector('[data-testid="running-indicator"].running');
    await page2.waitForSelector('[data-testid="running-indicator"].running', { timeout: 5000 });

    // Device 2: Pause timer
    await page2.click('[data-testid="pause-button"]');

    // Wait for sync
    await page1.waitForSelector('[data-testid="running-indicator"].paused', { timeout: 5000 });
    await page2.waitForSelector('[data-testid="running-indicator"].paused');

    // Both devices should show timer is paused
    const runningIndicator1 = page1.locator('[data-testid="running-indicator"]');
    const runningIndicator2 = page2.locator('[data-testid="running-indicator"]');

    await expect(runningIndicator1).toHaveClass(/paused/);
    await expect(runningIndicator2).toHaveClass(/paused/);

    // Timer should be at same elapsed time
    const time1 = await page1.locator('[data-testid="timer-time"]').textContent();
    const time2 = await page2.locator('[data-testid="timer-time"]').textContent();
    expect(time1).toBe(time2);
  });

  test('synchronizes timer reset across devices', async () => {
    // Device 1: Start timer and let it run
    await page1.click('[data-testid="start-button"]');
    await page1.waitForSelector('[data-testid="running-indicator"].running');
    await page2.waitForSelector('[data-testid="running-indicator"].running', { timeout: 5000 });

    // Wait a moment for timer to progress
    await page1.waitForTimeout(1500);

    // Device 2: Reset timer
    await page2.click('[data-testid="reset-button"]');

    // Wait for sync
    await page1.waitForSelector('[data-testid="timer-time"]', { text: '25:00', timeout: 5000 });
    await page2.waitForSelector('[data-testid="timer-time"]', { text: '25:00' });

    // Both should show reset state
    await expect(page1.locator('[data-testid="timer-time"]')).toHaveText('25:00');
    await expect(page2.locator('[data-testid="timer-time"]')).toHaveText('25:00');

    // Both should show Work session
    await expect(page1.locator('[data-testid="session-type"]')).toHaveText('Work Session');
    await expect(page2.locator('[data-testid="session-type"]')).toHaveText('Work Session');

    // Timer should be paused
    await expect(page1.locator('[data-testid="running-indicator"]')).toHaveClass(/paused/);
    await expect(page2.locator('[data-testid="running-indicator"]')).toHaveClass(/paused/);
  });

  test('synchronizes session skip across devices', async () => {
    // Device 1: Start timer
    await page1.click('[data-testid="start-button"]');
    await page1.waitForSelector('[data-testid="running-indicator"].running');
    await page2.waitForSelector('[data-testid="running-indicator"].running', { timeout: 5000 });

    // Device 2: Skip to next session
    await page2.click('[data-testid="skip-button"]');

    // Wait for sync
    await page1.waitForSelector('[data-testid="session-type"]', { text: 'Short Break', timeout: 5000 });
    await page2.waitForSelector('[data-testid="session-type"]', { text: 'Short Break' });

    // Both should show Short Break session
    await expect(page1.locator('[data-testid="session-type"]')).toHaveText('Short Break');
    await expect(page2.locator('[data-testid="session-type"]')).toHaveText('Short Break');

    // Both should show break duration
    await expect(page1.locator('[data-testid="timer-time"]')).toHaveText('05:00');
    await expect(page2.locator('[data-testid="timer-time"]')).toHaveText('05:00');

    // Timer should be paused after skip
    await expect(page1.locator('[data-testid="running-indicator"]')).toHaveClass(/paused/);
    await expect(page2.locator('[data-testid="running-indicator"])).toHaveClass(/paused/);
  });

  test('handles multiple concurrent operations', async () => {
    // Device 1: Start timer
    await page1.click('[data-testid="start-button"]');
    await page1.waitForSelector('[data-testid="running-indicator"].running');
    await page2.waitForSelector('[data-testid="running-indicator"].running', { timeout: 5000 });

    // Device 2: Pause timer quickly
    await page2.click('[data-testid="pause-button"]');

    // Device 1: Try to reset timer (should be handled gracefully)
    await page1.click('[data-testid="reset-button"]');

    // Both should end up in consistent state
    await page1.waitForSelector('[data-testid="timer-time"]', { text: '25:00', timeout: 5000 });
    await page2.waitForSelector('[data-testid="timer-time"]', { text: '25:00', timeout: 5000 });

    await expect(page1.locator('[data-testid="timer-time"]')).toHaveText('25:00');
    await expect(page2.locator('[data-testid="timer-time"])).toHaveText('25:00');
  });

  test('shows connection status indicators', async () => {
    // Both devices should initially show connection status
    await expect(page1.locator('[data-testid="connection-status"]')).toBeVisible();
    await expect(page2.locator('[data-testid="connection-status"]')).toBeVisible();

    // Should show connected status
    await expect(page1.locator('[data-testid="connection-indicator"].connected')).toBeVisible();
    await expect(page2.locator('[data-testid="connection-indicator"].connected')).toBeVisible();
  });

  test('handles reconnection scenarios', async () => {
    // Device 1: Start timer
    await page1.click('[data-testid="start-button"]');
    await page1.waitForSelector('[data-testid="running-indicator"].running');

    // Simulate network disconnection on Device 2
    await context2.setOffline(true);

    // Wait for disconnection indication
    await expect(page2.locator('[data-testid="connection-indicator"].disconnected')).toBeVisible({ timeout: 5000 });

    // Device 1: Pause timer
    await page1.click('[data-testid="pause-button"]');

    // Restore connection on Device 2
    await context2.setOffline(false);

    // Device 2 should reconnect and sync to current state
    await expect(page2.locator('[data-testid="connection-indicator"].connected')).toBeVisible({ timeout: 10000 });
    await expect(page2.locator('[data-testid="running-indicator"].paused')).toBeVisible({ timeout: 5000 });

    // Both should show same final state
    const time1 = await page1.locator('[data-testid="timer-time"]').textContent();
    const time2 = await page2.locator('[data-testid="timer-time"]').textContent();
    expect(time1).toBe(time2);
  });

  test('maintains sync during rapid timer progression', async () => {
    // Device 1: Start timer
    await page1.click('[data-testid="start-button"]');
    await page1.waitForSelector('[data-testid="running-indicator"].running');
    await page2.waitForSelector('[data-testid="running-indicator"].running', { timeout: 5000 });

    // Monitor timer progression on both devices
    let lastTime1 = '';
    let lastTime2 = '';

    for (let i = 0; i < 5; i++) {
      await page1.waitForTimeout(1000);

      const currentTime1 = await page1.locator('[data-testid="timer-time"]').textContent();
      const currentTime2 = await page2.locator('[data-testid="timer-time"]').textContent();

      // Times should remain synchronized
      expect(currentTime1).toBe(currentTime2);

      // Time should be decreasing
      if (lastTime1) {
        expect(currentTime1).not.toBe(lastTime1);
      }

      lastTime1 = currentTime1;
      lastTime2 = currentTime2;
    }
  });

  test('synchronizes across device types (desktop vs mobile)', async () => {
    // Desktop (page1) starts timer
    await page1.click('[data-testid="start-button"]');
    await page1.waitForSelector('[data-testid="running-indicator"].running');
    await page2.waitForSelector('[data-testid="running-indicator"].running', { timeout: 5000 });

    // Mobile (page2) should show same state
    await expect(page2.locator('[data-testid="running-indicator"]')).toHaveClass(/running/);

    // Mobile controls timer
    await page2.click('[data-testid="pause-button"]');

    // Desktop should reflect pause
    await expect(page1.locator('[data-testid="running-indicator"]')).toHaveClass(/paused/);

    // Both should have same time and session
    const time1 = await page1.locator('[data-testid="timer-time"]').textContent();
    const time2 = await page2.locator('[data-testid="timer-time"]').textContent();
    expect(time1).toBe(time2);

    // Mobile should have responsive layout
    const mobileContainer = page2.locator('[data-testid="timer-controls"]');
    await expect(mobileContainer).toBeVisible();

    // Desktop should have full layout
    const desktopContainer = page1.locator('[data-testid="timer-controls"]');
    await expect(desktopContainer).toBeVisible();
  });

  test('handles WebSocket message loss and recovery', async () => {
    // Device 1: Start timer
    await page1.click('[data-testid="start-button"]');
    await page1.waitForSelector('[data-testid="running-indicator"].running');

    // Intercept and block WebSocket messages on Device 2 (simulate packet loss)
    await page2.route('**/*', route => {
      if (route.request().url().includes('ws')) {
        // Simulate some message loss by not immediately responding
        setTimeout(() => route.continue(), 500);
      } else {
        route.continue();
      }
    });

    // Device 1: Pause timer
    await page1.click('[data-testid="pause-button"]');

    // Device 2 should eventually sync despite message delays
    await expect(page2.locator('[data-testid="running-indicator"].paused')).toBeVisible({ timeout: 10000 });

    // Clean up route
    await page2.unroute('**/*');
  });

  test('performance: sub-500ms synchronization timing', async () => {
    // Device 1: Start timer
    const startTime = Date.now();

    await page1.click('[data-testid="start-button"]');

    // Measure time until Device 2 receives the update
    await page2.waitForSelector('[data-testid="running-indicator"].running', { timeout: 5000 });

    const syncTime = Date.now() - startTime;

    // Should sync within 500ms as per requirements
    expect(syncTime).toBeLessThan(500, `Sync took ${syncTime}ms, should be under 500ms`);
  });
});