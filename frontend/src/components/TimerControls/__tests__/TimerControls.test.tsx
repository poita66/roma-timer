import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { TimerControls } from '../TimerControls';
import { TimerSession } from '../../types';

const mockTimerSession: TimerSession = {
  id: 'test-session-1',
  duration: 1500,
  elapsed: 0,
  timerType: 'Work',
  isRunning: false,
  createdAt: 1698569400,
  updatedAt: 1698569400,
};

describe('TimerControls Component', () => {
  const mockOnStart = jest.fn();
  const mockOnPause = jest.fn();
  const mockOnReset = jest.fn();
  const mockOnSkip = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();
  });

  test('renders all control buttons', () => {
    render(
      <TimerControls
        session={mockTimerSession}
        onStart={mockOnStart}
        onPause={mockOnPause}
        onReset={mockOnReset}
        onSkip={mockOnSkip}
      />
    );

    expect(screen.getByRole('button', { name: /start/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /pause/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /reset/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /skip/i })).toBeInTheDocument();
  });

  test('shows start button when timer is stopped', () => {
    render(
      <TimerControls
        session={mockTimerSession}
        onStart={mockOnStart}
        onPause={mockOnPause}
        onReset={mockOnReset}
        onSkip={mockOnSkip}
      />
    );

    const startButton = screen.getByRole('button', { name: /start/i });
    expect(startButton).toBeInTheDocument();
    expect(startButton).not.toBeDisabled();
  });

  test('shows pause button when timer is running', () => {
    const runningSession = { ...mockTimerSession, isRunning: true };

    render(
      <TimerControls
        session={runningSession}
        onStart={mockOnStart}
        onPause={mockOnPause}
        onReset={mockOnReset}
        onSkip={mockOnSkip}
      />
    );

    const pauseButton = screen.getByRole('button', { name: /pause/i });
    expect(pauseButton).toBeInTheDocument();
    expect(pauseButton).not.toBeDisabled();
  });

  test('calls onStart when start button is clicked', async () => {
    const user = userEvent.setup();

    render(
      <TimerControls
        session={mockTimerSession}
        onStart={mockOnStart}
        onPause={mockOnPause}
        onReset={mockOnReset}
        onSkip={mockOnSkip}
      />
    );

    const startButton = screen.getByRole('button', { name: /start/i });
    await user.click(startButton);

    expect(mockOnStart).toHaveBeenCalledTimes(1);
  });

  test('calls onPause when pause button is clicked', async () => {
    const user = userEvent.setup();
    const runningSession = { ...mockTimerSession, isRunning: true };

    render(
      <TimerControls
        session={runningSession}
        onStart={mockOnStart}
        onPause={mockOnPause}
        onReset={mockOnReset}
        onSkip={mockOnSkip}
      />
    );

    const pauseButton = screen.getByRole('button', { name: /pause/i });
    await user.click(pauseButton);

    expect(mockOnPause).toHaveBeenCalledTimes(1);
  });

  test('calls onReset when reset button is clicked', async () => {
    const user = userEvent.setup();

    render(
      <TimerControls
        session={mockTimerSession}
        onStart={mockOnStart}
        onPause={mockOnPause}
        onReset={mockOnReset}
        onSkip={mockOnSkip}
      />
    );

    const resetButton = screen.getByRole('button', { name: /reset/i });
    await user.click(resetButton);

    expect(mockOnReset).toHaveBeenCalledTimes(1);
  });

  test('calls onSkip when skip button is clicked', async () => {
    const user = userEvent.setup();

    render(
      <TimerControls
        session={mockTimerSession}
        onStart={mockOnStart}
        onPause={mockOnPause}
        onReset={mockOnReset}
        onSkip={mockOnSkip}
      />
    );

    const skipButton = screen.getByRole('button', { name: /skip/i });
    await user.click(skipButton);

    expect(mockOnSkip).toHaveBeenCalledTimes(1);
  });

  test('keyboard shortcuts work correctly', async () => {
    const user = userEvent.setup();

    render(
      <TimerControls
        session={mockTimerSession}
        onStart={mockOnStart}
        onPause={mockOnPause}
        onReset={mockOnReset}
        onSkip={mockOnSkip}
      />
    );

    // Space bar should toggle start/pause
    await user.keyboard(' ');
    expect(mockOnStart).toHaveBeenCalledTimes(1);

    // R key should reset
    await user.keyboard('r');
    expect(mockOnReset).toHaveBeenCalledTimes(1);

    // S key should skip
    await user.keyboard('s');
    expect(mockOnSkip).toHaveBeenCalledTimes(1);
  });

  test('space bar toggles between start and pause', async () => {
    const user = userEvent.setup();
    let session = { ...mockTimerSession, isRunning: false };

    const { rerender } = render(
      <TimerControls
        session={session}
        onStart={mockOnStart}
        onPause={mockOnPause}
        onReset={mockOnReset}
        onSkip={mockOnSkip}
      />
    );

    // Start with space
    await user.keyboard(' ');
    expect(mockOnStart).toHaveBeenCalledTimes(1);

    // Simulate timer now running
    session = { ...session, isRunning: true };
    rerender(
      <TimerControls
        session={session}
        onStart={mockOnStart}
        onPause={mockOnPause}
        onReset={mockOnReset}
        onSkip={mockOnSkip}
      />
    );

    // Pause with space
    await user.keyboard(' ');
    expect(mockOnPause).toHaveBeenCalledTimes(1);
  });

  test('disables buttons when loading', () => {
    render(
      <TimerControls
        session={mockTimerSession}
        onStart={mockOnStart}
        onPause={mockOnPause}
        onReset={mockOnReset}
        onSkip={mockOnSkip}
        loading={true}
      />
    );

    expect(screen.getByRole('button', { name: /start/i })).toBeDisabled();
    expect(screen.getByRole('button', { name: /reset/i })).toBeDisabled();
    expect(screen.getByRole('button', { name: /skip/i })).toBeDisabled();
  });

  test('has proper ARIA labels', () => {
    render(
      <TimerControls
        session={mockTimerSession}
        onStart={mockOnStart}
        onPause={mockOnPause}
        onReset={mockOnReset}
        onSkip={mockOnSkip}
      />
    );

    const startButton = screen.getByRole('button', { name: /start/i });
    expect(startButton).toHaveAttribute('aria-label', 'Start timer');

    const pauseButton = screen.getByRole('button', { name: /pause/i });
    expect(pauseButton).toHaveAttribute('aria-label', 'Pause timer');

    const resetButton = screen.getByRole('button', { name: /reset/i });
    expect(resetButton).toHaveAttribute('aria-label', 'Reset timer');

    const skipButton = screen.getByRole('button', { name: /skip/i });
    expect(skipButton).toHaveAttribute('aria-label', 'Skip to next session');
  });

  test('button states update based on timer session', () => {
    const { rerender } = render(
      <TimerControls
        session={mockTimerSession}
        onStart={mockOnStart}
        onPause={mockOnPause}
        onReset={mockOnReset}
        onSkip={mockOnSkip}
      />
    );

    // Initially stopped, start button visible
    expect(screen.getByRole('button', { name: /start/i })).toBeInTheDocument();

    // Change to running state
    const runningSession = { ...mockTimerSession, isRunning: true };
    rerender(
      <TimerControls
        session={runningSession}
        onStart={mockOnStart}
        onPause={mockOnPause}
        onReset={mockOnReset}
        onSkip={mockOnSkip}
      />
    );

    // Now pause button visible
    expect(screen.getByRole('button', { name: /pause/i })).toBeInTheDocument();
  });

  test('prevents rapid button clicks', async () => {
    const user = userEvent.setup();

    render(
      <TimerControls
        session={mockTimerSession}
        onStart={mockOnStart}
        onPause={mockOnPause}
        onReset={mockOnReset}
        onSkip={mockOnSkip}
      />
    );

    const startButton = screen.getByRole('button', { name: /start/i });

    // Rapid clicks
    await user.click(startButton);
    await user.click(startButton);
    await user.click(startButton);

    // Should only call once due to debouncing/prevention
    expect(mockOnStart).toHaveBeenCalledTimes(1);
  });

  test('handles null session gracefully', () => {
    render(
      <TimerControls
        session={null as any}
        onStart={mockOnStart}
        onPause={mockOnPause}
        onReset={mockOnReset}
        onSkip={mockOnSkip}
      />
    );

    expect(screen.getByText('No timer session')).toBeInTheDocument();
  });

  test('supports different themes', () => {
    const { container } = render(
      <TimerControls
        session={mockTimerSession}
        onStart={mockOnStart}
        onPause={mockOnPause}
        onReset={mockOnReset}
        onSkip={mockOnSkip}
        theme="dark"
      />
    );

    expect(container.querySelector('.timer-controls.dark')).toBeInTheDocument();
  });

  test('is accessible via screen reader', async () => {
    render(
      <TimerControls
        session={mockTimerSession}
        onStart={mockOnStart}
        onPause={mockOnPause}
        onReset={mockOnReset}
        onSkip={mockOnSkip}
      />
    );

    const controlsContainer = screen.getByRole('group', { name: /timer controls/i });
    expect(controlsContainer).toBeInTheDocument();

    // Check that all buttons are properly labeled for screen readers
    const buttons = screen.getAllByRole('button');
    buttons.forEach(button => {
      expect(button).toHaveAttribute('aria-label');
    });
  });
});