//! Settings Components Tests
//!
//! Component tests for all Settings components including DurationSettings,
//! NotificationSettings, and GeneralSettings components

import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import '@testing-library/jest-dom';
import { UserConfiguration } from '../../../types';
import DurationSettings from '../DurationSettings';
import NotificationSettings from '../NotificationSettings';
import GeneralSettings from '../GeneralSettings';

// Mock configuration for testing
const mockConfig: UserConfiguration = {
  id: 'test-config',
  workDuration: 1500,        // 25 minutes
  shortBreakDuration: 300,  // 5 minutes
  longBreakDuration: 900,   // 15 minutes
  longBreakFrequency: 4,
  notificationsEnabled: true,
  webhookUrl: undefined,
  waitForInteraction: false,
  theme: 'Light',
  createdAt: Date.now() / 1000,
  updatedAt: Date.now() / 1000,
};

describe('DurationSettings Component', () => {
  const mockOnChange = jest.fn();

  beforeEach(() => {
    mockOnChange.mockClear();
  });

  test('renders duration settings with default values', () => {
    render(
      <DurationSettings
        config={mockConfig}
        onChange={mockOnChange}
      />
    );

    expect(screen.getByLabelText(/Work Session Duration/i)).toHaveValue(25);
    expect(screen.getByLabelText(/Short Break Duration/i)).toHaveValue(5);
    expect(screen.getByLabelText(/Long Break Duration/i)).toHaveValue(15);
    expect(screen.getByLabelText(/Long Break After/i)).toHaveValue(4);
  });

  test('calls onChange when work duration is changed', async () => {
    const user = userEvent.setup();
    render(
      <DurationSettings
        config={mockConfig}
        onChange={mockOnChange}
      />
    );

    const workDurationInput = screen.getByLabelText(/Work Session Duration/i);
    await user.clear(workDurationInput);
    await user.type(workDurationInput, '30');

    expect(mockOnChange).toHaveBeenCalledWith('workDuration', 1800); // 30 minutes in seconds
  });

  test('calls onChange when short break duration is changed', async () => {
    const user = userEvent.setup();
    render(
      <DurationSettings
        config={mockConfig}
        onChange={mockOnChange}
      />
    );

    const shortBreakInput = screen.getByLabelText(/Short Break Duration/i);
    await user.clear(shortBreakInput);
    await user.type(shortBreakInput, '10');

    expect(mockOnChange).toHaveBeenCalledWith('shortBreakDuration', 600); // 10 minutes in seconds
  });

  test('calls onChange when long break duration is changed', async () => {
    const user = userEvent.setup();
    render(
      <DurationSettings
        config={mockConfig}
        onChange={mockOnChange}
      />
    );

    const longBreakInput = screen.getByLabelText(/Long Break Duration/i);
    await user.clear(longBreakInput);
    await user.type(longBreakInput, '20');

    expect(mockOnChange).toHaveBeenCalledWith('longBreakDuration', 1200); // 20 minutes in seconds
  });

  test('calls onChange when long break frequency is changed', async () => {
    const user = userEvent.setup();
    render(
      <DurationSettings
        config={mockConfig}
        onChange={mockOnChange}
      />
    );

    const frequencyInput = screen.getByLabelText(/Long Break After/i);
    await user.clear(frequencyInput);
    await user.type(frequencyInput, '6');

    expect(mockOnChange).toHaveBeenCalledWith('longBreakFrequency', 6);
  });

  test('displays validation errors', () => {
    const errors = {
      workDuration: 'Work duration must be between 5 and 60 minutes',
      shortBreakDuration: 'Short break duration must be between 1 and 15 minutes',
    };

    render(
      <DurationSettings
        config={mockConfig}
        onChange={mockOnChange}
        errors={errors}
      />
    );

    expect(screen.getByText('Work duration must be between 5 and 60 minutes')).toBeInTheDocument();
    expect(screen.getByText('Short break duration must be between 1 and 15 minutes')).toBeInTheDocument();
  });

  test('disables inputs when disabled prop is true', () => {
    render(
      <DurationSettings
        config={mockConfig}
        onChange={mockOnChange}
        disabled={true}
      />
    );

    expect(screen.getByLabelText(/Work Session Duration/i)).toBeDisabled();
    expect(screen.getByLabelText(/Short Break Duration/i)).toBeDisabled();
    expect(screen.getByLabelText(/Long Break Duration/i)).toBeDisabled();
    expect(screen.getByLabelText(/Long Break After/i)).toBeDisabled();
  });

  test('displays schedule summary', () => {
    render(
      <DurationSettings
        config={mockConfig}
        onChange={mockOnChange}
      />
    );

    expect(screen.getByText(/Work for 25 minutes, then take a 5 minute break./)).toBeInTheDocument();
    expect(screen.getByText(/After 4 work sessions, take a 15 minute long break./)).toBeInTheDocument();
  });
});

