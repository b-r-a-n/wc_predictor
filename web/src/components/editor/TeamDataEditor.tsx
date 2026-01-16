import { useMemo, useState } from 'react';
import { useSimulatorStore } from '../../store/simulatorStore';
import { getFlagEmoji } from '../../utils/formatting';
import { Button } from '../common/Button';
import type { Team, Confederation, TeamPreset } from '../../types';

type SortKey = 'name' | 'elo' | 'market' | 'fifa' | 'group' | 'confederation';
type SortDirection = 'asc' | 'desc';
type FilterConfederation = Confederation | 'all';

const CONFEDERATIONS: Confederation[] = ['UEFA', 'CONMEBOL', 'CONCACAF', 'CAF', 'AFC', 'OFC'];

interface SortHeaderProps {
  column: SortKey;
  label: string;
  className?: string;
  currentSortKey: SortKey;
  sortDirection: SortDirection;
  onSort: (key: SortKey) => void;
}

function SortHeader({
  column,
  label,
  className = '',
  currentSortKey,
  sortDirection,
  onSort,
}: SortHeaderProps) {
  return (
    <th
      className={`px-3 py-2 text-left text-xs font-medium text-gray-500 uppercase tracking-wider cursor-pointer hover:bg-gray-100 select-none ${className}`}
      onClick={() => onSort(column)}
    >
      <div className="flex items-center gap-1">
        {label}
        {currentSortKey === column && <span>{sortDirection === 'asc' ? '\u25B2' : '\u25BC'}</span>}
      </div>
    </th>
  );
}

