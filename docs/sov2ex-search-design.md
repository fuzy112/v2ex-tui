# SOV2EX Search Feature Design

## Overview

This document describes the design for integrating SOV2EX search functionality into V2EX TUI. SOV2EX is a third-party search engine for V2EX topics, providing full-text search capabilities that are not available through the official V2EX API.

**SOV2EX Website**: https://www.sov2ex.com  
**API Endpoint**: `https://www.sov2ex.com/api/search`

## API Specification

### Request Parameters

| Parameter | Type | Required | Description | Default |
|-----------|------|----------|-------------|---------|
| `q` | string | Yes | Search query | - |
| `from` | int | No | Offset for pagination | 0 |
| `size` | int | No | Number of results (max 50) | 10 |
| `sort` | string | No | Sort by `sumup` (relevance) or `created` (time) | sumup |
| `order` | int | No | 0=descending, 1=ascending | 0 |
| `node` | string | No | Filter by node name(s) | - |
| `username` | string | No | Filter by author | - |

### Response Structure

```json
{
    "took": 34,
    "timed_out": false,
    "total": 53591,
    "hits": [
        {
            "_source": {
                "node": 11,
                "replies": 13,
                "created": "2016-09-04T01:37:41",
                "member": "jasonailu",
                "id": 303776,
                "title": "Topic title",
                "content": "Topic content..."
            },
            "highlight": {
                "title": ["<em>highlighted</em> title"],
                "content": ["highlighted <em>content</em>"]
            }
        }
    ]
}
```

## Architecture

### Data Flow

```
User Input → SearchState → API Client → SOV2EX API → SearchResult[] → SearchView
```

### Components

#### 1. API Layer (`src/api.rs`)

New structs to add:

```rust
pub struct SearchHit {
    pub node: i64,        // Node ID only
    pub replies: i64,
    pub created: String,  // ISO 8601
    pub member: String,   // Username only
    pub id: i64,          // Topic ID
    pub title: String,
    pub content: String,
}

pub struct SearchHighlight {
    pub title: Option<Vec<String>>,
    pub content: Option<Vec<String>>,
}

pub struct SearchResult {
    #[serde(rename = "_source")]
    pub source: SearchHit,
    pub highlight: Option<SearchHighlight>,
}

pub struct SearchResponse {
    pub took: i64,        // Search time in ms
    pub timed_out: bool,
    pub total: i64,       // Total matching topics
    pub hits: Vec<SearchResult>,
}
```

New method on `V2exClient`:

```rust
pub async fn search_sov2ex(
    &self,
    query: &str,
    from: i32,
    size: i32,
    sort: &str,
) -> Result<SearchResponse>
```

**Note**: Unlike official V2EX API, SOV2EX does not require authentication.

#### 2. State Management (`src/state.rs`)

New `SearchState` struct:

```rust
#[derive(Debug, Default)]
pub struct SearchState {
    pub query: String,           // Current search query
    pub cursor: usize,           // Cursor position in input
    pub results: Vec<SearchResult>,
    pub selected: usize,
    pub from: i32,               // Pagination offset
    pub total: i64,
    pub has_more: bool,
    pub sort_by: SearchSort,
}

pub enum SearchSort {
    Relevance,  // sumup
    Created,    // created
}
```

Methods:
- `insert_char()`, `delete_char()` - Text input editing
- `move_cursor_left()`, `move_cursor_right()` - Cursor navigation
- `next_result()`, `previous_result()` - Result navigation
- `toggle_sort()` - Switch between relevance/time sorting
- `clear_input()`, `clear_results()` - Reset state

#### 3. View (`src/views/search.rs`)

**Layout**:
```
┌─────────────────────────────────────────┐
│ Query: rust programming _ [Sort: Rel]   │  ← Input box (3 lines)
├─────────────────────────────────────────┤
│ [123456] Topic title - @user (10) • 2d  │
│ [123457] Another result - @user2 (5)    │  ← Results list
│ [123458] Third result - @user3 (20)     │
│                                         │
│          1-20 of 1000                   │
└─────────────────────────────────────────┘
```

Features:
- Real-time query input with cursor
- Highlighted search results (HTML stripped)
- Author, reply count, and relative time display
- Topic ID shown in brackets

#### 4. Keybindings (`src/keymap.rs`)

> **Note**: The keymap system is currently being refactored. The details below may be out of date. Please check the current keymap implementation before proceeding.

**Search View Keys**:

