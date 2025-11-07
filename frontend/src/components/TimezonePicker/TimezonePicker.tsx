//! Timezone Picker Component
//!
//! A React component for selecting user timezone with search functionality
//! and common timezone shortcuts.

import React, { useState, useMemo, useCallback } from 'react';
import {
  View,
  Text,
  TextInput,
  FlatList,
  TouchableOpacity,
  StyleSheet,
  Dimensions,
  Platform,
} from 'react-native';

// Common timezone list with UTC offsets
const COMMON_TIMEZONES = [
  { id: 'UTC', name: 'UTC (Coordinated Universal Time)', offset: 0 },
  { id: 'America/New_York', name: 'New York (Eastern Time)', offset: -5 },
  { id: 'America/Chicago', name: 'Chicago (Central Time)', offset: -6 },
  { id: 'America/Denver', name: 'Denver (Mountain Time)', offset: -7 },
  { id: 'America/Los_Angeles', name: 'Los Angeles (Pacific Time)', offset: -8 },
  { id: 'America/Phoenix', name: 'Phoenix (Arizona)', offset: -7 },
  { id: 'America/Anchorage', name: 'Anchorage (Alaska)', offset: -9 },
  { id: 'Pacific/Honolulu', name: 'Honolulu (Hawaii)', offset: -10 },
  { id: 'America/Toronto', name: 'Toronto (Eastern)', offset: -5 },
  { id: 'America/Vancouver', name: 'Vancouver (Pacific)', offset: -8 },
  { id: 'Europe/London', name: 'London (Greenwich Mean Time)', offset: 0 },
  { id: 'Europe/Paris', name: 'Paris (Central European Time)', offset: 1 },
  { id: 'Europe/Berlin', name: 'Berlin (Central European Time)', offset: 1 },
  { id: 'Europe/Moscow', name: 'Moscow (Moscow Time)', offset: 3 },
  { id: 'Asia/Dubai', name: 'Dubai (Gulf Standard Time)', offset: 4 },
  { id: 'Asia/Kolkata', name: 'Mumbai (India Standard Time)', offset: 5.5 },
  { id: 'Asia/Shanghai', name: 'Shanghai (China Standard Time)', offset: 8 },
  { id: 'Asia/Tokyo', name: 'Tokyo (Japan Standard Time)', offset: 9 },
  { id: 'Asia/Seoul', name: 'Seoul (Korea Standard Time)', offset: 9 },
  { id: 'Asia/Singapore', name: 'Singapore (Singapore Time)', offset: 8 },
  { id: 'Australia/Sydney', name: 'Sydney (Australian Eastern Time)', offset: 10 },
  { id: 'Pacific/Auckland', name: 'Auckland (New Zealand Time)', offset: 12 },
];

interface Timezone {
  id: string;
  name: string;
  offset: number; // UTC offset in hours
}

interface TimezonePickerProps {
  selectedTimezone?: string;
  onTimezoneChange: (timezone: string) => void;
  placeholder?: string;
  style?: any;
}

interface TimezoneItemProps {
  timezone: Timezone;
  isSelected: boolean;
  onSelect: (timezone: string) => void;
}

const TimezoneItem: React.FC<TimezoneItemProps> = ({ timezone, isSelected, onSelect }) => {
  return (
    <TouchableOpacity
      style={[
        styles.timezoneItem,
        isSelected && styles.selectedTimezoneItem
      ]}
      onPress={() => onSelect(timezone.id)}
    >
      <View style={styles.timezoneInfo}>
        <Text style={[
          styles.timezoneName,
          isSelected && styles.selectedTimezoneName
        ]}>
          {timezone.name}
        </Text>
        <Text style={[
          styles.timezoneOffset,
          isSelected && styles.selectedTimezoneOffset
        ]}>
          UTC{timezone.offset >= 0 ? '+' : ''}{timezone.offset}
        </Text>
      </View>
      {isSelected && (
        <View style={styles.selectedIndicator}>
          <Text style={styles.selectedIndicatorText}>✓</Text>
        </View>
      )}
    </TouchableOpacity>
  );
};

export const TimezonePicker: React.FC<TimezonePickerProps> = ({
  selectedTimezone,
  onTimezoneChange,
  placeholder = 'Select timezone',
  style
}) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [isDropdownOpen, setIsDropdownOpen] = useState(false);

  // Filter timezones based on search query
  const filteredTimezones = useMemo(() => {
    if (!searchQuery.trim()) {
      return COMMON_TIMEZONES;
    }

    const query = searchQuery.toLowerCase();
    return COMMON_TIMEZONES.filter(timezone =>
      timezone.name.toLowerCase().includes(query) ||
      timezone.id.toLowerCase().includes(query) ||
      timezone.offset.toString().includes(query)
    );
  }, [searchQuery]);

  // Find selected timezone object
  const selectedTimezoneObj = useMemo(() => {
    return COMMON_TIMEZONES.find(tz => tz.id === selectedTimezone);
  }, [selectedTimezone]);

  const handleTimezoneSelect = useCallback((timezoneId: string) => {
    onTimezoneChange(timezoneId);
    setIsDropdownOpen(false);
    setSearchQuery('');
  }, [onTimezoneChange]);

  const renderTimezoneItem = ({ item }: { item: Timezone }) => (
    <TimezoneItem
      timezone={item}
      isSelected={item.id === selectedTimezone}
      onSelect={handleTimezoneSelect}
    />
  );

  return (
    <View style={[styles.container, style]}>
      <TouchableOpacity
        style={styles.trigger}
        onPress={() => setIsDropdownOpen(!isDropdownOpen)}
      >
        <Text style={styles.triggerText}>
          {selectedTimezoneObj
            ? `${selectedTimezoneObj.name} (UTC${selectedTimezoneObj.offset >= 0 ? '+' : ''}${selectedTimezoneObj.offset})`
            : placeholder
          }
        </Text>
        <Text style={styles.dropdownIcon}>
          {isDropdownOpen ? '▲' : '▼'}
        </Text>
      </TouchableOpacity>

      {isDropdownOpen && (
        <View style={styles.dropdown}>
          <View style={styles.searchContainer}>
            <TextInput
              style={styles.searchInput}
              placeholder="Search timezone..."
              value={searchQuery}
              onChangeText={setSearchQuery}
              autoFocus
              placeholderTextColor="#999"
            />
          </View>

          <FlatList
            data={filteredTimezones}
            renderItem={renderTimezoneItem}
            keyExtractor={(item) => item.id}
            style={styles.timezoneList}
            showsVerticalScrollIndicator={false}
            keyboardShouldPersistTaps="handled"
            nestedScrollEnabled
            ListEmptyComponent={
              <View style={styles.emptyState}>
                <Text style={styles.emptyStateText}>No timezones found</Text>
              </View>
            }
          />
        </View>
      )}
    </View>
  );
};