export function TeamDataEditor() {
  const {
    teams,
    originalTeams,
    groups,
    editedTeamIds,
    presets,
    activePresetName,
    updateTeam,
    resetTeam,
    resetAllTeams,
    savePreset,
    loadPreset,
    deletePreset,
  } = useSimulatorStore();

  const [sortKey, setSortKey] = useState<SortKey>('name');
  const [sortDirection, setSortDirection] = useState<SortDirection>('asc');
  const [filterConfederation, setFilterConfederation] = useState<FilterConfederation>('all');
  const [filterGroup, setFilterGroup] = useState<string>('all');
  const [searchTerm, setSearchTerm] = useState('');
  const [showSaveModal, setShowSaveModal] = useState(false);
  const [newPresetName, setNewPresetName] = useState('');

  // Create team-to-group mapping
  const teamGroupMap = useMemo(() => {
    const map = new Map<number, string>();
    groups.forEach((group) => {
      group.teams.forEach((teamId) => {
        map.set(teamId, group.id);
      });
    });
    return map;
  }, [groups]);

  // Filter and sort teams
  const filteredAndSortedTeams = useMemo(() => {
    let filtered = [...teams];

    // Apply search filter
    if (searchTerm) {
      const term = searchTerm.toLowerCase();
      filtered = filtered.filter(
        (team) =>
          team.name.toLowerCase().includes(term) ||
          team.code.toLowerCase().includes(term)
      );
    }

    // Apply confederation filter
    if (filterConfederation !== 'all') {
      filtered = filtered.filter((team) => team.confederation === filterConfederation);
    }

    // Apply group filter
    if (filterGroup !== 'all') {
      filtered = filtered.filter((team) => teamGroupMap.get(team.id) === filterGroup);
    }

    // Sort
    filtered.sort((a, b) => {
      let comparison = 0;
      switch (sortKey) {
        case 'name':
          comparison = a.name.localeCompare(b.name);
          break;
        case 'elo':
          comparison = a.elo_rating - b.elo_rating;
          break;
        case 'market':
          comparison = a.market_value_millions - b.market_value_millions;
          break;
        case 'fifa':
          comparison = a.fifa_ranking - b.fifa_ranking;
          break;
        case 'group':
          comparison = (teamGroupMap.get(a.id) || '').localeCompare(teamGroupMap.get(b.id) || '');
          break;
        case 'confederation':
          comparison = a.confederation.localeCompare(b.confederation);
          break;
      }
      return sortDirection === 'asc' ? comparison : -comparison;
    });

    return filtered;
  }, [teams, searchTerm, filterConfederation, filterGroup, sortKey, sortDirection, teamGroupMap]);

  const handleSort = (key: SortKey) => {
    if (key === sortKey) {
      setSortDirection((d) => (d === 'asc' ? 'desc' : 'asc'));
    } else {
      setSortKey(key);
      setSortDirection(key === 'fifa' ? 'asc' : 'desc'); // FIFA ranking is better when lower
    }
  };

  const handleSavePreset = () => {
    if (newPresetName.trim()) {
      savePreset(newPresetName.trim());
      setNewPresetName('');
      setShowSaveModal(false);
    }
  };

  const handleDeletePreset = (name: string) => {
    if (confirm(`Delete preset "${name}"?`)) {
      deletePreset(name);
    }
  };

  const hasEdits = editedTeamIds.size > 0;

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
        <div>
          <h2 className="text-lg font-semibold text-gray-900">Team Data Editor</h2>
          <p className="text-sm text-gray-500">
            Edit team ratings to see how changes affect simulation results
          </p>
        </div>
        <div className="flex items-center gap-2">
          {hasEdits && (
            <span className="text-sm text-amber-600 font-medium">
              {editedTeamIds.size} team{editedTeamIds.size !== 1 ? 's' : ''} modified
            </span>
          )}
          <Button
            variant="outline"
            size="sm"
            onClick={() => setShowSaveModal(true)}
            disabled={!hasEdits}
          >
            Save As...
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={resetAllTeams}
            disabled={!hasEdits}
          >
            Reset All
          </Button>
        </div>
      </div>

      {/* Presets Section */}
      {presets.length > 0 && (
        <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-4">
          <h3 className="text-sm font-medium text-gray-700 mb-3">Saved Presets</h3>
          <div className="flex flex-wrap gap-2">
            {presets.map((preset) => (
              <PresetCard
                key={preset.name}
                preset={preset}
                isActive={activePresetName === preset.name}
                onLoad={() => loadPreset(preset.name)}
                onDelete={() => handleDeletePreset(preset.name)}
              />
            ))}
          </div>
        </div>
      )}

      {/* Save Preset Modal */}
      {showSaveModal && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg shadow-xl p-6 w-full max-w-md mx-4">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">Save Preset</h3>
            <input
              type="text"
              placeholder="Preset name..."
              value={newPresetName}
              onChange={(e) => setNewPresetName(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleSavePreset()}
              autoFocus
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 mb-4"
            />
            <div className="flex justify-end gap-2">
              <Button
                variant="outline"
                onClick={() => {
                  setShowSaveModal(false);
                  setNewPresetName('');
                }}
              >
                Cancel
              </Button>
              <Button
                onClick={handleSavePreset}
                disabled={!newPresetName.trim()}
              >
                Save
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* Filters */}
      <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-4">
        <div className="flex flex-wrap gap-4">
          {/* Search */}
          <div className="flex-1 min-w-[200px]">
            <label htmlFor="search" className="block text-sm font-medium text-gray-700 mb-1">
              Search
            </label>
            <input
              id="search"
              type="text"
              placeholder="Search teams..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            />
          </div>

          {/* Confederation filter */}
          <div>
            <label htmlFor="confederation" className="block text-sm font-medium text-gray-700 mb-1">
              Confederation
            </label>
            <select
              id="confederation"
              value={filterConfederation}
              onChange={(e) => setFilterConfederation(e.target.value as FilterConfederation)}
              className="px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            >
              <option value="all">All</option>
              {CONFEDERATIONS.map((conf) => (
                <option key={conf} value={conf}>
                  {conf}
                </option>
              ))}
            </select>
          </div>

          {/* Group filter */}
          <div>
            <label htmlFor="group" className="block text-sm font-medium text-gray-700 mb-1">
              Group
            </label>
            <select
              id="group"
              value={filterGroup}
              onChange={(e) => setFilterGroup(e.target.value)}
              className="px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            >
              <option value="all">All</option>
              {groups.map((group) => (
                <option key={group.id} value={group.id}>
                  Group {group.id}
                </option>
              ))}
            </select>
          </div>
        </div>
      </div>

      {/* Table */}
      <div className="bg-white rounded-lg shadow-sm border border-gray-200 overflow-hidden">
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <SortHeader
                  column="name"
                  label="Team"
                  className="w-48"
                  currentSortKey={sortKey}
                  sortDirection={sortDirection}
                  onSort={handleSort}
                />
                <SortHeader
                  column="group"
                  label="Group"
                  className="w-20"
                  currentSortKey={sortKey}
                  sortDirection={sortDirection}
                  onSort={handleSort}
                />
                <SortHeader
                  column="confederation"
                  label="Conf"
                  className="w-24"
                  currentSortKey={sortKey}
                  sortDirection={sortDirection}
                  onSort={handleSort}
                />
                <SortHeader
                  column="elo"
                  label="ELO Rating"
                  currentSortKey={sortKey}
                  sortDirection={sortDirection}
                  onSort={handleSort}
                />
                <SortHeader
                  column="market"
                  label="Market Value (M)"
                  currentSortKey={sortKey}
                  sortDirection={sortDirection}
                  onSort={handleSort}
                />
                <SortHeader
                  column="fifa"
                  label="FIFA Rank"
                  currentSortKey={sortKey}
                  sortDirection={sortDirection}
                  onSort={handleSort}
                />
                <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 uppercase tracking-wider w-20">
                  Actions
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {filteredAndSortedTeams.map((team) => (
                <TeamRow
                  key={team.id}
                  team={team}
                  originalTeam={originalTeams.find((t) => t.id === team.id)}
                  groupId={teamGroupMap.get(team.id) || '-'}
                  isEdited={editedTeamIds.has(team.id)}
                  onUpdate={updateTeam}
                  onReset={resetTeam}
                />
              ))}
            </tbody>
          </table>
        </div>

        {filteredAndSortedTeams.length === 0 && (
          <div className="flex items-center justify-center h-32 text-gray-500">
            No teams match your filters
          </div>
        )}
      </div>

      {/* Info notice */}
      <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
        <p className="text-sm text-blue-800">
          <strong>Note:</strong> Changes to team data will take effect when you run a new simulation.
          The simulator will be re-initialized with your custom values.
        </p>
      </div>
    </div>
  );
}

