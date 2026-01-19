import type { Venue } from '../../types';

interface VenueSelectorProps {
  venues: Venue[];
  selectedVenueId: string | null;
  onSelect: (venueId: string) => void;
}

export function VenueSelector({ venues, selectedVenueId, onSelect }: VenueSelectorProps) {
  // Group venues by country
  const venuesByCountry = venues.reduce<Record<string, Venue[]>>((acc, venue) => {
    if (!acc[venue.country]) {
      acc[venue.country] = [];
    }
    acc[venue.country].push(venue);
    return acc;
  }, {});

  // Country order: USA first (most venues), then Canada, then Mexico
  const countryOrder = ['USA', 'Canada', 'Mexico'];

  return (
    <div className="mb-4">
      <label htmlFor="venue-select" className="block text-sm font-medium text-gray-700 mb-1">
        Select Venue
      </label>
      <select
        id="venue-select"
        value={selectedVenueId || ''}
        onChange={(e) => onSelect(e.target.value)}
        className="w-full md:w-96 px-3 py-2 border border-gray-300 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
      >
        <option value="">Choose a stadium...</option>
        {countryOrder.map((country) => {
          const countryVenues = venuesByCountry[country];
          if (!countryVenues || countryVenues.length === 0) return null;

          return (
            <optgroup key={country} label={country}>
              {countryVenues.map((venue) => (
                <option key={venue.id} value={venue.id}>
                  {venue.name} - {venue.city}
                </option>
              ))}
            </optgroup>
          );
        })}
      </select>
    </div>
  );
}
