# Schedule Validation Progress

## Overview

Validating that each match ID correctly maps to its venue/date/time against official FIFA and host city sources.

**Last Updated:** 2026-01-20
**Status:** All 16 venues validated

## Validation Status

| # | Venue ID | Stadium | City | Matches | Status | Issues Fixed |
|---|----------|---------|------|---------|--------|--------------|
| 1 | `mercedes_benz` | Mercedes-Benz Stadium | Atlanta, GA | 8 | ✅ Complete | 2 times |
| 2 | `metlife` | MetLife Stadium | East Rutherford, NJ | 8 | ✅ Complete | 4 times, 2 teams |
| 3 | `att` | AT&T Stadium | Arlington, TX | 9 | ✅ Complete | 5 times, 1 venue move |
| 4 | `hard_rock` | Hard Rock Stadium | Miami, FL | 7 | ✅ Complete | 4 times, 4 teams, 1 venue move |
| 5 | `sofi` | SoFi Stadium | Inglewood, CA | 8 | ✅ Complete | 4 times, 1 team |
| 6 | `nrg` | NRG Stadium | Houston, TX | 7 | ✅ Complete | 5 times, 4 teams |
| 7 | `lincoln_financial` | Lincoln Financial Field | Philadelphia, PA | 6 | ✅ Complete | 5 times, 4 teams |
| 8 | `arrowhead` | Arrowhead Stadium | Kansas City, MO | 6 | ✅ Complete | 3 times, 2 teams |
| 9 | `lumen` | Lumen Field | Seattle, WA | 6 | ✅ Complete | 4 times, 1 team, 1 R16 bracket fix |
| 10 | `levis` | Levi's Stadium | Santa Clara, CA | 6 | ✅ Complete | 5 times, 2 teams, 1 date, 1 R32 |
| 11 | `gillette` | Gillette Stadium | Foxborough, MA | 7 | ✅ Complete | 5 times, 4 match #s, 2 teams, 1 R32 |
| 12 | `bmo` | BMO Field | Toronto, ON | 6 | ✅ Complete | 3 times, 3 teams, 1 R32 |
| 13 | `bc_place` | BC Place | Vancouver, BC | 7 | ✅ Complete | 4 times, 1 date, 3 teams, 1 R16 |
| 14 | `azteca` | Estadio Azteca | Mexico City | 5 | ✅ Complete | 3 times, 1 team, 1 R32, 1 R16 |
| 15 | `bbva` | Estadio BBVA | Monterrey | 4 | ✅ Complete | 3 times, 1 date, 1 team, 1 R32 |
| 16 | `akron` | Estadio Akron | Guadalajara | 4 | ✅ Complete | 4 times, 3 match #s, 2 teams |

**Progress:** 16/16 venues validated (100%) ✅

## Common Issues Found

### 1. Time Discrepancies
Most venues had incorrect kickoff times, often off by several hours. Times in `schedule.json` did not match official FIFA announcements.

### 2. Team Placeholder Mismatches
`schedule.json` had diverged from `schedule_scraper.py` - the scraper often had correct team placeholders while schedule.json had incorrect ones.

### 3. Venue Misassignments
Two matches were assigned to wrong venues:
- **Japan vs Tunisia (Jun 20)**: Was at AT&T, moved to BBVA Monterrey (1000th FIFA World Cup match)
- **Norway vs France (Jun 26)**: Was at Hard Rock, moved to Gillette Stadium

## Detailed Fix Log

### Atlanta (mercedes_benz) - Pilot
| Match # | Date | Fix |
|---------|------|-----|
| 49 | Jun 24 | Time 15:00 → 18:00 |
| 67 | Jun 27 | Time 15:00 → 19:30 |

### MetLife (metlife)
| Match # | Date | Fix |
|---------|------|-----|
| 6 | Jun 13 | Time 15:00 → 18:00 |
| 42 | Jun 22 | Time 15:00 → 20:00, Teams I4/I2 → I3/I2 |
| 55 | Jun 25 | Time 15:00 → 16:00, Teams E4/E1 → E3/E1 |
| 71 | Jun 27 | Time 21:00 → 17:00 |