interface TeamRowProps {
  team: Team;
  originalTeam: Team | undefined;
  groupId: string;
  isEdited: boolean;
  onUpdate: (teamId: number, field: keyof Team, value: number) => void;
  onReset: (teamId: number) => void;
}

function TeamRow({ team, originalTeam, groupId, isEdited, onUpdate, onReset }: TeamRowProps) {
  const [editingField, setEditingField] = useState<string | null>(null);

  const handleFieldChange = (field: 'elo_rating' | 'market_value_millions' | 'fifa_ranking', value: string) => {
    const numValue = parseFloat(value);
    if (!isNaN(numValue) && numValue >= 0) {
      onUpdate(team.id, field, numValue);
    }
  };

  const getFieldDiff = (field: 'elo_rating' | 'market_value_millions' | 'fifa_ranking'): string | null => {
    if (!originalTeam || !isEdited) return null;
    const diff = team[field] - originalTeam[field];
    if (Math.abs(diff) < 0.01) return null;
    const sign = diff > 0 ? '+' : '';
    if (field === 'fifa_ranking') {
      return `${sign}${Math.round(diff)}`;
    }
    return `${sign}${diff.toFixed(1)}`;
  };

  return (
    <tr className={isEdited ? 'bg-amber-50' : ''}>
      {/* Team name */}
      <td className="px-3 py-2 whitespace-nowrap">
        <div className="flex items-center gap-2">
          <span className="text-lg">{getFlagEmoji(team.code)}</span>
          <div>
            <span className="text-sm font-medium text-gray-900">{team.name}</span>
            <span className="ml-2 text-xs text-gray-400">{team.code}</span>
          </div>
        </div>
      </td>

      {/* Group */}
      <td className="px-3 py-2 whitespace-nowrap">
        <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800">
          {groupId}
        </span>
      </td>

      {/* Confederation */}
      <td className="px-3 py-2 whitespace-nowrap">
        <span className="text-sm text-gray-600">{team.confederation}</span>
      </td>

      {/* ELO Rating */}
      <td className="px-3 py-2 whitespace-nowrap">
        <EditableCell
          value={team.elo_rating}
          isEditing={editingField === 'elo'}
          diff={getFieldDiff('elo_rating')}
          onStartEdit={() => setEditingField('elo')}
          onEndEdit={() => setEditingField(null)}
          onChange={(value) => handleFieldChange('elo_rating', value)}
          formatValue={(v) => Math.round(v).toString()}
          min={1000}
          max={2500}
          step={1}
        />
      </td>

      {/* Market Value */}
      <td className="px-3 py-2 whitespace-nowrap">
        <EditableCell
          value={team.market_value_millions}
          isEditing={editingField === 'market'}
          diff={getFieldDiff('market_value_millions')}
          onStartEdit={() => setEditingField('market')}
          onEndEdit={() => setEditingField(null)}
          onChange={(value) => handleFieldChange('market_value_millions', value)}
          formatValue={(v) => v.toFixed(1)}
          min={0}
          max={2000}
          step={0.1}
        />
      </td>

      {/* FIFA Ranking */}
      <td className="px-3 py-2 whitespace-nowrap">
        <EditableCell
          value={team.fifa_ranking}
          isEditing={editingField === 'fifa'}
          diff={getFieldDiff('fifa_ranking')}
          onStartEdit={() => setEditingField('fifa')}
          onEndEdit={() => setEditingField(null)}
          onChange={(value) => handleFieldChange('fifa_ranking', value)}
          formatValue={(v) => Math.round(v).toString()}
          min={1}
          max={200}
          step={1}
          invertDiffColor
        />
      </td>

      {/* Actions */}
      <td className="px-3 py-2 whitespace-nowrap">
        {isEdited && (
          <button
            onClick={() => onReset(team.id)}
            className="text-sm text-blue-600 hover:text-blue-800 font-medium"
          >
            Reset
          </button>
        )}
      </td>
    </tr>
  );
}

