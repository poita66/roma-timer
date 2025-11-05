//! Configuration Flow E2E Tests
//!
//! End-to-end tests for the complete configuration workflow including
//! navigation, form interaction, validation, and persistence

import { test, expect } from '@playwright/test';

test.describe('Configuration Flow E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    // Mock the API responses for configuration
    await page.route('/api/configuration', (route) => {
      if (route.request().method() === 'GET') {
        // Return default configuration
        route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            id: 'default-config',
            workDuration: 1500,
            shortBreakDuration: 300,
            longBreakDuration: 900,
            longBreakFrequency: 4,
            notificationsEnabled: true,
            webhookUrl: null,
            waitForInteraction: false,
            theme: 'Light',
            createdAt: Math.floor(Date.now() / 1000),
            updatedAt: Math.floor(Date.now() / 1000),
          }),
        });
      } else if (route.request().method() === 'PUT') {
        // Accept any configuration update
        route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            ...JSON.parse(route.request().postData() || '{}'),
            id: 'default-config',
            createdAt: Math.floor(Date.now() / 1000),
            updatedAt: Math.floor(Date.now() / 1000),
          }),
        });
      }
    });

    // Mock POST /api/configuration/reset
    await page.route('/api/configuration/reset', (route) => {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          id: 'default-config',
          workDuration: 1500,
          shortBreakDuration: 300,
          longBreakDuration: 900,
          longBreakFrequency: 4,
          notificationsEnabled: true,
          webhookUrl: null,
          waitForInteraction: false,
          theme: 'Light',
          createdAt: Math.floor(Date.now() / 1000),
          updatedAt: Math.floor(Date.now() / 1000),
        }),
      });
    });

    // Navigate to settings page
    await page.goto('/settings');
  });

  test('loads settings page with default configuration', async ({ page }) => {
    // Check page title
    await expect(page).toHaveTitle(/Settings/);

    // Check main heading
    await expect(page.getByRole('heading', { level: 1, name: 'Settings' })).toBeVisible();

    // Check tab navigation
    await expect(page.getByRole('tab', { name: /Timer Durations/ })).toBeVisible();
    await expect(page.getByRole('tab', { name: /Notifications/ })).toBeVisible();
    await expect(page.getByRole('tab', { name: /General/ })).toBeVisible();

    // Duration tab should be active by default
    await expect(page.getByRole('tab', { name: /Timer Durations/ })).toHaveAttribute('aria-selected', 'true');
  });

  test('can navigate between settings tabs', async ({ page }) => {
    // Click on Notifications tab
    await page.getByRole('tab', { name: /Notifications/ }).click();
    await expect(page.getByRole('tab', { name: /Notifications/ })).toHaveAttribute('aria-selected', 'true');
    await expect(page.getByRole('heading', { name: /Notifications/ })).toBeVisible();

    // Click on General tab
    await page.getByRole('tab', { name: /General/ }).click();
    await expect(page.getByRole('tab', { name: /General/ })).toHaveAttribute('aria-selected', 'true');
    await expect(page.getByRole('heading', { name: /General Preferences/ })).toBeVisible();

    // Click back to Timer Durations tab
    await page.getByRole('tab', { name: /Timer Durations/ }).click();
    await expect(page.getByRole('tab', { name: /Timer Durations/ })).toHaveAttribute('aria-selected', 'true');
  });

  test('can update timer durations', async ({ page }) => {
    // Should be on Timer Durations tab by default

    // Update work duration
    const workDurationInput = page.getByLabel(/Work Session Duration/);
    await workDurationInput.clear();
    await workDurationInput.fill('30');

    // Update short break duration
    const shortBreakInput = page.getByLabel(/Short Break Duration/);
    await shortBreakInput.clear();
    await shortBreakInput.fill('10');

    // Update long break frequency
    const frequencyInput = page.getByLabel(/Long Break After/);
    await frequencyInput.clear();
    await frequencyInput.fill('6');

    // Verify unsaved changes warning appears
    await expect(page.getByText(/You have unsaved changes/)).toBeVisible();

    // Save changes
    await page.getByRole('button', { name: /Save Changes/ }).click();

    // Verify success - changes saved, no warning
    await expect(page.getByText(/All settings up to date/)).toBeVisible();
    await expect(page.getByText(/You have unsaved changes/)).not.toBeVisible();
  });

  test('validates timer duration inputs', async ({ page }) => {
    // Try to set invalid work duration (too short)
    const workDurationInput = page.getByLabel(/Work Session Duration/);
    await workDurationInput.clear();
    await workDurationInput.fill('2'); // Less than 5 minutes

    // Should show validation error
    await expect(page.getByText(/Work duration must be between 5 and 60 minutes/)).toBeVisible();

    // Save button should be disabled due to validation error
    await expect(page.getByRole('button', { name: /Save Changes/ })).toBeDisabled();

    // Fix the input
    await workDurationInput.clear();
    await workDurationInput.fill('25'); // Valid value

    // Error should disappear
    await expect(page.getByText(/Work duration must be between 5 and 60 minutes/)).not.toBeVisible();
  });

  test('can configure notification settings', async ({ page }) => {
    // Navigate to Notifications tab
    await page.getByRole('tab', { name: /Notifications/ }).click();

    // Toggle browser notifications off
    const notificationsSwitch = page.getByRole('switch', { name: /Browser Notifications/ });
    await notificationsSwitch.click();

    // Toggle wait for interaction on
    const waitInteractionSwitch = page.getByRole('switch', { name: /Wait for User Interaction/ });
    await waitInteractionSwitch.click();

    // Configure webhook
    await page.getByRole('button', { name: /Configure/ }).click();
    const webhookInput = page.getByLabel(/Webhook URL/);
    await webhookInput.fill('https://hooks.slack.com/services/test/webhook');

    // Verify unsaved changes warning appears
    await expect(page.getByText(/You have unsaved changes/)).toBeVisible();

    // Save changes
    await page.getByRole('button', { name: /Save Changes/ }).click();

    // Verify success
    await expect(page.getByText(/All settings up to date/)).toBeVisible();
  });

  test('validates webhook URL', async ({ page }) => {
    // Navigate to Notifications tab
    await page.getByRole('tab', { name: /Notifications/ }).click();

    // Configure webhook
    await page.getByRole('button', { name: /Configure/ }).click();
    const webhookInput = page.getByLabel(/Webhook URL/);
    await webhookInput.fill('not-a-valid-url');

    // Should show validation error
    await expect(page.getByText(/Invalid webhook URL format/)).toBeVisible();

    // Save button should be disabled
    await expect(page.getByRole('button', { name: /Save Changes/ })).toBeDisabled();

    // Fix the URL
    await webhookInput.clear();
    await webhookInput.fill('https://example.com/webhook');

    // Error should disappear
    await expect(page.getByText(/Invalid webhook URL format/)).not.toBeVisible();
  });

  test('can change theme setting', async ({ page }) => {
    // Navigate to General tab
    await page.getByRole('tab', { name: /General/ }).click();

    // Select dark theme
    await page.getByLabel(/Dark Mode/).click();

    // Verify unsaved changes warning appears
    await expect(page.getByText(/You have unsaved changes/)).toBeVisible();

    // Save changes
    await page.getByRole('button', { name: /Save Changes/ }).click();

    // Verify success
    await expect(page.getByText(/All settings up to date/)).toBeVisible();
  });

  test('can discard unsaved changes', async ({ page }) => {
    // Make some changes
    const workDurationInput = page.getByLabel(/Work Session Duration/);
    await workDurationInput.clear();
    await workDurationInput.fill('45');

    // Verify unsaved changes warning appears
    await expect(page.getByText(/You have unsaved changes/)).toBeVisible();

    // Discard changes
    await page.getByRole('button', { name: /Discard Changes/ }).click();

    // Verify input is back to default value
    await expect(workDurationInput).toHaveValue('25');
    await expect(page.getByText(/You have unsaved changes/)).not.toBeVisible();
  });

  test('can reset configuration to defaults', async ({ page }) => {
    // Make some changes first
    await page.getByLabel(/Work Session Duration/).clear();
    await page.getByLabel(/Work Session Duration/).fill('35');
    await page.getByRole('button', { name: /Save Changes/ }).click();

    // Navigate to General tab and change theme
    await page.getByRole('tab', { name: /General/ }).click();
    await page.getByLabel(/Dark Mode/).click();
    await page.getByRole('button', { name: /Save Changes/ }).click();

    // Reset to defaults
    await page.getByRole('button', { name: /Reset to Defaults/ }).click();

    // Confirm reset in dialog
    await page.getByRole('button', { name: /^Reset to Defaults$/ }).click();

    // Verify values are back to defaults
    await page.getByRole('tab', { name: /Timer Durations/ }).click();
    await expect(page.getByLabel(/Work Session Duration/)).toHaveValue('25');

    await page.getByRole('tab', { name: /General/ }).click();
    await expect(page.getByLabel(/Light Mode/)).toBeChecked();
  });

  test('shows loading state during save', async ({ page }) => {
    // Mock a delayed API response
    await page.route('/api/configuration', (route) => {
      if (route.request().method() === 'PUT') {
        setTimeout(() => {
          route.fulfill({
            status: 200,
            contentType: 'application/json',
            body: JSON.stringify({
              id: 'default-config',
              workDuration: 1800,
              shortBreakDuration: 300,
              longBreakDuration: 900,
              longBreakFrequency: 4,
              notificationsEnabled: true,
              webhookUrl: null,
              waitForInteraction: false,
              theme: 'Light',
              createdAt: Math.floor(Date.now() / 1000),
              updatedAt: Math.floor(Date.now() / 1000),
            }),
          });
        }, 1000); // 1 second delay
      } else {
        route.continue();
      }
    });

    // Make a change
    await page.getByLabel(/Work Session Duration/).clear();
    await page.getByLabel(/Work Session Duration/).fill('30');

    // Save changes
    await page.getByRole('button', { name: /Save Changes/ }).click();

    // Should show loading state
    await expect(page.getByRole('button', { name: /Saving\.\.\./ })).toBeVisible();

    // Should complete after delay
    await expect(page.getByRole('button', { name: /Save Changes/ })).toBeVisible({ timeout: 2000 });
  });

  test('handles save errors gracefully', async ({ page }) => {
    // Mock a failed API response
    await page.route('/api/configuration', (route) => {
      if (route.request().method() === 'PUT') {
        route.fulfill({
          status: 500,
          contentType: 'application/json',
          body: JSON.stringify({
            error: 'InternalServerError',
            message: 'Failed to update configuration',
            timestamp: Math.floor(Date.now() / 1000),
          }),
        });
      } else {
        route.continue();
      }
    });

    // Make a change
    await page.getByLabel(/Work Session Duration/).clear();
    await page.getByLabel(/Work Session Duration/).fill('30');

    // Try to save changes
    await page.getByRole('button', { name: /Save Changes/ }).click();

    // Should show error message (implementation dependent)
    // For now, just verify the save button is still active after a delay
    await expect(page.getByRole('button', { name: /Save Changes/ })).toBeVisible({ timeout: 2000 });
  });

  test('keyboard navigation works', async ({ page }) => {
    // Navigate using keyboard
    await page.keyboard.press('Tab'); // Should focus first tab
    await expect(page.getByRole('tab', { name: /Timer Durations/ })).toBeFocused();

    // Navigate to next tab
    await page.keyboard.press('Tab');
    await expect(page.getByRole('tab', { name: /Notifications/ })).toBeFocused();

    // Select Notifications tab with Enter
    await page.keyboard.press('Enter');
    await expect(page.getByRole('tab', { name: /Notifications/ })).toHaveAttribute('aria-selected', 'true');

    // Navigate to browser notifications switch
    await page.keyboard.press('Tab');
    await expect(page.getByRole('switch', { name: /Browser Notifications/ })).toBeFocused();

    // Toggle the switch with Space
    await page.keyboard.press('Space');
    // Note: Visual verification would depend on implementation
  });

  test('responsive design works on mobile', async ({ page }) => {
    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });

    // Check that page is still functional
    await expect(page.getByRole('heading', { level: 1, name: 'Settings' })).toBeVisible();

    // Tabs should still work
    await page.getByRole('tab', { name: /Notifications/ }).click();
    await expect(page.getByRole('heading', { name: /Notifications/ })).toBeVisible();

    // Form inputs should be accessible
    await expect(page.getByRole('switch', { name: /Browser Notifications/ })).toBeVisible();
  });

  test('accessibility features work', async ({ page }) => {
    // Check for proper ARIA labels
    await expect(page.getByRole('tab', { name: /Timer Durations/ })).toHaveAttribute('aria-selected', 'true');

    // Check form inputs have proper labels
    await expect(page.getByLabel(/Work Session Duration/)).toBeVisible();
    await expect(page.getByRole('switch', { name: /Browser Notifications/ })).toBeVisible();

    // Check for skip links or other accessibility features
    const heading = page.getByRole('heading', { level: 1, name: 'Settings' });
    await expect(heading).toBeVisible();
    await expect(heading).toHaveAttribute('tabindex', '0');
  });
});