describe('NotificationSettings Component', () => {
  const mockOnChange = jest.fn();

  beforeEach(() => {
    mockOnChange.mockClear();
  });

  test('renders notification settings with default values', () => {
    render(
      <NotificationSettings
        config={mockConfig}
        onChange={mockOnChange}
      />
    );

    expect(screen.getByRole('switch', { name: /Browser Notifications/i })).toHaveAttribute('aria-checked', 'true');
    expect(screen.getByRole('switch', { name: /Wait for User Interaction/i })).toHaveAttribute('aria-checked', 'false');
  });

  test('toggles browser notifications', async () => {
    const user = userEvent.setup();
    render(
      <NotificationSettings
        config={mockConfig}
        onChange={mockOnChange}
      />
    );

    const browserNotificationsSwitch = screen.getByRole('switch', { name: /Browser Notifications/i });
    await user.click(browserNotificationsSwitch);

    expect(mockOnChange).toHaveBeenCalledWith('notificationsEnabled', false);
  });

  test('toggles wait for interaction', async () => {
    const user = userEvent.setup();
    render(
      <NotificationSettings
        config={mockConfig}
        onChange={mockOnChange}
      />
    );

    const waitInteractionSwitch = screen.getByRole('switch', { name: /Wait for User Interaction/i });
    await user.click(waitInteractionSwitch);

    expect(mockOnChange).toHaveBeenCalledWith('waitForInteraction', true);
  });

  test('shows webhook configuration when configure button is clicked', async () => {
    const user = userEvent.setup();
    render(
      <NotificationSettings
        config={mockConfig}
        onChange={mockOnChange}
      />
    );

    const configureButton = screen.getByRole('button', { name: /Configure/i });
    await user.click(configureButton);

    expect(screen.getByLabelText(/Webhook URL/i)).toBeInTheDocument();
    expect(screen.getByPlaceholderText(/https:\/\/example.com\/webhook/i)).toBeInTheDocument();
  });

  test('calls onChange when webhook URL is entered', async () => {
    const user = userEvent.setup();
    render(
      <NotificationSettings
        config={mockConfig}
        onChange={mockOnChange}
      />
    );

    // Show webhook configuration
    const configureButton = screen.getByRole('button', { name: /Configure/i });
    await user.click(configureButton);

    const webhookInput = screen.getByLabelText(/Webhook URL/i);
    await user.type(webhookInput, 'https://example.com/webhook');

    // Wait for debounced onChange
    await waitFor(() => {
      expect(mockOnChange).toHaveBeenCalledWith('webhookUrl', 'https://example.com/webhook');
    });
  });

  test('displays webhook validation errors', async () => {
    const user = userEvent.setup();
    const errors = {
      webhookUrl: 'Invalid webhook URL format',
    };

    render(
      <NotificationSettings
        config={mockConfig}
        onChange={mockOnChange}
        errors={errors}
      />
    );

    // Show webhook configuration
    const configureButton = screen.getByRole('button', { name: /Configure/i });
    await user.click(configureButton);

    expect(screen.getByText('Invalid webhook URL format')).toBeInTheDocument();
  });

  test('disables controls when disabled prop is true', () => {
    render(
      <NotificationSettings
        config={mockConfig}
        onChange={mockOnChange}
        disabled={true}
      />
    );

    expect(screen.getByRole('switch', { name: /Browser Notifications/i })).toBeDisabled();
    expect(screen.getByRole('switch', { name: /Wait for User Interaction/i })).toBeDisabled();
  });

  test('shows webhook success message when URL is configured', () => {
    const configWithWebhook = {
      ...mockConfig,
      webhookUrl: 'https://example.com/webhook',
    };

    render(
      <NotificationSettings
        config={configWithWebhook}
        onChange={mockOnChange}
      />
    );

    expect(screen.getByText(/✅ Webhook configured successfully/)).toBeInTheDocument();
  });
});