| Key | Action |
|-----|--------|
| `/` | Open search view (from other views) |
| `Enter` | Execute search / Open selected result |
| `n` / `↓` | Next result (auto-load more at end) |
| `p` / `↑` | Previous result |
| `s` | Toggle sort (Relevance/Time) |
| `C` | Clear query and results |
| `o` | Open selected topic in browser |
| `g` | Refresh results |
| `<` / `>` | First/last result |
| `C-v` | Page down (5 results) |
| `M-v` | Page up (5 results) |
| `q` / `Esc` | Close search view |
| `l` / `←` | History back |
| `a/m/u` | Navigate to aggregate/notifications/profile |

**Input Mode**:
- Type characters to enter query
- `Backspace` to delete
- `←` / `→` to move cursor

#### 5. App Integration (`src/app.rs`)

New `View` variant:
```rust
pub enum View {
    // ... existing variants
    Search,
}
```

New field in `App`:
```rust
pub search_state: SearchState,
```

New method:
```rust
pub async fn load_search_results(&mut self, client: &V2exClient, append: bool)
```

### Global `/` Keybinding

Add to all keymaps:
- `TopicListKeyMap`
- `TopicDetailKeyMap`
- `NotificationsKeyMap`
- `ProfileKeyMap`
- `AggregateKeyMap`

Pressing `/` in any view opens the search view.

## Design Decisions

### 1. Third-Party API Disclaimer

**Decision**: SOV2EX is not official V2EX. We should add a note in help text indicating this is a third-party service.

**Rationale**: Users should understand that search availability depends on SOV2EX, not V2EX.

### 2. Data Format Differences

**Challenge**: SOV2EX returns simplified topic data:
- Node as ID (not full Node object)
- Member as username string (not full Member object)

**Solution**: When opening a search result, load the full topic via the official V2EX API using the topic ID. This ensures we have complete data for the topic detail view.

### 3. Search Persistence

**Decision**: Search query and results persist when navigating away but are cleared when opening a new search with `/`.

**Rationale**: Allows users to go back and forth between search results and topic details while maintaining context.

### 4. Highlight Display

**Challenge**: SOV2EX returns HTML-highlighted snippets (`<em>keyword</em>`).

**Solution**: Strip HTML tags using `html2text` before display. Future enhancement could parse `<em>` tags and apply theme colors.

### 5. Pagination Strategy

**Approach**: 
- Initial load: 20 results
- Auto-load more when pressing `n` at last result (like existing topic list)
- Update offset and append to results list

### 6. Error Handling

Handle these scenarios:
- Network failures (show error in UI)
- Empty results (show "No results found")
- API timeouts (SOV2EX returns `timed_out: true`)
- Invalid queries (server returns error)

## User Experience

### Workflow

1. **Opening Search**: User presses `/` from any view
2. **Entering Query**: User types search terms
3. **Executing**: User presses `Enter` to search
4. **Browsing**: User navigates results with `n`/`p`
5. **Opening**: User presses `Enter` on a result to view topic
6. **Returning**: User presses `l` to go back to search results

### Sort Options

- **Relevance** (`sumup`): SOV2EX's weighted scoring (default)
- **Time** (`created`): Most recent first

Toggle with `s` key.

### Keyboard Shortcuts Consistency

Search view follows existing patterns:
- `n`/`p` for navigation (Emacs/dired style)
- `l`/`r` for history
- `q`/`Esc` to close
- `a`/`m`/`u` for global navigation

## Implementation Checklist

- [ ] Add search structs to `src/api.rs`
- [ ] Implement `search_sov2ex()` method
- [ ] Add `SearchState` to `src/state.rs`
- [ ] Add `View::Search` to `src/app.rs`
- [ ] Add `search_state` field to `App`
- [ ] Implement `load_search_results()` method
- [ ] Create `src/views/search.rs` view
- [ ] Add `SearchKeyMap` to `src/keymap.rs`
- [ ] Add `/` keybinding to all existing keymaps
- [ ] Add render arm for `View::Search` in `App::render()`
- [ ] Update help text in `src/views/help.rs`
- [ ] Add unit tests for `SearchState`
- [ ] Run `cargo fmt` and `cargo check`
- [ ] Test end-to-end with real queries

## Future Enhancements

1. **Advanced Filters**: Support `node`, `username`, `gte`/`lte` parameters
2. **Highlight Coloring**: Parse `<em>` tags and apply accent color
3. **Search History**: Remember recent queries
4. **Saved Searches**: Allow bookmarking common searches
5. **Integration with Node View**: Search within specific node

## References

- **SOV2EX API Docs**: https://github.com/bynil/sov2ex/blob/v2/API.md
- **V2EX Official API**: https://www.v2ex.com/help/api
- **Design Philosophy**: Follow existing patterns for views, keymaps, and state management