// Hook for timezone detection and user's current timezone
export const useCurrentTimezone = () => {
  const [currentTimezone, setCurrentTimezone] = useState<string>('UTC');

  React.useEffect(() => {
    // Get user's timezone from browser/OS
    const detectTimezone = () => {
      try {
        // In a real implementation, you might use a library like 'moment-timezone'
        // or make an API call to detect the user's timezone
        const detected = Intl.DateTimeFormat().resolvedOptions().timeZone;

        // Map common IANA timezones to our supported list
        const timezoneMap: { [key: string]: string } = {
          'America/New_York': 'America/New_York',
          'America/Chicago': 'America/Chicago',
          'America/Denver': 'America/Denver',
          'America/Los_Angeles': 'America/Los_Angeles',
          'America/Phoenix': 'America/Phoenix',
          'America/Toronto': 'America/Toronto',
          'America/Vancouver': 'America/Vancouver',
          'Europe/London': 'Europe/London',
          'Europe/Paris': 'Europe/Paris',
          'Europe/Berlin': 'Europe/Berlin',
          'Europe/Moscow': 'Europe/Moscow',
          'Asia/Dubai': 'Asia/Dubai',
          'Asia/Kolkata': 'Asia/Kolkata',
          'Asia/Shanghai': 'Asia/Shanghai',
          'Asia/Tokyo': 'Asia/Tokyo',
          'Asia/Seoul': 'Asia/Seoul',
          'Asia/Singapore': 'Asia/Singapore',
          'Australia/Sydney': 'Australia/Sydney',
          'Pacific/Auckland': 'Pacific/Auckland',
        };

        return timezoneMap[detected] || 'UTC';
      } catch (error) {
        console.warn('Failed to detect timezone:', error);
        return 'UTC';
      }
    };

    const timezone = detectTimezone();
    setCurrentTimezone(timezone);
  }, []);

  return currentTimezone;
};

const { width, height } = Dimensions.get('window');

const styles = StyleSheet.create({
  container: {
    width: '100%',
    zIndex: 1000,
  },
  trigger: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    backgroundColor: '#fff',
    borderWidth: 1,
    borderColor: '#ddd',
    borderRadius: 8,
    paddingHorizontal: 16,
    paddingVertical: 12,
    minHeight: 48,
  },
  triggerText: {
    fontSize: 16,
    color: '#333',
    flex: 1,
  },
  dropdownIcon: {
    fontSize: 12,
    color: '#666',
    marginLeft: 8,
  },
  dropdown: {
    position: 'absolute',
    top: '100%',
    left: 0,
    right: 0,
    backgroundColor: '#fff',
    borderWidth: 1,
    borderColor: '#ddd',
    borderTopWidth: 0,
    borderRadius: 8,
    borderBottomLeftRadius: 8,
    borderBottomRightRadius: 8,
    maxHeight: height * 0.5,
    elevation: 5,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.1,
    shadowRadius: 4,
    zIndex: 1001,
  },
  searchContainer: {
    paddingHorizontal: 16,
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#eee',
  },
  searchInput: {
    backgroundColor: '#f8f9fa',
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 8,
    fontSize: 16,
    borderWidth: 1,
    borderColor: '#e9ecef',
  },
  timezoneList: {
    flex: 1,
  },
  timezoneItem: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingHorizontal: 16,
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#f0f0f0',
  },
  selectedTimezoneItem: {
    backgroundColor: '#e3f2fd',
  },
  timezoneInfo: {
    flex: 1,
  },
  timezoneName: {
    fontSize: 16,
    color: '#333',
    fontWeight: '500',
  },
  selectedTimezoneName: {
    color: '#1976d2',
  },
  timezoneOffset: {
    fontSize: 14,
    color: '#666',
    marginTop: 2,
  },
  selectedTimezoneOffset: {
    color: '#1565c0',
  },
  selectedIndicator: {
    width: 24,
    height: 24,
    borderRadius: 12,
    backgroundColor: '#1976d2',
    justifyContent: 'center',
    alignItems: 'center',
  },
  selectedIndicatorText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: 'bold',
  },
  emptyState: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingVertical: 32,
  },
  emptyStateText: {
    fontSize: 16,
    color: '#999',
  },
});

export default TimezonePicker;