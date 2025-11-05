import React, { useCallback, useEffect } from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
  AccessibilityInfo,
} from 'react-native';
import { TimerControlsProps, TimerSession } from '../../types';

const TimerControls: React.FC<TimerControlsProps> = ({
  session,
  onStart,
  onPause,
  onReset,
  onSkip,
  loading = false,
  theme = 'light',
  disabled = false,
  ...props
}) => {
  // Handle keyboard shortcuts
  const handleKeyPress = useCallback((event: KeyboardEvent) => {
    if (disabled || loading) return;

    switch (event.key.toLowerCase()) {
      case ' ':
      case 'space':
        event.preventDefault();
        if (session?.isRunning) {
          onPause();
        } else {
          onStart();
        }
        break;
      case 'r':
        event.preventDefault();
        onReset();
        break;
      case 's':
        event.preventDefault();
        onSkip();
        break;
      case 'p':
        event.preventDefault();
        if (session?.isRunning) {
          onPause();
        } else {
          onStart();
        }
        break;
    }
  }, [session?.isRunning, onStart, onPause, onReset, onSkip, disabled, loading]);

  // Set up keyboard event listeners
  useEffect(() => {
    window.addEventListener('keydown', handleKeyPress);
    return () => {
      window.removeEventListener('keydown', handleKeyPress);
    };
  }, [handleKeyPress]);

  // Prevent multiple rapid clicks
  const createClickHandler = (handler: () => void) => {
    return () => {
      if (loading || disabled) return;
      handler();
    };
  };

  // Theme-based styles
  const themedStyles = React.useMemo(() => {
    const colors = theme === 'dark'
      ? {
          background: '#1a1a1a',
          surface: '#2a2a2a',
          text: '#ffffff',
          primary: '#bb86fc',
          secondary: '#03dac6',
          accent: '#cf6679',
          disabled: '#666666',
        }
      : {
          background: '#ffffff',
          surface: '#f5f5f5',
          text: '#1a1a1a',
          primary: '#6200ee',
          secondary: '#018786',
          accent: '#b00020',
          disabled: '#cccccc',
        };

    return {
      container: {
        backgroundColor: colors.background,
      },
      button: {
        backgroundColor: colors.surface,
        borderColor: colors.primary,
      },
      buttonText: {
        color: colors.text,
      },
      primaryButton: {
        backgroundColor: colors.primary,
      },
      primaryButtonText: {
        color: '#ffffff',
      },
      disabledButton: {
        backgroundColor: colors.disabled,
        borderColor: colors.disabled,
      },
      disabledButtonText: {
        color: '#ffffff',
        opacity: 0.5,
      },
      loadingText: {
        color: colors.text,
      },
    };
  }, [theme]);

  const isRunning = session?.isRunning || false;

  if (!session) {
    return (
      <View
        style={[styles.container, themedStyles.container]}
        role="group"
        aria-label="Timer controls"
        testID="timer-controls"
        {...props}
      >
        <Text style={[styles.errorText, themedStyles.buttonText]}>
          No timer session available
        </Text>
      </View>
    );
  }

  return (
    <View
      style={[styles.container, themedStyles.container]}
      role="group"
      aria-label="Timer controls"
      testID="timer-controls"
      {...props}
    >
      {loading ? (
        <Text
          style={[styles.loadingText, themedStyles.loadingText]}
          testID="loading-indicator"
        >
          Loading...
        </Text>
      ) : (
        <>
          {/* Start/Pause Button (Primary) */}
          <TouchableOpacity
            style={[
              styles.button,
              styles.primaryButton,
              isRunning ? styles.pauseButton : styles.startButton,
              (disabled || loading) && styles.disabledButton,
              themedStyles.button,
              themedStyles.primaryButton,
              (disabled || loading) && themedStyles.disabledButton,
            ]}
            onPress={createClickHandler(isRunning ? onPause : onStart)}
            disabled={disabled || loading}
            testID={isRunning ? "pause-button" : "start-button"}
            accessibility-label={isRunning ? "Pause timer" : "Start timer"}
            accessibility-role="button"
            accessibility-state={{ disabled: disabled || loading }}
          >
            <Text
              style={[
                styles.buttonText,
                styles.primaryButtonText,
                (disabled || loading) && styles.disabledButtonText,
                themedStyles.primaryButtonText,
                (disabled || loading) && themedStyles.disabledButtonText,
              ]}
            >
              {isRunning ? 'Pause' : 'Start'}
            </Text>
          </TouchableOpacity>

          {/* Control Buttons Row */}
          <View style={styles.buttonRow}>
            {/* Reset Button */}
            <TouchableOpacity
              style={[
                styles.button,
                styles.secondaryButton,
                (disabled || loading) && styles.disabledButton,
                themedStyles.button,
                (disabled || loading) && themedStyles.disabledButton,
              ]}
              onPress={createClickHandler(onReset)}
              disabled={disabled || loading}
              testID="reset-button"
              accessibility-label="Reset timer"
              accessibility-role="button"
              accessibility-state={{ disabled: disabled || loading }}
            >
              <Text
                style={[
                  styles.buttonText,
                  styles.secondaryButtonText,
                  (disabled || loading) && styles.disabledButtonText,
                  themedStyles.buttonText,
                  (disabled || loading) && themedStyles.disabledButtonText,
                ]}
              >
                Reset
              </Text>
            </TouchableOpacity>

            {/* Skip Button */}
            <TouchableOpacity
              style={[
                styles.button,
                styles.secondaryButton,
                (disabled || loading) && styles.disabledButton,
                themedStyles.button,
                (disabled || loading) && themedStyles.disabledButton,
              ]}
              onPress={createClickHandler(onSkip)}
              disabled={disabled || loading}
              testID="skip-button"
              accessibility-label="Skip to next session"
              accessibility-role="button"
              accessibility-state={{ disabled: disabled || loading }}
            >
              <Text
                style={[
                  styles.buttonText,
                  styles.secondaryButtonText,
                  (disabled || loading) && styles.disabledButtonText,
                  themedStyles.buttonText,
                  (disabled || loading) && themedStyles.disabledButtonText,
                ]}
              >
                Skip
              </Text>
            </TouchableOpacity>
          </View>

          {/* Keyboard Shortcuts Help */}
          <View style={styles.shortcutsHelp}>
            <Text style={[styles.shortcutsText, themedStyles.buttonText]}>
              Keyboard: Space (toggle) • R (reset) • S (skip)
            </Text>
          </View>
        </>
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
  },
  button: {
    paddingVertical: 16,
    paddingHorizontal: 32,
    borderRadius: 8,
    borderWidth: 2,
    alignItems: 'center',
    justifyContent: 'center',
    minWidth: 120,
    marginVertical: 8,
  },
  primaryButton: {
    minWidth: 200,
    paddingVertical: 20,
  },
  startButton: {
    // Additional styling for start button
  },
  pauseButton: {
    // Additional styling for pause button
  },
  secondaryButton: {
    minWidth: 100,
    paddingVertical: 12,
    marginHorizontal: 8,
  },
  buttonText: {
    fontSize: 16,
    fontWeight: '600',
    textAlign: 'center',
  },
  primaryButtonText: {
    fontSize: 18,
    fontWeight: 'bold',
  },
  secondaryButtonText: {
    fontSize: 14,
  },
  disabledButton: {
    opacity: 0.5,
    // Remove button feedback when disabled
    transform: [{ scale: 1 }],
  },
  disabledButtonText: {
    opacity: 0.5,
  },
  buttonRow: {
    flexDirection: 'row',
    justifyContent: 'center',
    alignItems: 'center',
    marginTop: 16,
  },
  shortcutsHelp: {
    marginTop: 20,
    alignItems: 'center',
  },
  shortcutsText: {
    fontSize: 12,
    opacity: 0.6,
    textAlign: 'center',
  },
  loadingText: {
    fontSize: 16,
    fontStyle: 'italic',
    textAlign: 'center',
  },
  errorText: {
    fontSize: 16,
    fontStyle: 'italic',
    textAlign: 'center',
    opacity: 0.7,
  },
});

export { TimerControls };
export default TimerControls;