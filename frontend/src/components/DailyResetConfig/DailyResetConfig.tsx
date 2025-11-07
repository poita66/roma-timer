//! Daily Reset Configuration Component
//!
//! React component for configuring daily session reset settings including
//! timezone selection, reset time options, and session count management.

import React, { useState, useCallback, useEffect } from 'react';
import {
  View,
  Text,
  Switch,
  TouchableOpacity,
  StyleSheet,
  Alert,
  ScrollView,
  Platform,
} from 'react-native';
import { TimezonePicker, useCurrentTimezone } from '../TimezonePicker/TimezonePicker';

// Types for daily reset configuration
export interface DailyResetConfig {
  enabled: boolean;
  timezone: string;
  resetTimeType: 'midnight' | 'hour' | 'custom';
  resetHour?: number; // 0-23 for hourly resets
  customTime?: string; // HH:MM format for custom times
}

export interface DailyResetStatus {
  nextResetTimeUtc?: number;
  resetDueToday?: boolean;
  currentSessionCount: number;
  manualSessionOverride?: number;
  lastResetTimeUtc?: number;
}

interface DailyResetConfigProps {
  config: DailyResetConfig;
  status?: DailyResetStatus;
  onConfigChange: (config: DailyResetConfig) => void;
  onSessionReset?: () => void;
  style?: any;
}

interface ResetTimeOptionProps {
  title: string;
  description: string;
  value: DailyResetConfig['resetTimeType'];
  selected: boolean;
  onSelect: (value: DailyResetConfig['resetTimeType']) => void;
}

const ResetTimeOption: React.FC<ResetTimeOptionProps> = ({
  title,
  description,
  value,
  selected,
  onSelect
}) => {
  return (
    <TouchableOpacity
      style={[
        styles.resetTimeOption,
        selected && styles.selectedResetTimeOption
      ]}
      onPress={() => onSelect(value)}
    >
      <View style={styles.resetTimeOptionContent}>
        <View style={styles.resetTimeOptionInfo}>
          <Text style={[
            styles.resetTimeOptionTitle,
            selected && styles.selectedResetTimeOptionTitle
          ]}>
            {title}
          </Text>
          <Text style={styles.resetTimeOptionDescription}>
            {description}
          </Text>
        </View>
        <View style={[
          styles.resetTimeOptionRadio,
          selected && styles.selectedResetTimeOptionRadio
        ]}>
          {selected && <View style={styles.resetTimeOptionRadioInner} />}
        </View>
      </View>
    </TouchableOpacity>
  );
};

interface SessionCountCardProps {
  currentCount: number;
  manualOverride?: number;
  onManualOverride: () => void;
  onReset: () => void;
}

const SessionCountCard: React.FC<SessionCountCardProps> = ({
  currentCount,
  manualOverride,
  onManualOverride,
  onReset
}) => {
  return (
    <View style={styles.sessionCountCard}>
      <Text style={styles.sessionCountTitle}>Today's Session Count</Text>

      <View style={styles.sessionCountDisplay}>
        <Text style={styles.sessionCountNumber}>
          {manualOverride !== undefined ? manualOverride : currentCount}
        </Text>
        {manualOverride !== undefined && (
          <Text style={styles.overrideIndicator}>Manual Override</Text>
        )}
      </View>

      <View style={styles.sessionCountActions}>
        <TouchableOpacity
          style={styles.actionButton}
          onPress={onManualOverride}
        >
          <Text style={styles.actionButtonText}>Override</Text>
        </TouchableOpacity>

        <TouchableOpacity
          style={[styles.actionButton, styles.resetButton]}
          onPress={onReset}
        >
          <Text style={[styles.actionButtonText, styles.resetButtonText]}>
            Reset
          </Text>
        </TouchableOpacity>
      </View>
    </View>
  );
};

