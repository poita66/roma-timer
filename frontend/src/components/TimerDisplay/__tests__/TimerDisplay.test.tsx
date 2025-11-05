import React from 'react';
import { render, screen } from '@testing-library/react';
import { TimerDisplay } from '../TimerDisplay';
import { TimerSession } from '../../types';

// Mock timer session data
const mockTimerSession: TimerSession = {
  id: 'test-session-1',
  duration: 1500, // 25 minutes
  elapsed: 750,   // 12.5 minutes elapsed
  timerType: 'Work',
  isRunning: true,
  createdAt: 1698569400,
  updatedAt: 1698569400,
};

describe('TimerDisplay Component', () => {
  test('displays remaining time correctly', () => {
    render(<TimerDisplay session={mockTimerSession} />);

    // Should display "12:30" (12 minutes 30 seconds remaining)
    const timeDisplay = screen.getByText('12:30');
    expect(timeDisplay).toBeInTheDocument();
  });

  test('displays session type indicator', () => {
    render(<TimerDisplay session={mockTimerSession} />);

    const sessionType = screen.getByText('Work Session');
    expect(sessionType).toBeInTheDocument();
  });

  test('shows running indicator when timer is active', () => {
    render(<TimerDisplay session={mockTimerSession} />);

    const runningIndicator = screen.getByTestId('running-indicator');
    expect(runningIndicator).toHaveClass('running');
  });

  test('shows paused indicator when timer is stopped', () => {
    const pausedSession: TimerSession = {
      ...mockTimerSession,
      isRunning: false,
    };

    render(<TimerDisplay session={pausedSession} />);

    const runningIndicator = screen.getByTestId('running-indicator');
    expect(runningIndicator).toHaveClass('paused');
  });

  test('displays break session correctly', () => {
    const breakSession: TimerSession = {
      ...mockTimerSession,
      timerType: 'ShortBreak',
      duration: 300, // 5 minutes
      elapsed: 120,  // 2 minutes elapsed
    };

    render(<TimerDisplay session={breakSession} />);

    expect(screen.getByText('Short Break')).toBeInTheDocument();
    expect(screen.getByText('03:00')).toBeInTheDocument(); // 3 minutes remaining
  });

  test('formats time correctly with leading zeros', () => {
    const shortSession: TimerSession = {
      ...mockTimerSession,
      duration: 65, // 1 minute 5 seconds
      elapsed: 10,  // 10 seconds elapsed
    };

    render(<TimerDisplay session={shortSession} />);

    // Should display "00:55" (55 seconds remaining)
    expect(screen.getByText('00:55')).toBeInTheDocument();
  });

  test('shows completion state when timer reaches duration', () => {
    const completedSession: TimerSession = {
      ...mockTimerSession,
      elapsed: 1500, // Equal to duration
      isRunning: false,
    };

    render(<TimerDisplay session={completedSession} />);

    const completionIndicator = screen.getByTestId('completion-indicator');
    expect(completionIndicator).toBeInTheDocument();
  });

  test('displays progress bar correctly', () => {
    render(<TimerDisplay session={mockTimerSession} />);

    const progressBar = screen.getByTestId('progress-bar');
    expect(progressBar).toBeInTheDocument();

    // Progress should be 50% (750/1500)
    expect(progressBar).toHaveStyle('width: 50%');
  });

  test('updates when session prop changes', () => {
    const { rerender } = render(<TimerDisplay session={mockTimerSession} />);

    expect(screen.getByText('12:30')).toBeInTheDocument();

    const updatedSession: TimerSession = {
      ...mockTimerSession,
      elapsed: 900, // 15 minutes elapsed, 10 remaining
    };

    rerender(<TimerDisplay session={updatedSession} />);

    expect(screen.getByText('10:00')).toBeInTheDocument();
  });

  test('handles zero duration gracefully', () => {
    const zeroDurationSession: TimerSession = {
      ...mockTimerSession,
      duration: 0,
      elapsed: 0,
    };

    render(<TimerDisplay session={zeroDurationSession} />);

    expect(screen.getByText('00:00')).toBeInTheDocument();
  });

  test('accessibility attributes are present', () => {
    render(<TimerDisplay session={mockTimerSession} />);

    const timerDisplay = screen.getByTestId('timer-display');
    expect(timerDisplay).toHaveAttribute('aria-label', 'Timer display: 12 minutes 30 seconds remaining');
    expect(timerDisplay).toHaveAttribute('role', 'timer');
  });

  test('displays session count when available', () => {
    const sessionWithCount: TimerSession = {
      ...mockTimerSession,
      sessionCount: 3,
    } as any; // sessionCount is optional

    render(<TimerDisplay session={sessionWithCount} />);

    expect(screen.getByText('Session 3')).toBeInTheDocument();
  });

  test('supports different themes', () => {
    const { container } = render(<TimerDisplay session={mockTimerSession} theme="dark" />);

    expect(container.querySelector('.timer-display.dark')).toBeInTheDocument();
  });

  test('handles missing session gracefully', () => {
    render(<TimerDisplay session={null as any} />);

    const errorMessage = screen.getByText('No timer session');
    expect(errorMessage).toBeInTheDocument();
  });

  test('performance: renders quickly', () => {
    const startTime = performance.now();

    for (let i = 0; i < 100; i++) {
      const { unmount } = render(<TimerDisplay session={mockTimerSession} />);
      unmount();
    }

    const endTime = performance.now();
    const averageTime = (endTime - startTime) / 100;

    // Should render in under 16ms for 60fps
    expect(averageTime).toBeLessThan(16);
  });
});