describe('GeneralSettings Component', () => {
  const mockOnChange = jest.fn();

  beforeEach(() => {
    mockOnChange.mockClear();
  });

  test('renders general settings with default values', () => {
    render(
      <GeneralSettings
        config={mockConfig}
        onChange={mockOnChange}
      />
    );

    // Check that Light theme is selected
    expect(screen.getByLabelText(/Light Mode/i)).toBeChecked();
    expect(screen.getByLabelText(/Dark Mode/i)).not.toBeChecked();
  });

  test('selects dark theme when dark theme is configured', () => {
    const darkConfig = {
      ...mockConfig,
      theme: 'Dark' as const,
    };

    render(
      <GeneralSettings
        config={darkConfig}
        onChange={mockOnChange}
      />
    );

    expect(screen.getByLabelText(/Light Mode/i)).not.toBeChecked();
    expect(screen.getByLabelText(/Dark Mode/i)).toBeChecked();
  });

  test('calls onChange when theme is changed', async () => {
    const user = userEvent.setup();
    render(
      <GeneralSettings
        config={mockConfig}
        onChange={mockOnChange}
      />
    );

    const darkThemeRadio = screen.getByLabelText(/Dark Mode/i);
    await user.click(darkThemeRadio);

    expect(mockOnChange).toHaveBeenCalledWith('theme', 'Dark');
  });

  test('calls onChange when light theme is selected', async () => {
    const user = userEvent.setup();
    const darkConfig = {
      ...mockConfig,
      theme: 'Dark' as const,
    };

    render(
      <GeneralSettings
        config={darkConfig}
        onChange={mockOnChange}
      />
    );

    const lightThemeRadio = screen.getByLabelText(/Light Mode/i);
    await user.click(lightThemeRadio);

    expect(mockOnChange).toHaveBeenCalledWith('theme', 'Light');
  });

  test('displays theme validation errors', () => {
    const errors = {
      theme: 'Theme must be either "Light" or "Dark"',
    };

    render(
      <GeneralSettings
        config={mockConfig}
        onChange={mockOnChange}
        errors={errors}
      />
    );

    expect(screen.getByText('Theme must be either "Light" or "Dark"')).toBeInTheDocument();
  });

  test('disables radio buttons when disabled prop is true', () => {
    render(
      <GeneralSettings
        config={mockConfig}
        onChange={mockOnChange}
        disabled={true}
      />
    );

    expect(screen.getByLabelText(/Light Mode/i)).toBeDisabled();
    expect(screen.getByLabelText(/Dark Mode/i)).toBeDisabled();
  });

  test('shows correct theme preview for light theme', () => {
    render(
      <GeneralSettings
        config={mockConfig}
        onChange={mockOnChange}
      />
    );

    expect(screen.getByText(/This is how your timer will look with light mode./)).toBeInTheDocument();
  });

  test('shows correct theme preview for dark theme', () => {
    const darkConfig = {
      ...mockConfig,
      theme: 'Dark' as const,
    };

    render(
      <GeneralSettings
        config={darkConfig}
        onChange={mockOnChange}
      />
    );

    expect(screen.getByText(/This is how your timer will look with dark mode./)).toBeInTheDocument();
  });
});

describe('Settings Components Integration', () => {
  test('all settings components handle empty errors object', () => {
    render(
      <DurationSettings
        config={mockConfig}
        onChange={jest.fn()}
        errors={{}}
      />
    );

    render(
      <NotificationSettings
        config={mockConfig}
        onChange={jest.fn()}
        errors={{}}
      />
    );

    render(
      <GeneralSettings
        config={mockConfig}
        onChange={jest.fn()}
        errors={{}}
      />
    );

    // Should not display any error messages
    expect(screen.queryByText(/must be between/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/Invalid/i)).not.toBeInTheDocument();
  });

  test('all settings components handle undefined errors', () => {
    render(
      <DurationSettings
        config={mockConfig}
        onChange={jest.fn()}
        errors={undefined}
      />
    );

    render(
      <NotificationSettings
        config={mockConfig}
        onChange={jest.fn()}
        errors={undefined}
      />
    );

    render(
      <GeneralSettings
        config={mockConfig}
        onChange={jest.fn()}
        errors={undefined}
      />
    );

    // Should render without errors
    expect(screen.getByLabelText(/Work Session Duration/i)).toBeInTheDocument();
    expect(screen.getByRole('switch', { name: /Browser Notifications/i })).toBeInTheDocument();
    expect(screen.getByLabelText(/Light Mode/i)).toBeInTheDocument();
  });
});