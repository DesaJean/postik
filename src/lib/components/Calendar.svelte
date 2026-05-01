<script lang="ts">
  import { onMount } from 'svelte';
  import { calendarStore } from '../stores/calendar.svelte';
  import { settingsStore } from '../stores/settings.svelte';
  import Switch from './Switch.svelte';
  import type { SyncRangeKind } from '../types';

  onMount(() => {
    calendarStore.load();
  });

  const OFFSET_OPTIONS = [
    { value: 0, label: 'at time' },
    { value: 5 * 60, label: '5m' },
    { value: 10 * 60, label: '10m' },
    { value: 15 * 60, label: '15m' },
  ];

  const RANGES: Array<{ id: SyncRangeKind; label: string }> = [
    { id: 'today', label: 'Today' },
    { id: 'seven_days', label: '7 days' },
  ];

  function fmtTime(unixSeconds: number): string {
    const d = new Date(unixSeconds * 1000);
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }

  function nextOffset(current: number): number {
    const idx = OFFSET_OPTIONS.findIndex((o) => o.value === current);
    return OFFSET_OPTIONS[(idx + 1) % OFFSET_OPTIONS.length].value;
  }

  function offsetLabel(seconds: number): string {
    return OFFSET_OPTIONS.find((o) => o.value === seconds)?.label ?? `${seconds}s`;
  }
</script>

