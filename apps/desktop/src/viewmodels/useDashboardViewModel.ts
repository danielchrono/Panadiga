import { useState, useEffect, useCallback } from 'react';
import { SystemOverview } from '@/models';
import { invokeCommand, isDemoModeActive, setDemoMode } from '@/services/ipc';
import { logger } from '@/utils/logger';

export function useDashboardViewModel() {
  const [overview, setOverview] = useState<SystemOverview | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [demoMode, setDemoState] = useState<boolean>(isDemoModeActive());

  const fetchOverview = useCallback(async () => {
    setLoading(true);
    try {
      const data = await invokeCommand<SystemOverview>('get_system_overview');
      setOverview(data);
    } catch (err) {
      logger.error('DashboardViewModel', 'Error fetching system overview', err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchOverview();
  }, [fetchOverview]);

  const toggleDemoMode = useCallback((val: boolean) => {
    setDemoMode(val);
    setDemoState(val);
    fetchOverview();
  }, [fetchOverview]);

  return {
    overview,
    loading,
    demoMode,
    toggleDemoMode,
    refresh: fetchOverview,
  };
}
