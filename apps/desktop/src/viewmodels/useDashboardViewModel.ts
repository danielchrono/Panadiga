import { useState, useEffect, useCallback } from 'react';
import { SystemOverview } from '@/models';
import { invokeCommand, isDemoModeActive, setDemoMode } from '@/services/ipc';
import { logger } from '@/utils/logger';

export function useDashboardViewModel() {
  const [overview, setOverview] = useState<SystemOverview | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [demoMode, setDemoState] = useState<boolean>(isDemoModeActive());

  const fetchOverview = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await invokeCommand<SystemOverview>('get_system_overview');
      setOverview(data);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
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
    error,
    demoMode,
    toggleDemoMode,
    refresh: fetchOverview,
  };
}