<div class="calendar">
  {#if !calendarStore.isConfigured}
    <div class="empty-state">
      <h3>Setup needed</h3>
      <p>
        Google Calendar credentials aren't baked into this build. See
        <code>docs/google-calendar-setup.md</code> in the repo for the one-time Google Cloud steps.
      </p>
    </div>
  {:else if !calendarStore.account}
    <div class="empty-state">
      <h3>Connect Google Calendar</h3>
      <p>Sync your events into Postik so they ring before they start.</p>
      <button class="primary-btn" onclick={() => calendarStore.connect()}>
        {calendarStore.loading ? 'Connecting…' : 'Connect Google Calendar'}
      </button>
      {#if calendarStore.error}<p class="error">{calendarStore.error}</p>{/if}
      <p class="muted">
        Read-only access. We only read your primary calendar; tokens stay on your machine.
      </p>
    </div>
  {:else}
    <div class="header-row">
      <div class="account">
        <span class="email">{calendarStore.account.email}</span>
      </div>
      <button
        class="icon-btn"
        onclick={() => calendarStore.sync()}
        disabled={calendarStore.loading}
        aria-label="Sync now"
        title="Sync now"
      >
        {calendarStore.loading ? '…' : '⟳'}
      </button>
      <button
        class="icon-btn"
        onclick={() => calendarStore.disconnect()}
        aria-label="Disconnect"
        title="Disconnect"
      >
        ⏻
      </button>
    </div>

    <div class="auto-sync-row">
      <span class="auto-sync-label">Auto-sync (15 min)</span>
      <Switch
        checked={settingsStore.googleCalendarAutoSync}
        onChange={(v) => settingsStore.setGoogleCalendarAutoSync(v)}
        label="Auto-sync"
      />
    </div>

    <div class="ranges">
      {#each RANGES as r (r.id)}
        <button
          class="range-chip"
          class:active={calendarStore.range === r.id}
          onclick={() => calendarStore.setRange(r.id)}
        >
          {r.label}
        </button>
      {/each}
    </div>

    {#if calendarStore.error}
      <p class="error">{calendarStore.error}</p>
    {/if}

    {#if calendarStore.events.length === 0}
      <div class="empty-events">
        <p class="muted">No events in this range.</p>
        <p class="muted small">Click ⟳ to sync.</p>
      </div>
    {:else}
      <ul class="event-list">
        {#each calendarStore.events as ev (ev.event_id)}
          <li class="event-item">
            <button class="event-row" onclick={() => calendarStore.openEvent(ev.event_id)}>
              <span class="time">{fmtTime(ev.start_time)}</span>
              <span class="title">{ev.title}</span>
            </button>
            <button
              class="offset-chip"
              onclick={() =>
                calendarStore.setEventTimer(
                  ev.event_id,
                  ev.timer_armed,
                  nextOffset(ev.timer_offset_seconds),
                )}
              title="Cycle alarm offset"
            >
              {offsetLabel(ev.timer_offset_seconds)}
            </button>
            <button
              class="bell"
              class:armed={ev.timer_armed}
              onclick={() =>
                calendarStore.setEventTimer(ev.event_id, !ev.timer_armed, ev.timer_offset_seconds)}
              title={ev.timer_armed ? 'Disable timer' : 'Enable timer'}
              aria-label={ev.timer_armed ? 'Disable timer' : 'Enable timer'}
            >
              {ev.timer_armed ? '🔔' : '🔕'}
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  {/if}
</div>

<style>
  .calendar {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 8px 12px;
    flex: 1;
    overflow-y: auto;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 24px 8px;
    text-align: center;
  }
  .empty-state h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }
  .empty-state p {
    margin: 0;
    font-size: 12px;
    line-height: 1.4;
  }
  .empty-state code {
    font-size: 11px;
    background: rgba(0, 0, 0, 0.06);
    padding: 1px 4px;
    border-radius: 3px;
  }

  .primary-btn {
    margin-top: 6px;
    padding: 8px 14px;
    border-radius: 6px;
    background: var(--accent);
    color: white;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    align-self: center;
  }
  .primary-btn:hover {
    background: #c64f29;
  }
  .primary-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .header-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .account {
    flex: 1;
    min-width: 0;
  }
  .email {
    font-size: 11px;
    color: rgba(0, 0, 0, 0.65);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    display: block;
  }
  .icon-btn {
    width: 24px;
    height: 24px;
    border-radius: 4px;
    background: rgba(0, 0, 0, 0.04);
    font-size: 13px;
    line-height: 1;
    cursor: pointer;
  }
  .icon-btn:hover {
    background: rgba(216, 90, 48, 0.1);
  }
  .icon-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .auto-sync-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 0;
  }
  .auto-sync-label {
    font-size: 11px;
    color: rgba(0, 0, 0, 0.65);
  }

  .ranges {
    display: flex;
    gap: 6px;
  }
  .range-chip {
    padding: 4px 10px;
    border-radius: 11px;
    background: rgba(0, 0, 0, 0.05);
    font-size: 11px;
    cursor: pointer;
  }
  .range-chip.active {
    background: var(--accent);
    color: white;
  }

  .empty-events {
    padding: 24px 8px;
    text-align: center;
  }
  .muted {
    color: rgba(0, 0, 0, 0.45);
    font-size: 11px;
    margin: 4px 0;
  }
  .muted.small {
    font-size: 10px;
  }
  .error {
    color: var(--accent);
    font-size: 11px;
    margin: 4px 0;
  }

  .event-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .event-item {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .event-row {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    border-radius: 4px;
    background: transparent;
    text-align: left;
    cursor: pointer;
    min-width: 0;
  }
  .event-row:hover {
    background: rgba(0, 0, 0, 0.04);
  }
  .time {
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    font-weight: 600;
    color: rgba(0, 0, 0, 0.6);
  }
  .title {
    font-size: 12px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    min-width: 0;
  }
  .offset-chip {
    padding: 2px 6px;
    border-radius: 8px;
    background: rgba(0, 0, 0, 0.05);
    font-size: 10px;
    cursor: pointer;
    font-variant-numeric: tabular-nums;
  }
  .offset-chip:hover {
    background: rgba(216, 90, 48, 0.12);
  }
  .bell {
    width: 22px;
    height: 22px;
    border-radius: 4px;
    background: transparent;
    font-size: 12px;
    cursor: pointer;
    opacity: 0.4;
  }
  .bell.armed {
    opacity: 1;
  }
  .bell:hover {
    background: rgba(0, 0, 0, 0.05);
  }

  @media (prefers-color-scheme: dark) {
    .email,
    .auto-sync-label,
    .time {
      color: rgba(255, 255, 255, 0.65);
    }
    .icon-btn,
    .range-chip,
    .offset-chip {
      background: rgba(255, 255, 255, 0.06);
      color: inherit;
    }
    .icon-btn:hover,
    .event-row:hover,
    .offset-chip:hover {
      background: rgba(255, 255, 255, 0.1);
    }
    .empty-state code {
      background: rgba(255, 255, 255, 0.08);
    }
    .muted {
      color: rgba(255, 255, 255, 0.45);
    }
  }
</style>
