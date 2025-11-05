import React, { useMemo } from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { TimerDisplayProps, TimerSession } from '../../types';

const TimerDisplay: React.FC<TimerDisplayProps> = ({
  session,
  theme = 'light',
  className = '',
  ...props
}) => {
  // Format time as MM:SS with leading zeros
  const formatTime = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };

  // Calculate remaining time and progress
  const { displayTime, progress, isComplete } = useMemo(() => {
    if (!session) {
      return {
        displayTime: '00:00',
        progress: 0,
        isComplete: false,
      };
    }

    const remaining = Math.max(0, session.duration - session.elapsed);
    const displayTime = formatTime(remaining);
    const progress = session.duration > 0 ? (session.elapsed / session.duration) * 100 : 0;
    const isComplete = session.elapsed >= session.duration;

    return { displayTime, progress, isComplete };
  }, [session]);

  // Get session type display text
  const getSessionTypeText = (timerType: string): string => {
    switch (timerType) {
      case 'Work':
        return 'Work Session';
      case 'ShortBreak':
        return 'Short Break';
      case 'LongBreak':
        return 'Long Break';
      default:
        return 'Timer';
    }
  };

  // Theme-based styles
  const themedStyles = useMemo(() => {
    const colors = theme === 'dark'
      ? {
          background: '#1a1a1a',
          surface: '#2a2a2a',
          text: '#ffffff',
          primary: '#bb86fc',
          secondary: '#03dac6',
          accent: '#cf6679',
        }
      : {
          background: '#ffffff',
          surface: '#f5f5f5',
          text: '#1a1a1a',
          primary: '#6200ee',
          secondary: '#018786',
          accent: '#b00020',
        };

    return {
      container: {
        backgroundColor: colors.background,
      },
      timerText: {
        color: colors.text,
      },
      sessionType: {
        color: colors.secondary,
      },
      progressContainer: {
        backgroundColor: colors.surface,
      },
      progressBar: {
        backgroundColor: colors.primary,
      },
      runningIndicator: {
        backgroundColor: session?.isRunning ? colors.secondary : colors.surface,
      },
      completionIndicator: {
        color: colors.accent,
      },
    };
  }, [theme, session?.isRunning]);

  if (!session) {
    return (
      <View
        style={[styles.container, themedStyles.container, className]}
        testID="timer-display"
        accessibility-label="No timer session"
        accessibility-role="timer"
        {...props}
      >
        <Text style={[styles.errorText, themedStyles.timerText]}>
          No timer session
        </Text>
      </View>
    );
  }

  const ariaLabel = `Timer display: ${displayTime} remaining - ${getSessionTypeText(session.timerType)} - ${session.isRunning ? 'running' : 'paused'}`;

  return (
    <View
      style={[styles.container, themedStyles.container, className]}
      testID="timer-display"
      accessibility-label={ariaLabel}
      accessibility-role="timer"
      aria-live="polite"
      {...props}
    >
      {/* Session Type */}
      <Text
        style={[styles.sessionType, themedStyles.sessionType]}
        testID="session-type"
      >
        {getSessionTypeText(session.timerType)}
      </Text>

      {/* Main Timer Display */}
      <Text
        style={[styles.timerText, themedStyles.timerText]}
        testID="timer-time"
      >
        {displayTime}
      </Text>

      {/* Session Count (if available) */}
      {session.sessionCount && (
        <Text
          style={[styles.sessionCount, themedStyles.sessionType]}
          testID="session-count"
        >
          Session {session.sessionCount}
        </Text>
      )}

      {/* Progress Bar */}
      <View
        style={[styles.progressContainer, themedStyles.progressContainer]}
        testID="progress-container"
      >
        <View
          style={[
            styles.progressBar,
            themedStyles.progressBar,
            { width: `${Math.min(progress, 100)}%` }
          ]}
          testID="progress-bar"
        />
      </View>

      {/* Running Indicator */}
      <View
        style={[
          styles.runningIndicator,
          themedStyles.runningIndicator,
          session.isRunning ? styles.running : styles.paused
        ]}
        testID="running-indicator"
      />

      {/* Completion Indicator */}
      {isComplete && (
        <Text
          style={[styles.completionIndicator, themedStyles.completionIndicator]}
          testID="completion-indicator"
        >
          ✓ Complete!
        </Text>
      )}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    alignItems: 'center',
    justifyContent: 'center',
    padding: 20,
    borderRadius: 12,
    minWidth: 300,
    minHeight: 200,
  },
  timerText: {
    fontSize: 72,
    fontWeight: 'bold',
    fontFamily: 'monospace',
    marginVertical: 10,
  },
  sessionType: {
    fontSize: 18,
    fontWeight: '600',
    marginBottom: 10,
  },
  sessionCount: {
    fontSize: 14,
    opacity: 0.7,
    marginTop: 5,
  },
  progressContainer: {
    width: '100%',
    height: 8,
    borderRadius: 4,
    overflow: 'hidden',
    marginTop: 20,
  },
  progressBar: {
    height: '100%',
    borderRadius: 4,
    transition: 'width 0.3s ease-in-out',
  },
  runningIndicator: {
    width: 12,
    height: 12,
    borderRadius: 6,
    marginTop: 15,
  },
  running: {
    backgroundColor: '#03dac6',
  },
  paused: {
    backgroundColor: '#bbbbbb',
  },
  completionIndicator: {
    fontSize: 24,
    fontWeight: 'bold',
    marginTop: 10,
  },
  errorText: {
    fontSize: 16,
    opacity: 0.7,
  },
});

export { TimerDisplay };
export default TimerDisplay;