interface EditableCellProps {
  value: number;
  isEditing: boolean;
  diff: string | null;
  onStartEdit: () => void;
  onEndEdit: () => void;
  onChange: (value: string) => void;
  formatValue: (value: number) => string;
  min: number;
  max: number;
  step: number;
  invertDiffColor?: boolean;
}

function EditableCell({
  value,
  isEditing,
  diff,
  onStartEdit,
  onEndEdit,
  onChange,
  formatValue,
  min,
  max,
  step,
  invertDiffColor = false,
}: EditableCellProps) {
  const [inputValue, setInputValue] = useState(formatValue(value));

  const handleFocus = () => {
    setInputValue(formatValue(value));
    onStartEdit();
  };

  const handleBlur = () => {
    onChange(inputValue);
    onEndEdit();
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      onChange(inputValue);
      onEndEdit();
    } else if (e.key === 'Escape') {
      setInputValue(formatValue(value));
      onEndEdit();
    }
  };

  // Determine diff color
  let diffColor = 'text-gray-500';
  if (diff) {
    const isPositive = diff.startsWith('+');
    if (invertDiffColor) {
      // For FIFA ranking, lower is better
      diffColor = isPositive ? 'text-red-500' : 'text-green-500';
    } else {
      diffColor = isPositive ? 'text-green-500' : 'text-red-500';
    }
  }

  if (isEditing) {
    return (
      <input
        type="number"
        value={inputValue}
        onChange={(e) => setInputValue(e.target.value)}
        onBlur={handleBlur}
        onKeyDown={handleKeyDown}
        min={min}
        max={max}
        step={step}
        autoFocus
        className="w-24 px-2 py-1 text-sm border border-blue-500 rounded focus:ring-2 focus:ring-blue-500 focus:outline-none"
      />
    );
  }

  return (
    <div
      onClick={handleFocus}
      className="cursor-pointer hover:bg-gray-100 rounded px-2 py-1 -mx-2 -my-1"
    >
      <span className="text-sm font-medium text-gray-900">{formatValue(value)}</span>
      {diff && (
        <span className={`ml-1 text-xs ${diffColor}`}>({diff})</span>
      )}
    </div>
  );
}

interface PresetCardProps {
  preset: TeamPreset;
  isActive: boolean;
  onLoad: () => void;
  onDelete: () => void;
}

function PresetCard({ preset, isActive, onLoad, onDelete }: PresetCardProps) {
  const date = new Date(preset.createdAt);
  const formattedDate = date.toLocaleDateString(undefined, {
    month: 'short',
    day: 'numeric',
  });

  return (
    <div
      className={`flex items-center gap-3 px-3 py-2 rounded-lg border ${
        isActive
          ? 'border-blue-500 bg-blue-50'
          : 'border-gray-200 bg-gray-50 hover:bg-gray-100'
      }`}
    >
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium text-gray-900 truncate">{preset.name}</span>
          {isActive && (
            <span className="text-xs text-blue-600 font-medium">(active)</span>
          )}
        </div>
        <span className="text-xs text-gray-500">{formattedDate}</span>
      </div>
      <div className="flex items-center gap-1">
        {!isActive && (
          <button
            onClick={onLoad}
            className="px-2 py-1 text-xs font-medium text-blue-600 hover:text-blue-800 hover:bg-blue-100 rounded"
          >
            Load
          </button>
        )}
        <button
          onClick={onDelete}
          className="px-2 py-1 text-xs font-medium text-red-600 hover:text-red-800 hover:bg-red-100 rounded"
        >
          Delete
        </button>
      </div>
    </div>
  );
}
