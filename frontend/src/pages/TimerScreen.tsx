import React, { useEffect } from 'react';
import {
  View,
  StyleSheet,
  Alert,
  BackHandler,
  AppStateStatus,
  AppState,
  Text,
  TouchableOpacity,
} from 'react-native';
import { TimerDisplay } from '../components/TimerDisplay';
import { TimerControls } from '../components/TimerControls';
import { useTimer } from '../hooks/useTimer';
import { TimerScreenState } from '../types';

const TimerScreen: React.FC = () => {
  const {
    session,
    startTimer,
    pauseTimer,
    resetTimer,
    skipTimer,
    loading,
    error,
    isConnected,
    syncStatus,
  } = useTimer();

  // Handle app state changes (background/foreground)
  useEffect(() => {
    const handleAppStateChange = (nextAppState: AppStateStatus) => {
      if (nextAppState === 'active' && session?.isRunning) {
        // App came to foreground, refresh timer state to ensure sync
        // This will be handled by the useTimer hook's WebSocket reconnection
      }
    };

    const subscription = AppState.addEventListener('change', handleAppStateChange);

    return () => {
      subscription?.remove();
    };
  }, [session?.isRunning]);

  // Handle back button press
  useEffect(() => {
    const handleBackPress = () => {
      if (session?.isRunning) {
        Alert.alert(
          'Timer Running',
          'The timer is still running. Are you sure you want to exit?',
          [
            {
              text: 'Cancel',
              style: 'cancel',
            },
            {
              text: 'Exit',
              style: 'destructive',
              onPress: () => BackHandler.exitApp(),
            },
          ]
        );
        return true; // Prevent default back action
      }
      return false; // Allow default back action
    };

    BackHandler.addEventListener('hardwareBackPress', handleBackPress);

    return () => {
      BackHandler.removeEventListener('hardwareBackPress', handleBackPress);
    };
  }, [session?.isRunning]);

  // Handle timer control errors
  const handleTimerError = (error: Error, action: string) => {
    Alert.alert(
      'Timer Error',
      `Failed to ${action}: ${error.message}`,
      [{ text: 'OK' }]
    );
  };

  // Wrapper functions with error handling
  const handleStart = async () => {
    try {
      await startTimer();
    } catch (error) {
      handleTimerError(error as Error, 'start timer');
    }
  };

  const handlePause = async () => {
    try {
      await pauseTimer();
    } catch (error) {
      handleTimerError(error as Error, 'pause timer');
    }
  };

  const handleReset = async () => {
    try {
      await resetTimer();
    } catch (error) {
      handleTimerError(error as Error, 'reset timer');
    }
  };

  const handleSkip = async () => {
    try {
      await skipTimer();
    } catch (error) {
      handleTimerError(error as Error, 'skip timer');
    }
  };

  // Helper functions for sync status display
  const getSyncStatusText = (): string => {
    if (!isConnected) {
      return 'Offline';
    }

    switch (syncStatus) {
      case 'synced':
        return 'Connected';
      case 'syncing':
        return 'Syncing...';
      case 'conflict':
        return 'Conflict';
      case 'offline':
        return 'Offline';
      default:
        return 'Unknown';
    }
  };

  const getSyncStatusEmoji = (): string => {
    switch (syncStatus) {
      case 'synced':
        return '✓';
      case 'syncing':
        return '⟳';
      case 'conflict':
        return '⚠';
      default:
        return '';
    }
  };

  // Determine theme (could be from user configuration in the future)
  const theme = 'light'; // For now, default to light theme

  return (
    <View style={[styles.container, styles[`${theme}Container`]]}>
      {/* Connection and Sync Status Bar */}
      <View style={styles.connectionBar}>
        <View
          style={[
            styles.connectionIndicator,
            isConnected ? styles.connected : styles.disconnected
          ]}
        />
        <View style={styles.statusText}>
          <Text style={[styles.statusText, styles[`${theme}Text`]]}>
            {getSyncStatusText()}
          </Text>
        </View>

        {/* Sync Status Indicator */}
        {syncStatus !== 'offline' && (
          <View style={styles.syncContainer}>
            <View
              style={[
                styles.syncIndicator,
                styles[`${syncStatus}Indicator`]
              ]}
            />
            <Text style={[styles.syncText, styles[`${theme}Text`]]}>
              {getSyncStatusEmoji()}
            </Text>
          </View>
        )}
      </View>

      {/* Sync Status Message */}
      {syncStatus === 'conflict' && (
        <TouchableOpacity
          style={styles.conflictBar}
          onPress={() => {
            Alert.alert(
              'Sync Conflict',
              'Multiple devices tried to control the timer simultaneously. Timer state has been automatically resolved.',
              [{ text: 'OK' }]
            );
          }}
        >
          <Text style={[styles.conflictText, styles[`${theme}ErrorText`]]}>
            ⚠️ Sync conflict detected - Tap for details
          </Text>
        </TouchableOpacity>
      )}

      {/* Error Display */}
      {error && syncStatus !== 'conflict' && (
        <View style={styles.errorContainer}>
          <Text style={[styles.errorText, styles[`${theme}ErrorText`]]}>
            {error}
          </Text>
        </View>
      )}

      {/* Loading State */}
      {loading && !session && (
        <View style={styles.loadingContainer}>
          <Text style={[styles.loadingText, styles[`${theme}Text`]]}>
            Loading timer...
          </Text>
        </View>
      )}

      {/* Main Timer Display */}
      {session && (
        <View style={styles.timerSection}>
          <TimerDisplay
            session={session}
            theme={theme}
            style={styles.timerDisplay}
          />
        </View>
      )}

      {/* Timer Controls */}
      <View style={styles.controlsSection}>
        <TimerControls
          session={session}
          onStart={handleStart}
          onPause={handlePause}
          onReset={handleReset}
          onSkip={handleSkip}
          loading={loading}
          theme={theme}
          disabled={!session}
          style={styles.timerControls}
        />
      </View>

      {/* Session Information */}
      {session && (
        <View style={styles.infoSection}>
          <Text style={[styles.infoText, styles[`${theme}Text`]]}>
            {session.timerType === 'Work' ? 'Focus Time' : 'Break Time'}
          </Text>
          <Text style={[styles.subInfoText, styles[`${theme}SubText`]]}>
            Stay productive! 💪
          </Text>
        </View>
      )}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    padding: 20,
    justifyContent: 'space-between',
    minHeight: '100%',
  },
  lightContainer: {
    backgroundColor: '#f5f5f5',
  },
  darkContainer: {
    backgroundColor: '#1a1a1a',
  },
  connectionBar: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'flex-end',
    paddingHorizontal: 10,
    paddingVertical: 5,
  },
  connectionIndicator: {
    width: 8,
    height: 8,
    borderRadius: 4,
    marginRight: 8,
  },
  connected: {
    backgroundColor: '#4CAF50',
  },
  disconnected: {
    backgroundColor: '#F44336',
  },
  statusText: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  lightText: {
    color: '#1a1a1a',
  },
  darkText: {
    color: '#ffffff',
  },
  errorContainer: {
    backgroundColor: '#ffebee',
    padding: 12,
    borderRadius: 8,
    marginVertical: 10,
    borderLeftWidth: 4,
    borderLeftColor: '#f44336',
  },
  errorText: {
    color: '#c62828',
    fontSize: 14,
    textAlign: 'center',
  },
  lightErrorText: {
    color: '#c62828',
  },
  darkErrorText: {
    color: '#ef5350',
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  loadingText: {
    fontSize: 16,
    fontStyle: 'italic',
  },
  timerSection: {
    flex: 2,
    justifyContent: 'center',
    alignItems: 'center',
    marginVertical: 20,
  },
  timerDisplay: {
    width: '100%',
    maxWidth: 400,
  },
  controlsSection: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    marginVertical: 20,
  },
  timerControls: {
    width: '100%',
    maxWidth: 400,
  },
  infoSection: {
    flex: 0,
    alignItems: 'center',
    paddingVertical: 20,
  },
  infoText: {
    fontSize: 18,
    fontWeight: '600',
    marginBottom: 5,
  },
  subInfoText: {
    fontSize: 14,
    opacity: 0.7,
  },
  lightSubText: {
    color: '#666666',
  },
  darkSubText: {
    color: '#cccccc',
  },
  // Sync status styles
  syncContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    marginLeft: 'auto',
    paddingLeft: 10,
  },
  syncIndicator: {
    width: 6,
    height: 6,
    borderRadius: 3,
    marginRight: 4,
  },
  syncedIndicator: {
    backgroundColor: '#4CAF50',
  },
  syncingIndicator: {
    backgroundColor: '#2196F3',
  },
  conflictIndicator: {
    backgroundColor: '#FF9800',
  },
  syncText: {
    fontSize: 12,
    opacity: 0.7,
  },
  conflictBar: {
    backgroundColor: '#FFF3CD',
    paddingVertical: 8,
    paddingHorizontal: 12,
    marginVertical: 5,
    borderRadius: 4,
    borderLeftWidth: 3,
    borderLeftColor: '#FF9800',
  },
  conflictText: {
    fontSize: 12,
    fontWeight: '500',
    textAlign: 'center',
  },
});

export { TimerScreen };
export default TimerScreen;