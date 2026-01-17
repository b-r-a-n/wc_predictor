import { useState, useMemo } from 'react';
import type { Team } from '../../types';
import { getFlagEmoji } from '../../utils/formatting';

interface TeamSelectorProps {
  teams: Team[];
  selectedTeam: Team | null;
  onChange: (team: Team | null) => void;
  label: string;
  excludeTeamId?: number;
}

export function TeamSelector({
  teams,
  selectedTeam,
  onChange,
  label,
  excludeTeamId,
}: TeamSelectorProps) {
  const [searchQuery, setSearchQuery] = useState('');
  const [isOpen, setIsOpen] = useState(false);

  const filteredTeams = useMemo(() => {
    return teams
      .filter((t) => t.id !== excludeTeamId)
      .filter(
        (t) =>
          t.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
          t.code.toLowerCase().includes(searchQuery.toLowerCase())
      )
      .sort((a, b) => a.name.localeCompare(b.name));
  }, [teams, searchQuery, excludeTeamId]);

  const handleSelect = (team: Team) => {
    onChange(team);
    setIsOpen(false);
    setSearchQuery('');
  };

  return (
    <div className="relative">
      <label className="block text-sm font-medium text-gray-700 mb-1">{label}</label>

      <div
        className="relative cursor-pointer"
        onClick={() => setIsOpen(!isOpen)}
      >
        <div className="flex items-center gap-2 px-3 py-2 border border-gray-300 rounded-lg bg-white hover:border-gray-400">
          {selectedTeam ? (
            <>
              <span className="text-lg">{getFlagEmoji(selectedTeam.code)}</span>
              <span className="font-medium">{selectedTeam.name}</span>
            </>
          ) : (
            <span className="text-gray-400">Select a team...</span>
          )}
          <span className="ml-auto text-gray-400">&#9660;</span>
        </div>
      </div>

      {isOpen && (
        <div className="absolute z-10 w-full mt-1 bg-white border border-gray-200 rounded-lg shadow-lg max-h-64 overflow-hidden">
          <div className="p-2 border-b border-gray-100">
            <input
              type="text"
              placeholder="Search teams..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full px-3 py-2 border border-gray-200 rounded focus:outline-none focus:border-blue-500"
              onClick={(e) => e.stopPropagation()}
              autoFocus
            />
          </div>
          <div className="overflow-y-auto max-h-48">
            {filteredTeams.map((team) => (
              <div
                key={team.id}
                className="flex items-center gap-2 px-3 py-2 hover:bg-gray-100 cursor-pointer"
                onClick={() => handleSelect(team)}
              >
                <span className="text-lg">{getFlagEmoji(team.code)}</span>
                <span className="font-medium">{team.name}</span>
                <span className="text-xs text-gray-500 ml-auto">
                  ELO: {team.elo_rating} | Form: {team.sofascore_form.toFixed(1)}
                </span>
              </div>
            ))}
            {filteredTeams.length === 0 && (
              <div className="px-3 py-4 text-center text-gray-500">No teams found</div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