export const DailyResetConfig: React.FC<DailyResetConfigProps> = ({
  config,
  status,
  onConfigChange,
  onSessionReset,
  style
}) => {
  const [localTimezone, setLocalTimezone] = useState(config.timezone);
  const [isConfigExpanded, setIsConfigExpanded] = useState(false);
  const currentTimezone = useCurrentTimezone();

  // Update local timezone when config changes
  useEffect(() => {
    setLocalTimezone(config.timezone);
  }, [config.timezone]);

  // Handle timezone change
  const handleTimezoneChange = useCallback((timezone: string) => {
    setLocalTimezone(timezone);
    onConfigChange({ ...config, timezone });
  }, [config, onConfigChange]);

  // Handle daily reset toggle
  const handleToggleReset = useCallback((enabled: boolean) => {
    onConfigChange({ ...config, enabled });
  }, [config, onConfigChange]);

  // Handle reset time type selection
  const handleResetTimeTypeSelect = useCallback((resetTimeType: DailyResetConfig['resetTimeType']) => {
    onConfigChange({ ...config, resetTimeType });
  }, [config, onConfigChange]);

  // Handle manual session override
  const handleManualOverride = useCallback(() => {
    Alert.prompt(
      'Override Session Count',
      'Enter the number of sessions completed today:',
      [
        { text: 'Cancel', style: 'cancel' },
        { text: 'Override', onPress: (value) => {
          const count = parseInt(value || '0', 10);
          if (!isNaN(count) && count >= 0 && count <= 100) {
            // In a real implementation, this would call the WebSocket service
            console.log('Setting manual override:', count);
          } else {
            Alert.alert('Invalid Input', 'Please enter a number between 0 and 100');
          }
        }}
      ],
      'plain-text'
    );
  }, []);

  // Handle session reset
  const handleSessionReset = useCallback(() => {
    Alert.alert(
      'Reset Session Count',
      'This will reset today\'s session count to 0. Are you sure?',
      [
        { text: 'Cancel', style: 'cancel' },
        { text: 'Reset', onPress: () => {
          if (onSessionReset) {
            onSessionReset();
          }
        }}
      ],
      'destructive'
    );
  }, [onSessionReset]);

  // Format next reset time
  const formatNextResetTime = useCallback(() => {
    if (!status?.nextResetTimeUtc) return null;

    const nextResetDate = new Date(status.nextResetTimeUtc * 1000);
    return nextResetDate.toLocaleString();
  }, [status?.nextResetTimeUtc]);

  return (
    <ScrollView style={[styles.container, style]}>
      {/* Enable/Disable Daily Reset */}
      <View style={styles.section}>
        <View style={styles.sectionHeader}>
          <Text style={styles.sectionTitle}>Daily Session Reset</Text>
          <Switch
            value={config.enabled}
            onValueChange={handleToggleReset}
            trackColor={{ false: '#ccc', true: '#4CAF50' }}
          />
        </View>
        <Text style={styles.sectionDescription}>
          Automatically reset session count at a specific time each day
        </Text>
      </View>

      {config.enabled && (
        <>
          {/* Timezone Selection */}
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>Timezone</Text>
            <TimezonePicker
              selectedTimezone={localTimezone}
              onTimezoneChange={handleTimezoneChange}
              placeholder="Select your timezone"
            />
            <Text style={styles.sectionDescription}>
              All reset times will be calculated in your local timezone
            </Text>
          </View>

          {/* Reset Time Configuration */}
          <View style={styles.section}>
            <View style={styles.sectionHeader}>
              <Text style={styles.sectionTitle}>Reset Time</Text>
              <TouchableOpacity
                onPress={() => setIsConfigExpanded(!isConfigExpanded)}
              >
                <Text style={styles.expandButton}>
                  {isConfigExpanded ? '▼' : '▶'}
                </Text>
              </TouchableOpacity>
            </View>

            <ResetTimeOption
              title="Midnight"
              description="Reset at 12:00 AM each day"
              value="midnight"
              selected={config.resetTimeType === 'midnight'}
              onSelect={handleResetTimeTypeSelect}
            />

            <ResetTimeOption
              title="Custom Hour"
              description="Choose a specific hour for daily reset"
              value="hour"
              selected={config.resetTimeType === 'hour'}
              onSelect={handleResetTimeTypeSelect}
            />

            <ResetTimeOption
              title="Custom Time"
              description="Set exact time (HH:MM) for daily reset"
              value="custom"
              selected={config.resetTimeType === 'custom'}
              onSelect={handleResetTimeTypeSelect}
            />
          </View>

          {/* Session Count Management */}
          <View style={styles.section}>
            <SessionCountCard
              currentCount={status?.currentSessionCount || 0}
              manualOverride={status?.manualSessionOverride}
              onManualOverride={handleManualOverride}
              onReset={handleSessionReset}
            />
          </View>

          {/* Status Information */}
          {status && (
            <View style={styles.section}>
              <Text style={styles.sectionTitle}>Status Information</Text>

              {formatNextResetTime() && (
                <View style={styles.statusItem}>
                  <Text style={styles.statusLabel}>Next Reset:</Text>
                  <Text style={styles.statusValue}>{formatNextResetTime()}</Text>
                </View>
              )}

              <View style={styles.statusItem}>
                <Text style={styles.statusLabel}>Reset Due Today:</Text>
                <Text style={[
                  styles.statusValue,
                  status.resetDueToday ? styles.statusDue : styles.statusNotDue
                ]}>
                  {status.resetDueToday ? 'Yes' : 'No'}
                </Text>
              </View>

              {status.lastResetTimeUtc && (
                <View style={styles.statusItem}>
                  <Text style={styles.statusLabel}>Last Reset:</Text>
                  <Text style={styles.statusValue}>
                    {new Date(status.lastResetTimeUtc * 1000).toLocaleString()}
                  </Text>
                </View>
              )}
            </View>
          )}
        </>
      )}
    </ScrollView>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f8f9fa',
  },
  section: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginVertical: 8,
    borderRadius: 12,
    padding: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.1,
    shadowRadius: 2,
    elevation: 2,
  },
  sectionHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 8,
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#333',
  },
  sectionDescription: {
    fontSize: 14,
    color: '#666',
    lineHeight: 20,
    marginTop: 4,
  },
  expandButton: {
    fontSize: 16,
    color: '#666',
    padding: 4,
  },
  resetTimeOption: {
    borderWidth: 1,
    borderColor: '#e0e0e0',
    borderRadius: 8,
    padding: 16,
    marginVertical: 4,
  },
  selectedResetTimeOption: {
    borderColor: '#2196F3',
    backgroundColor: '#f3f8ff',
  },
  resetTimeOptionContent: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  resetTimeOptionInfo: {
    flex: 1,
  },
  resetTimeOptionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
  },
  selectedResetTimeOptionTitle: {
    color: '#1976D2',
  },
  resetTimeOptionDescription: {
    fontSize: 14,
    color: '#666',
    marginTop: 4,
  },
  resetTimeOptionRadio: {
    width: 20,
    height: 20,
    borderRadius: 10,
    borderWidth: 2,
    borderColor: '#ccc',
    justifyContent: 'center',
    alignItems: 'center',
  },
  selectedResetTimeOptionRadio: {
    borderColor: '#2196F3',
  },
  resetTimeOptionRadioInner: {
    width: 10,
    height: 10,
    borderRadius: 5,
    backgroundColor: '#2196F3',
  },
  sessionCountCard: {
    padding: 16,
    alignItems: 'center',
  },
  sessionCountTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    marginBottom: 16,
  },
  sessionCountDisplay: {
    alignItems: 'center',
    marginBottom: 16,
  },
  sessionCountNumber: {
    fontSize: 48,
    fontWeight: 'bold',
    color: '#2196F3',
  },
  overrideIndicator: {
    fontSize: 12,
    color: '#FF9800',
    marginTop: 4,
    fontWeight: '500',
  },
  sessionCountActions: {
    flexDirection: 'row',
    justifyContent: 'space-around',
    width: '100%',
  },
  actionButton: {
    paddingHorizontal: 20,
    paddingVertical: 10,
    borderRadius: 8,
    backgroundColor: '#2196F3',
    minWidth: 100,
  },
  resetButton: {
    backgroundColor: '#f44336',
  },
  actionButtonText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '600',
    textAlign: 'center',
  },
  resetButtonText: {
    color: '#fff',
  },
  statusItem: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 8,
    borderBottomWidth: 1,
    borderBottomColor: '#f0f0f0',
  },
  statusLabel: {
    fontSize: 14,
    color: '#666',
  },
  statusValue: {
    fontSize: 14,
    fontWeight: '500',
    color: '#333',
  },
  statusDue: {
    color: '#4CAF50',
  },
  statusNotDue: {
    color: '#FF9800',
  },
});

export default DailyResetConfig;