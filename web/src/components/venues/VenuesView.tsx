import { useState, useMemo } from 'react';
import { useSimulatorStore } from '../../store/simulatorStore';
import { VenueSelector } from './VenueSelector';
import { VenueMatchCard } from './VenueMatchCard';
import type { Venue } from '../../types';

export function VenuesView() {
  const { venues, schedule, teams, groups, results } = useSimulatorStore();
  const [selectedVenueId, setSelectedVenueId] = useState<string | null>(null);

  // Get the selected venue
  const selectedVenue: Venue | null = useMemo(() => {
    if (!selectedVenueId || !venues) return null;
    return venues.find(v => v.id === selectedVenueId) || null;
  }, [selectedVenueId, venues]);

  // Filter matches for the selected venue, sorted by date
  const venueMatches = useMemo(() => {
    if (!selectedVenueId || !schedule?.matches) return [];

    return schedule.matches
      .filter(match => match.venueId === selectedVenueId)
      .sort((a, b) => {
        // Sort by date, then by time
        const dateCompare = a.date.localeCompare(b.date);
        if (dateCompare !== 0) return dateCompare;
        return a.time.localeCompare(b.time);
      });
  }, [selectedVenueId, schedule]);

  // Loading states
  if (!venues || venues.length === 0) {
    return (
      <div className="p-6 text-center text-gray-500">
        Loading venues...
      </div>
    );
  }

  if (!schedule) {
    return (
      <div className="p-6 text-center text-gray-500">
        Loading schedule...
      </div>
    );
  }

  return (
    <div className="p-4 md:p-6 max-w-4xl mx-auto">
      <div className="mb-6">
        <h2 className="text-2xl font-bold text-gray-900 mb-2">Match Venues</h2>
        <p className="text-gray-600">
          Browse all World Cup 2026 matches by stadium location.
          {!results && ' Run a simulation to see predicted matchups for knockout rounds.'}
        </p>
      </div>

      <VenueSelector
        venues={venues}
        selectedVenueId={selectedVenueId}
        onSelect={setSelectedVenueId}
      />

      {selectedVenue && (
        <div className="mt-6">
          {/* Venue header */}
          <div className="bg-gradient-to-r from-blue-600 to-blue-700 rounded-lg p-4 mb-4 text-white">
            <h3 className="text-xl font-bold">{selectedVenue.name}</h3>
            <p className="text-blue-100">{selectedVenue.city}, {selectedVenue.country}</p>
            <p className="text-sm text-blue-200 mt-1">
              {venueMatches.length} match{venueMatches.length !== 1 ? 'es' : ''} scheduled
            </p>
          </div>

          {/* Match list */}
          {venueMatches.length === 0 ? (
            <div className="text-center py-8 text-gray-500">
              No matches scheduled at this venue.
            </div>
          ) : (
            <div className="space-y-3">
              {venueMatches.map((match) => (
                <VenueMatchCard
                  key={match.matchNumber}
                  match={match}
                  teams={teams}
                  groups={groups}
                  results={results}
                />
              ))}
            </div>
          )}
        </div>
      )}

      {!selectedVenueId && (
        <div className="mt-8 text-center py-12 bg-gray-50 rounded-lg border-2 border-dashed border-gray-200">
          <p className="text-gray-500">
            Select a venue above to see its scheduled matches.
          </p>
        </div>
      )}
    </div>
  );
}