### AT&T Stadium (att)
| Match # | Date | Fix |
|---------|------|-----|
| 11 | Jun 14 | Time 18:00 → 16:00 |
| 24 | Jun 17 | Time 21:00 → 16:00 |
| 35 | Jun 20 | **Moved to BBVA**, Teams F1/F3 → F2/F3 |
| 43 | Jun 22 | Time 18:00 → 13:00 |
| 59 | Jun 25 | Time 21:00 → 19:00, Teams F4/F1 → F2/F4 |
| 69 | Jun 27 | Time 18:00 → 22:00 |

### Hard Rock (hard_rock)
| Match # | Date | Fix |
|---------|------|-----|
| 14 | Jun 15 | Time 15:00 → 18:00, Teams H3/H4 → H2/H3 |
| 38 | Jun 21 | Time 15:00 → 18:00, Teams H4/H2 → H3/H4 |
| 50 | Jun 24 | Time 15:00 → 18:00, Teams C4/C1 → C3/C1 |
| 62 | Jun 26 | **Moved to Gillette**, Teams I2/I3 → I3/I1 |
| 68 | Jun 27 | Time 15:00 → 19:30, Teams K3/K2 → K2/K1 |

### SoFi (sofi)
| Match # | Date | Fix |
|---------|------|-----|
| 4 | Jun 12 | Teams D1/D2 → D1/D3 (USA vs Paraguay) |
| 28 | Jun 18 | Time 21:00 → 15:00 |
| 39 | Jun 21 | Time 18:00 → 15:00 |
| 57 | Jun 25 | Time 18:00 → 22:00 |
| 98 | Jul 10 | Time 15:00 → 12:00 |

### NRG (nrg)
| Match # | Date | Fix |
|---------|------|-----|
| 9 | Jun 14 | Time 12:00 → 13:00, Teams E1/E2 → E1/E4 |
| 22 | Jun 17 | Time 15:00 → 13:00, Teams K3/K4 → K1/K4 |
| 36 | Jun 20 | Time 21:00 → 13:00, Teams F4/F2 → F1/F4 |
| 45 | Jun 23 | Time 12:00 → 13:00 |
| 66 | Jun 26 | Time 21:00 → 20:00, Teams H2/H3 → H4/H2 |

### Lincoln Financial (lincoln_financial)
| Match # | Date | Fix |
|---------|------|-----|
| 10 | Jun 14 | Time 15:00 → 19:00, Teams E3/E4 → E2/E3 |
| 30 | Jun 19 | Time 15:00 → 21:00, Teams C1/C3 → C1/C4 |
| 41 | Jun 22 | Time 12:00 → 17:00, Teams I1/I3 → I1/I4 |
| 56 | Jun 25 | Time 15:00 → 16:00, Teams E2/E3 → E4/E2 |
| 72 | Jun 27 | Time 21:00 → 17:00 |

### Arrowhead (arrowhead)
| Match # | Date | Fix |
|---------|------|-----|
| 33 | Jun 20 | Time 12:00 → 20:00, Teams E4/E2 → E3/E4 |
| 60 | Jun 25 | Time 21:00 → 19:00, Teams F2/F3 → F3/F1 |
| 70 | Jun 27 | Time 18:00 → 22:00 |

### Seattle (lumen)
| Match # | Date | Fix |
|---------|------|-----|
| 15 | Jun 15 | Time 18:00 → 15:00 |
| 31 | Jun 19 | Time 18:00 → 15:00, Teams D1/D2 → D1/D3 (scraper only) |
| 53 | Jun 24 | Time 18:00 → 15:00, **Added missing match** (scraper only) |
| 63 | Jun 26 | Time 18:00 → 23:00 |
| 82 | Jul 1 | Teams 2A/2B → 1G/3A/E/H/I/J (scraper only) |
| 93/94 | Jul 6 | **R16 bracket swap**: Match 93 (ATT) W9/W10 → W11/W12, Match 94 (lumen) W11/W12 → W9/W10 |

