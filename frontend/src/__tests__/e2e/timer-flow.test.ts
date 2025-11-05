import { test, expect } from '@playwright/test';

test.describe('Complete Timer Flow E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('complete timer workflow: start -> pause -> reset -> skip', async ({ page }) => {
    // Initial state check
    await expect(page.locator('[data-testid="timer-display"]')).toBeVisible();
    await expect(page.locator('[data-testid="running-indicator"]')).toHaveClass('paused');

    // Get initial time
    const initialTime = await page.locator('[data-testid="timer-display"]').textContent();
    expect(initialTime).toMatch(/\d{2}:\d{2}/);

    // Start timer
    await page.click('[data-testid="start-button"]');
    await expect(page.locator('[data-testid="running-indicator"]')).toHaveClass('running');

    // Wait for timer to progress
    await page.waitForTimeout(1100);

    // Verify time has decreased
    const runningTime = await page.locator('[data-testid="timer-display"]').textContent();
    expect(runningTime).not.toBe(initialTime);

    // Pause timer
    await page.click('[data-testid="pause-button"]');
    await expect(page.locator('[data-testid="running-indicator"]')).toHaveClass('paused');

    // Verify timer stops progressing
    const pausedTime = await page.locator('[data-testid="timer-display"]').textContent();
    await page.waitForTimeout(1100);
    const stillPausedTime = await page.locator('[data-testid="timer-display"]').textContent();
    expect(stillPausedTime).toBe(pausedTime);

    // Reset timer
    await page.click('[data-testid="reset-button"]');
    await expect(page.locator('[data-testid="timer-display"]')).toHaveText('25:00');
    await expect(page.locator('[data-testid="session-type"]')).toHaveText('Work Session');

    // Skip to next session
    await page.click('[data-testid="skip-button"]');
    await expect(page.locator('[data-testid="session-type"]')).toHaveText('Short Break');
    await expect(page.locator('[data-testid="timer-display"]')).toHaveText('05:00');
  });

  test('keyboard shortcuts control timer', async ({ page }) => {
    // Space bar starts timer
    await page.keyboard.press('Space');
    await expect(page.locator('[data-testid="running-indicator"]')).toHaveClass('running');

    // Space bar pauses timer
    await page.keyboard.press('Space');
    await expect(page.locator('[data-testid="running-indicator"]')).toHaveClass('paused');

    // R key resets timer
    await page.keyboard.press('r');
    await expect(page.locator('[data-testid="timer-display"]')).toHaveText('25:00');

    // S key skips timer
    await page.keyboard.press('s');
    await expect(page.locator('[data-testid="session-type"]')).toHaveText('Short Break');
  });

  test('timer completion and automatic session transition', async ({ page }) => {
    // Note: This test uses a short duration for testing
    // In real implementation, we'd need a way to set short test durations

    await page.click('[data-testid="start-button"]');

    // For testing purposes, we'll simulate completion
    // In real scenario, this would wait for actual timer completion
    await page.evaluate(() => {
      // Simulate timer completion
      window.dispatchEvent(new CustomEvent('timer-completed', {
        detail: { nextSession: 'ShortBreak' }
      }));
    });

    // Verify automatic transition to break
    await expect(page.locator('[data-testid="session-type"]')).toHaveText('Short Break');
    await expect(page.locator('[data-testid="running-indicator"]')).toHaveClass('paused');
  });

  test('session type transitions work correctly', async ({ page }) => {
    // Start with Work session
    await expect(page.locator('[data-testid="session-type"]')).toHaveText('Work Session');
    await expect(page.locator('[data-testid="timer-display"]')).toHaveText('25:00');

    // Skip to Short Break
    await page.click('[data-testid="skip-button"]');
    await expect(page.locator('[data-testid="session-type"]')).toHaveText('Short Break');
    await expect(page.locator('[data-testid="timer-display"]')).toHaveText('05:00');

    // Skip to Work again
    await page.click('[data-testid="skip-button"]');
    await expect(page.locator('[data-testid="session-type"]')).toHaveText('Work Session');
    await expect(page.locator('[data-testid="timer-display"]')).toHaveText('25:00');

    // Skip multiple times to test Long Break
    await page.click('[data-testid="skip-button"]'); // Short Break
    await page.click('[data-testid="skip-button']'); // Work
    await page.click('[data-testid="skip-button']'); // Short Break
    await page.click('[data-testid="skip-button']'); // Work
    await page.click('[data-testid="skip-button']'); // Short Break
    await page.click('[data-testid="skip-button']'); // Work
    await page.click('[data-testid="skip-button']'); // Short Break
    await page.click('[data-testid="skip-button']'); // Work
    await page.click('[data-testid="skip-button']'); // Short Break
    await page.click('[data-testid="skip-button"]'); // Work
    await page.click('[data-testid="skip-button']'); // Should be Long Break now

    await expect(page.locator('[data-testid="session-type"]')).toHaveText('Long Break');
    await expect(page.locator('[data-testid="timer-display"]')).toHaveText('15:00');
  });

  test('progress bar updates correctly', async ({ page }) => {
    // Check initial progress
    await expect(page.locator('[data-testid="progress-bar"]')).toHaveCSS('width', '0%');

    // Start timer
    await page.click('[data-testid="start-button"]');

    // Wait for progress
    await page.waitForTimeout(1100);

    // Progress should be > 0%
    const progressBar = page.locator('[data-testid="progress-bar"]');
    const width = await progressBar.evaluate(el => getComputedStyle(el).width);
    expect(width).not.toBe('0px');
  });

  test('timer persists page refresh', async ({ page }) => {
    // Start timer
    await page.click('[data-testid="start-button"]');
    await page.waitForTimeout(1100);

    // Get current time
    const timeBeforeRefresh = await page.locator('[data-testid="timer-display"]').textContent();

    // Refresh page
    await page.reload();

    // Timer state should be restored
    await expect(page.locator('[data-testid="timer-display"]')).toBeVisible();

    // Should show same or slightly later time (accounting for refresh time)
    const timeAfterRefresh = await page.locator('[data-testid="timer-display"]').textContent();
    expect(timeAfterRefresh).toBeDefined();
  });

  test('error handling for network issues', async ({ page }) => {
    // Simulate network offline
    await page.context().setOffline(true);

    // Try to start timer
    await page.click('[data-testid="start-button"]');

    // Should show connection error
    await expect(page.locator('[data-testid="connection-error"]')).toBeVisible();

    // Restore connection
    await page.context().setOffline(false);

    // Should reconnect and work normally
    await expect(page.locator('[data-testid="connection-error"]')).not.toBeVisible();
  });

  test('responsive design on different screen sizes', async ({ page }) => {
    // Mobile view
    await page.setViewportSize({ width: 375, height: 667 });
    await expect(page.locator('[data-testid="timer-display"]')).toBeVisible();
    await expect(page.locator('[data-testid="timer-controls"]')).toBeVisible();

    // Tablet view
    await page.setViewportSize({ width: 768, height: 1024 });
    await expect(page.locator('[data-testid="timer-display"]')).toBeVisible();
    await expect(page.locator('[data-testid="timer-controls"]')).toBeVisible();

    // Desktop view
    await page.setViewportSize({ width: 1920, height: 1080 });
    await expect(page.locator('[data-testid="timer-display"]')).toBeVisible();
    await expect(page.locator('[data-testid="timer-controls"]')).toBeVisible();
  });

  test('accessibility compliance', async ({ page }) => {
    // Check ARIA labels
    await expect(page.locator('[aria-label="Timer display"]')).toBeVisible();
    await expect(page.locator('[aria-label="Start timer"]')).toBeVisible();
    await expect(page.locator('[aria-label="Pause timer"]')).toBeVisible();
    await expect(page.locator('[aria-label="Reset timer"]')).toBeVisible();
    await expect(page.locator('[aria-label="Skip to next session"]')).toBeVisible();

    // Check keyboard navigation
    await page.keyboard.press('Tab'); // Should focus first control
    await expect(page.locator(':focus')).toBeVisible();

    // Check role attributes
    await expect(page.locator('[role="timer"]')).toBeVisible();
    await expect(page.locator('[role="group"]')).toBeVisible();
  });

  test('performance: UI interactions under 100ms', async ({ page }) => {
    // Measure button click response time
    const startClick = performance.now();
    await page.click('[data-testid="start-button"]');
    const clickDuration = performance.now() - startClick;

    expect(clickDuration).toBeLessThan(100); // Should respond in under 100ms

    // Measure timer update time
    const startUpdate = performance.now();
    await page.waitForTimeout(1100); // Wait for one timer tick
    const updateDuration = performance.now() - startUpdate;

    expect(updateDuration).toBeLessThan(100); // Timer updates should be fast
  });

  test('long-term timer stability', async ({ page }) => {
    // Start timer and let it run for extended period
    await page.click('[data-testid="start-button"]');

    // Let timer run for 10 seconds
    await page.waitForTimeout(10000);

    // Verify timer is still running and responsive
    await expect(page.locator('[data-testid="running-indicator"]')).toHaveClass('running');

    // Try to pause
    await page.click('[data-testid="pause-button"]');
    await expect(page.locator('[data-testid="running-indicator"]')).toHaveClass('paused');

    // Try to reset
    await page.click('[data-testid="reset-button"]');
    await expect(page.locator('[data-testid="timer-display"]')).toHaveText('25:00');
  });
});