**Notes:**
- Seattle hosts 6 matches, not 4 as originally documented
- Scraper R16 structure differs significantly from schedule.json (Seattle not in scraper R16 list)
- Official sources: [Visit Seattle](https://visitseattle.org/press/press-releases/seattle-world-cup-matches-set/), [FOX Sports](https://www.foxsports.com/stories/soccer/2026-world-cup-matches-seattle-start-times-dates-locations)

### San Francisco (levis)
| Match # | Date | Fix |
|---------|------|-----|
| 7 | Jun 13 | Time 18:00 → 15:00, Teams B3/B4 → B3/B2 (Qatar vs Switzerland) |
| 19 | Jun 16→17 | Date 2026-06-16 → 2026-06-17, Time 18:00 → 00:00 (midnight ET) |
| 32 | Jun 19→20 | Date 2026-06-19 → 2026-06-20, Time 21:00 → 00:00, Teams D4/D2 → D4/D3 |
| 44 | Jun 22 | Time 21:00 → 23:00 |
| 58 | Jun 25 | Time 18:00 → 22:00 |
| 81 | Jul 1 | Time 20:00 → 22:00 (R32), Scraper: 1B/3A/C/D → 1D/3B/E/F/I/J |

**Notes:**
- San Francisco hosts 6 matches (5 group stage + 1 R32), not 4 as previously documented
- Times are late-night (9PM-midnight PT) for East Coast prime time coverage
- Scraper R32 bracket had completely wrong matchup (1B vs 3A/C/D instead of 1D vs 3B/E/F/I/J)
- Official sources: [CBS San Francisco](https://www.cbsnews.com/sanfrancisco/news/san-francisco-bay-area-fifa-world-cup-2026-match-schedule-six-games/), [FOX Sports](https://www.foxsports.com/stories/soccer/2026-world-cup-matches-san-francisco-start-times-dates-locations)

### Boston (gillette)
| Match # | Date | Fix |
|---------|------|-----|
| 5 | Jun 13 | Time 12:00 → 21:00, Teams C3/C4 → C4/C3 (Haiti vs Scotland) |
| 17→18 | Jun 16 | Match # 17→18, Time 12:00 → 18:00 |
| 29→30 | Jun 19 | Match # 29→30, Time 12:00 → 18:00, Teams C4/C2 → C3/C2 |
| 47→45 | Jun 23 | Match # 47→45, Time 18:00 → 16:00 |
| 62→61 | Jun 26 | Match # 62→61 |
| 74 | Jun 29 | Time 13:00 → 16:30, Teams 1A/3C/D/E → 1E/3A/B/C/D/F (R32) |

**Notes:**
- Boston hosts 7 matches (5 group stage + 1 R32 + 1 QF)
- Multiple match number corrections needed
- Official sources: [Boston FWC26](https://bostonfwc26.com/match-schedule/), [FOX Sports](https://www.foxsports.com/stories/soccer/2026-world-cup-matches-boston-start-times-dates-locations)

### Toronto (bmo)
| Match # | Date | Fix |
|---------|------|-----|
| 3 | Jun 12 | Teams B1/B2 → B1/B4 (Canada vs UEFA Playoff A) |
| 23 | Jun 17 | Time 18:00 → 19:00 |
| 34 | Jun 20 | Time 15:00 → 16:00, Teams E1/E3 → E1/E2 (Germany vs Ivory Coast) |
| 48 | Jun 23 | Time 21:00 → 19:00 |
| 61 | Jun 26 | Teams I4/I1 → I2/I4 (Senegal vs FIFA Playoff 2) |
| 83 | Jul 2 | Teams 1H/3G/K/L → 2K/2L (R32, scraper only) |

**Notes:**
- Toronto hosts 6 matches (5 group stage + 1 R32)
- Official sources: [Toronto FWC26](https://torontofwc26.ca/game), [FOX Sports](https://www.foxsports.com/stories/soccer/2026-world-cup-matches-toronto-start-times-dates-locations)

### Vancouver (bc_place)
| Match # | Date | Fix |
|---------|------|-----|
| 8 | Jun 13→14 | Date 2026-06-13 → 2026-06-14, Time 21:00 → 00:00, Teams D3/D4 → D2/D4 |
| 26 | Jun 18 | Time 15:00 → 18:00 |
| 54 | Jun 24 | Time 18:00 → 15:00, Teams B1/B4 → B2/B1 |
| 64 | Jun 26 | Time 18:00 → 23:00 |
| 85 | Jul 2 | Time → 23:00, Teams 1F/3H/I/J → 1B/3E/F/G/I/J (R32) |
| 96 | Jul 7 | Teams W15/W16 → W85/W87 (R16) |

**Notes:**
- Vancouver hosts 7 matches (5 group stage + 1 R32 + 1 R16)
- Late-night PT times cross midnight into next day ET
- Official sources: [Vancouver FWC26](https://www.vancouverfwc26.ca/match-centre/match-day-schedule), [BC Place](https://www.bcplace.com/)

### Mexico City (azteca)
| Match # | Date | Fix |
|---------|------|-----|
| 1 | Jun 11 | Time 12:00 → 15:00 (Opening match) |
| 21 | Jun 17 | Time 12:00 → 22:00, Teams K1/K2 → K3/K2 (Uzbekistan vs Colombia) |
| 51 | Jun 24 | Time 18:00 → 21:00 |
| 79 | Jun 30 | Teams 1E/3G/H/I → 1A/3C/E/F/H/I (R32, scraper) |
| 92 | Jul 5 | Added Azteca to R16 venues (scraper) |

**Notes:**
- Mexico City hosts 5 matches (3 group stage + 1 R32 + 1 R16)
- Includes tournament opening match (Mexico vs South Africa)
- Official sources: [FOX Sports](https://www.foxsports.com/stories/soccer/2026-world-cup-matches-mexico-city-start-times-dates-locations)

### Monterrey (bbva)
| Match # | Date | Fix |
|---------|------|-----|
| 12 | Jun 14 | Time 21:00 → 22:00 |
| 35 | Jun 20→21 | Date 2026-06-20 → 2026-06-21, Time 23:00 → 00:00, Teams F2/F3 → F3/F2 |
| 52 | Jun 24 | Time 18:00 → 21:00 |
| 75 | Jun 29 | Time 19:00 → 21:00, Teams 1C/3A/B/F → 1F/2C (R32) |

**Notes:**
- Monterrey hosts 4 matches (3 group stage + 1 R32)
- Hosts the historic 1000th FIFA World Cup match (Japan vs Tunisia)
- Official sources: [FIFA.com](https://www.fifa.com/en/tournaments/mens/worldcup/canadamexicousa2026/articles/monterrey-host-four-matches-stadium-estadio), [FOX Sports](https://www.foxsports.com/stories/soccer/2026-world-cup-matches-monterrey-start-times-dates-locations)

### Guadalajara (akron)
| Match # | Date | Fix |
|---------|------|-----|
| 2 | Jun 11 | Time 18:00 → 22:00 |
| 27→28 | Jun 18 | Match # 27→28, Time 18:00 → 21:00 |
| 46→48 | Jun 23 | Match # 46→48, Time 15:00 → 22:00, Teams K4/K2 → K2/K4 |
| 65→66 | Jun 26 | Match # 65→66, Time 21:00 → 20:00, Teams H4/H1 → H3/H1 |

**Notes:**
- Guadalajara hosts 4 matches (all group stage)
- Multiple match number corrections needed
- Official sources: [FOX Sports](https://www.foxsports.com/stories/soccer/2026-world-cup-matches-guadalajara-start-times-dates-locations)

## Validation Process

For each venue:

1. **Extract matches** from `schedule.json` for the venue
2. **Find authoritative source**: Official host city page, FIFA.com, MLS Soccer, FOX Sports
3. **Compare fields**: date, time (ET), round, team placeholders
4. **Document discrepancies** in table format
5. **Fix both files**: `web/public/data/schedule.json` and `scrapers/sources/schedule_scraper.py`

## Key Sources

- [FIFA World Cup 2026 Match Schedule](https://www.fifa.com/en/tournaments/mens/worldcup/canadamexicousa2026/articles/match-schedule-fixtures-results-teams-stadiums)
- [MLS Soccer Schedule](https://www.mlssoccer.com/news/fifa-world-cup-2026-schedule-every-game-by-city-stadium)
- Individual host city pages (nynjfwc26.com, dallasfwc26.com, miamifwc26.com, etc.)
- FOX Sports venue-specific schedule articles

## Files Modified

- `web/public/data/schedule.json` - Runtime schedule data
- `scrapers/sources/schedule_scraper.py` - Source generator (kept in sync)

## Next Steps

1. ~~Continue validation for remaining venues~~ ✅ **All 16 venues validated**
2. Fix scraper knockout bracket structure (R32/R16) to fully match schedule.json
3. Regenerate `schedule.json` from scraper to ensure complete sync
4. Add automated validation tests comparing against known fixtures
5. Address duplicate match numbers found in schedule.json (matches 18, 30, 45, 61)
