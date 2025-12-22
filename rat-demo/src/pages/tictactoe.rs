//! Gomoku (Five in a Row) - Human vs AI game
//! Showcases: Component composition, AI heuristics, State management, Canvas rendering, Mouse support

use rat_nexus::{Component, Context, EventContext, Event, Action, Entity};
use ratatui::{
    layout::{Layout, Constraint, Direction, Alignment, Rect},
    widgets::{Block, Borders, Paragraph, BorderType, canvas::{Canvas, Line as CanvasLine, Circle}},
    style::{Style, Color, Modifier},
    text::{Line, Span},
};
use crossterm::event::{KeyCode, MouseEventKind, MouseButton};

const BOARD_SIZE: usize = 15;
const WIN_COUNT: usize = 5;

// ============================================
// Cell Component - Single grid cell
// ============================================
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cell {
    Empty,
    Black,  // Human player (goes first)
    White,  // AI player
}

impl Cell {
    pub fn symbol(&self) -> &'static str {
        match self {
            Cell::Empty => "·",
            Cell::Black => "●",
            Cell::White => "○",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Cell::Empty => Color::DarkGray,
            Cell::Black => Color::White,
            Cell::White => Color::Yellow,
        }
    }

    pub fn opponent(&self) -> Cell {
        match self {
            Cell::Black => Cell::White,
            Cell::White => Cell::Black,
            Cell::Empty => Cell::Empty,
        }
    }
}

// ============================================
// Board Component - 15x15 game grid
// ============================================
#[derive(Clone)]
pub struct Board {
    cells: [[Cell; BOARD_SIZE]; BOARD_SIZE],
    last_move: Option<(usize, usize)>,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            cells: [[Cell::Empty; BOARD_SIZE]; BOARD_SIZE],
            last_move: None,
        }
    }
}

impl Board {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, row: usize, col: usize) -> Cell {
        self.cells[row][col]
    }

    pub fn set(&mut self, row: usize, col: usize, cell: Cell) {
        self.cells[row][col] = cell;
        self.last_move = Some((row, col));
    }

    pub fn is_empty(&self, row: usize, col: usize) -> bool {
        self.cells[row][col] == Cell::Empty
    }

    pub fn is_full(&self) -> bool {
        self.cells.iter().all(|row| row.iter().all(|c| *c != Cell::Empty))
    }

    /// Check if there's a winner, returns the winning cell type and winning line
    pub fn check_winner(&self) -> Option<(Cell, Vec<(usize, usize)>)> {
        let directions = [
            (0, 1),   // horizontal
            (1, 0),   // vertical
            (1, 1),   // diagonal \
            (1, -1),  // diagonal /
        ];

        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                let cell = self.cells[row][col];
                if cell == Cell::Empty {
                    continue;
                }

                for (dr, dc) in directions {
                    let mut line = vec![(row, col)];
                    let mut r = row as i32 + dr;
                    let mut c = col as i32 + dc;

                    while r >= 0 && r < BOARD_SIZE as i32 && c >= 0 && c < BOARD_SIZE as i32 {
                        if self.cells[r as usize][c as usize] == cell {
                            line.push((r as usize, c as usize));
                            if line.len() >= WIN_COUNT {
                                return Some((cell, line));
                            }
                            r += dr;
                            c += dc;
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        None
    }

    pub fn reset(&mut self) {
        self.cells = [[Cell::Empty; BOARD_SIZE]; BOARD_SIZE];
        self.last_move = None;
    }
}

// ============================================
// AI Component - Heuristic-based evaluation
// ============================================
pub struct AI;

impl AI {
    /// Find the best move using heuristic evaluation
    pub fn find_best_move(board: &Board) -> Option<(usize, usize)> {
        let mut best_score = i32::MIN;
        let mut best_moves = Vec::new();

        // If board is empty, play center
        if board.last_move.is_none() {
            return Some((BOARD_SIZE / 2, BOARD_SIZE / 2));
        }

        // Only consider positions near existing pieces
        let candidates = Self::get_candidate_moves(board);

        for (row, col) in candidates {
            if !board.is_empty(row, col) {
                continue;
            }

            // Evaluate this position
            let score = Self::evaluate_position(board, row, col, Cell::White);

            if score > best_score {
                best_score = score;
                best_moves.clear();
                best_moves.push((row, col));
            } else if score == best_score {
                best_moves.push((row, col));
            }
        }

        // Return random best move for variety
        if !best_moves.is_empty() {
            use std::time::{SystemTime, UNIX_EPOCH};
            let seed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .subsec_nanos() as usize;
            Some(best_moves[seed % best_moves.len()])
        } else {
            None
        }
    }

    /// Get positions near existing pieces (within 2 cells)
    fn get_candidate_moves(board: &Board) -> Vec<(usize, usize)> {
        let mut candidates = std::collections::HashSet::new();

        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                if board.cells[row][col] != Cell::Empty {
                    // Add nearby empty cells
                    for dr in -2i32..=2 {
                        for dc in -2i32..=2 {
                            let nr = row as i32 + dr;
                            let nc = col as i32 + dc;
                            if nr >= 0 && nr < BOARD_SIZE as i32
                                && nc >= 0 && nc < BOARD_SIZE as i32
                            {
                                let nr = nr as usize;
                                let nc = nc as usize;
                                if board.is_empty(nr, nc) {
                                    candidates.insert((nr, nc));
                                }
                            }
                        }
                    }
                }
            }
        }

        candidates.into_iter().collect()
    }

    /// Evaluate a position's score
    fn evaluate_position(board: &Board, row: usize, col: usize, player: Cell) -> i32 {
        let opponent = player.opponent();

        // Check offensive score (if we place here)
        let offensive = Self::count_patterns(board, row, col, player);

        // Check defensive score (block opponent)
        let defensive = Self::count_patterns(board, row, col, opponent);

        // Prioritize: Win > Block opponent win > Attack > Defense
        let mut score = 0;

        // Offensive scoring
        if offensive.five >= 1 { score += 100000; }      // Win!
        if offensive.open_four >= 1 { score += 50000; }  // Guaranteed win
        if offensive.four >= 1 { score += 10000; }       // Threat
        if offensive.open_three >= 1 { score += 5000; }  // Strong attack
        if offensive.three >= 1 { score += 1000; }       // Attack
        if offensive.open_two >= 1 { score += 500; }     // Development
        if offensive.two >= 1 { score += 100; }          // Presence

        // Defensive scoring (slightly lower priority)
        if defensive.five >= 1 { score += 90000; }       // Must block!
        if defensive.open_four >= 1 { score += 45000; }  // Must block
        if defensive.four >= 1 { score += 9000; }        // Should block
        if defensive.open_three >= 1 { score += 4500; }  // Should block
        if defensive.three >= 1 { score += 900; }        // Consider blocking

        score
    }

    /// Count pattern types in all directions from a position
    fn count_patterns(board: &Board, row: usize, col: usize, player: Cell) -> PatternCount {
        let directions = [
            (0, 1),   // horizontal
            (1, 0),   // vertical
            (1, 1),   // diagonal \
            (1, -1),  // diagonal /
        ];

        let mut patterns = PatternCount::default();

        for (dr, dc) in directions {
            let (count, open_ends) = Self::count_line(board, row, col, dr, dc, player);

            match (count, open_ends) {
                (5.., _) => patterns.five += 1,
                (4, 2) => patterns.open_four += 1,
                (4, 1) => patterns.four += 1,
                (3, 2) => patterns.open_three += 1,
                (3, 1) => patterns.three += 1,
                (2, 2) => patterns.open_two += 1,
                (2, 1) => patterns.two += 1,
                _ => {}
            }
        }

        patterns
    }

    /// Count consecutive pieces in a line and number of open ends
    fn count_line(board: &Board, row: usize, col: usize, dr: i32, dc: i32, player: Cell) -> (usize, usize) {
        let mut count = 1; // Include the position itself
        let mut open_ends = 0;

        // Count forward
        let mut r = row as i32 + dr;
        let mut c = col as i32 + dc;
        while r >= 0 && r < BOARD_SIZE as i32 && c >= 0 && c < BOARD_SIZE as i32 {
            let cell = board.cells[r as usize][c as usize];
            if cell == player {
                count += 1;
                r += dr;
                c += dc;
            } else {
                if cell == Cell::Empty {
                    open_ends += 1;
                }
                break;
            }
        }
        if r < 0 || r >= BOARD_SIZE as i32 || c < 0 || c >= BOARD_SIZE as i32 {
            // Edge of board, not open
        }

        // Count backward
        r = row as i32 - dr;
        c = col as i32 - dc;
        while r >= 0 && r < BOARD_SIZE as i32 && c >= 0 && c < BOARD_SIZE as i32 {
            let cell = board.cells[r as usize][c as usize];
            if cell == player {
                count += 1;
                r -= dr;
                c -= dc;
            } else {
                if cell == Cell::Empty {
                    open_ends += 1;
                }
                break;
            }
        }

        (count, open_ends)
    }
}

#[derive(Default)]
struct PatternCount {
    five: usize,
    open_four: usize,
    four: usize,
    open_three: usize,
    three: usize,
    open_two: usize,
    two: usize,
}

// ============================================
// Game State
// ============================================
#[derive(Clone, Copy, PartialEq)]
pub enum GameStatus {
    Playing,
    HumanWon,
    AIWon,
    Draw,
}

#[derive(Clone)]
pub struct GomokuState {
    board: Board,
    cursor: (usize, usize),
    status: GameStatus,
    human_score: u32,
    ai_score: u32,
    is_human_turn: bool,
    winning_line: Option<Vec<(usize, usize)>>,
}

impl Default for GomokuState {
    fn default() -> Self {
        Self {
            board: Board::new(),
            cursor: (BOARD_SIZE / 2, BOARD_SIZE / 2),
            status: GameStatus::Playing,
            human_score: 0,
            ai_score: 0,
            is_human_turn: true,
            winning_line: None,
        }
    }
}

impl GomokuState {
    fn screen_to_cell(x: u16, y: u16, board_area: Rect) -> Option<(usize, usize)> {
        if board_area.width == 0 || board_area.height == 0 {
            return None;
        }

        let inner_x = x.checked_sub(board_area.x + 1)?;
        let inner_y = y.checked_sub(board_area.y + 1)?;

        let inner_width = board_area.width.saturating_sub(2);
        let inner_height = board_area.height.saturating_sub(2);

        if inner_x >= inner_width || inner_y >= inner_height {
            return None;
        }

        let col = (inner_x as usize * BOARD_SIZE) / inner_width as usize;
        let row = (inner_y as usize * BOARD_SIZE) / inner_height as usize;

        if row < BOARD_SIZE && col < BOARD_SIZE {
            Some((row, col))
        } else {
            None
        }
    }

    fn make_human_move(&mut self) -> bool {
        self.make_move_at(self.cursor.0, self.cursor.1)
    }

    fn make_move_at(&mut self, row: usize, col: usize) -> bool {
        if self.status != GameStatus::Playing || !self.is_human_turn {
            return false;
        }

        if !self.board.is_empty(row, col) {
            return false;
        }

        self.board.set(row, col, Cell::Black);
        self.check_game_status();

        if self.status == GameStatus::Playing {
            self.is_human_turn = false;
        }

        true
    }

    fn make_ai_move(&mut self) {
        if self.status != GameStatus::Playing || self.is_human_turn {
            return;
        }

        if let Some((row, col)) = AI::find_best_move(&self.board) {
            self.board.set(row, col, Cell::White);
            self.check_game_status();
        }

        self.is_human_turn = true;
    }

    fn check_game_status(&mut self) {
        if let Some((winner, line)) = self.board.check_winner() {
            self.winning_line = Some(line);
            match winner {
                Cell::Black => {
                    self.status = GameStatus::HumanWon;
                    self.human_score += 1;
                }
                Cell::White => {
                    self.status = GameStatus::AIWon;
                    self.ai_score += 1;
                }
                _ => {}
            }
        } else if self.board.is_full() {
            self.status = GameStatus::Draw;
        }
    }

    fn reset(&mut self) {
        self.board.reset();
        self.cursor = (BOARD_SIZE / 2, BOARD_SIZE / 2);
        self.status = GameStatus::Playing;
        self.is_human_turn = true;
        self.winning_line = None;
    }
}

// ============================================
// Gomoku Page Component (renamed from TicTacToe)
// ============================================
pub struct TicTacToePage {
    state: Entity<GomokuState>,
    board_area: Rect,  // Store separately to avoid update in render
}

impl TicTacToePage {
    pub fn new(cx: &rat_nexus::AppContext) -> Self {
        Self {
            state: cx.new_entity(GomokuState::default()),
            board_area: Rect::default(),
        }
    }

    fn render_board(&self, frame: &mut ratatui::Frame, area: Rect, state: &GomokuState) {
        let winning_line = state.winning_line.clone();
        let last_move = state.board.last_move;
        let cursor = state.cursor;
        let is_playing = state.status == GameStatus::Playing;

        let canvas = Canvas::default()
            .block(Block::default()
                .title(format!(" Gomoku {}x{} ", BOARD_SIZE, BOARD_SIZE))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan)))
            .x_bounds([0.0, 100.0])
            .y_bounds([0.0, 100.0])
            .paint(move |ctx| {
                let margin = 5.0;
                let cell_size = (100.0 - 2.0 * margin) / (BOARD_SIZE - 1) as f64;

                // Draw grid lines
                for i in 0..BOARD_SIZE {
                    let pos = margin + i as f64 * cell_size;
                    // Vertical
                    ctx.draw(&CanvasLine {
                        x1: pos, y1: margin,
                        x2: pos, y2: 100.0 - margin,
                        color: Color::DarkGray,
                    });
                    // Horizontal
                    ctx.draw(&CanvasLine {
                        x1: margin, y1: pos,
                        x2: 100.0 - margin, y2: pos,
                        color: Color::DarkGray,
                    });
                }

                // Draw star points (for 15x15 board)
                let star_points = [(3, 3), (3, 11), (7, 7), (11, 3), (11, 11)];
                for (sr, sc) in star_points {
                    let sx = margin + sc as f64 * cell_size;
                    let sy = 100.0 - margin - sr as f64 * cell_size;
                    ctx.draw(&Circle {
                        x: sx, y: sy,
                        radius: 0.8,
                        color: Color::Gray,
                    });
                }

                // Draw pieces
                for row in 0..BOARD_SIZE {
                    for col in 0..BOARD_SIZE {
                        let cell = state.board.get(row, col);
                        if cell == Cell::Empty {
                            continue;
                        }

                        let cx = margin + col as f64 * cell_size;
                        let cy = 100.0 - margin - row as f64 * cell_size;

                        // Check if this is part of winning line
                        let is_winning = winning_line.as_ref().map_or(false, |line| {
                            line.iter().any(|&(r, c)| r == row && c == col)
                        });

                        // Check if this is the last move
                        let is_last = last_move == Some((row, col));

                        let color = if is_winning {
                            Color::Green
                        } else {
                            cell.color()
                        };

                        // Draw piece
                        ctx.draw(&Circle {
                            x: cx, y: cy,
                            radius: cell_size * 0.4,
                            color,
                        });

                        // Mark last move with a dot
                        if is_last && !is_winning {
                            let dot_color = if cell == Cell::Black { Color::Black } else { Color::DarkGray };
                            ctx.draw(&Circle {
                                x: cx, y: cy,
                                radius: cell_size * 0.1,
                                color: dot_color,
                            });
                        }
                    }
                }

                // Draw cursor
                if is_playing {
                    let cx = margin + cursor.1 as f64 * cell_size;
                    let cy = 100.0 - margin - cursor.0 as f64 * cell_size;
                    let r = cell_size * 0.45;

                    // Cursor as a square outline
                    ctx.draw(&CanvasLine { x1: cx - r, y1: cy - r, x2: cx + r, y2: cy - r, color: Color::Cyan });
                    ctx.draw(&CanvasLine { x1: cx + r, y1: cy - r, x2: cx + r, y2: cy + r, color: Color::Cyan });
                    ctx.draw(&CanvasLine { x1: cx + r, y1: cy + r, x2: cx - r, y2: cy + r, color: Color::Cyan });
                    ctx.draw(&CanvasLine { x1: cx - r, y1: cy + r, x2: cx - r, y2: cy - r, color: Color::Cyan });
                }
            });

        frame.render_widget(canvas, area);
    }

    fn render_info_panel(&self, frame: &mut ratatui::Frame, area: Rect, state: &GomokuState) {
        let status_text = match state.status {
            GameStatus::Playing => {
                if state.is_human_turn { "Your turn (●)" } else { "AI thinking..." }
            }
            GameStatus::HumanWon => "🎉 You Win!",
            GameStatus::AIWon => "🤖 AI Wins!",
            GameStatus::Draw => "🤝 Draw!",
        };

        let status_color = match state.status {
            GameStatus::Playing => Color::Cyan,
            GameStatus::HumanWon => Color::Green,
            GameStatus::AIWon => Color::Red,
            GameStatus::Draw => Color::Yellow,
        };

        let info_lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Status: ", Style::default().fg(Color::DarkGray)),
                Span::styled(status_text, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Score", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("  You (●): ", Style::default().fg(Color::White)),
                Span::styled(format!("{}", state.human_score), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("  AI  (○): ", Style::default().fg(Color::Yellow)),
                Span::styled(format!("{}", state.ai_score), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Cursor: ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("({}, {})", state.cursor.0 + 1, state.cursor.1 + 1), Style::default().fg(Color::Cyan)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Controls", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("  🖱️ Click  ", Style::default().fg(Color::Green)),
                Span::raw("Place stone"),
            ]),
            Line::from(vec![
                Span::styled("  ↑↓←→    ", Style::default().fg(Color::Green)),
                Span::raw("Move cursor"),
            ]),
            Line::from(vec![
                Span::styled("  Enter   ", Style::default().fg(Color::Green)),
                Span::raw("Place stone"),
            ]),
            Line::from(vec![
                Span::styled("  R/RMB   ", Style::default().fg(Color::Green)),
                Span::raw("New game"),
            ]),
            Line::from(vec![
                Span::styled("  M/Esc   ", Style::default().fg(Color::Green)),
                Span::raw("Back to menu"),
            ]),
        ];

        let info = Paragraph::new(info_lines)
            .block(Block::default()
                .title(" Gomoku 五子棋 ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Magenta)));

        frame.render_widget(info, area);
    }
}

impl Component for TicTacToePage {
    fn render(&mut self, frame: &mut ratatui::Frame, cx: &mut Context<Self>) {
        cx.subscribe(&self.state);
        let state = self.state.read(|s| s.clone()).unwrap_or_default();
        let area = frame.area();

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        // Header
        let header_text = match state.status {
            GameStatus::Playing => "🎮 Gomoku - Human vs AI",
            GameStatus::HumanWon => "🎉 Victory! Five in a row!",
            GameStatus::AIWon => "🤖 AI Wins!",
            GameStatus::Draw => "🤝 It's a Draw!",
        };
        let header_color = match state.status {
            GameStatus::Playing => Color::Cyan,
            GameStatus::HumanWon => Color::Green,
            GameStatus::AIWon => Color::Red,
            GameStatus::Draw => Color::Yellow,
        };

        let header = Paragraph::new(format!(" {} ", header_text))
            .style(Style::default().fg(header_color).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded));
        frame.render_widget(header, main_layout[0]);

        // Content
        let content_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
            .split(main_layout[1]);

        // Store board area for mouse click detection (no state update needed)
        self.board_area = content_layout[0];

        self.render_board(frame, content_layout[0], &state);
        self.render_info_panel(frame, content_layout[1], &state);

        // Footer
        let footer = Paragraph::new(" Click/Enter Place | ↑↓←→ Move | R Reset | M Menu | Q Quit ")
            .style(Style::default().bg(Color::Cyan).fg(Color::Black))
            .alignment(Alignment::Center);
        frame.render_widget(footer, main_layout[2]);
    }

    fn handle_event(&mut self, event: Event, _cx: &mut EventContext<Self>) -> Option<Action> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => Some(Action::Quit),
                KeyCode::Char('m') | KeyCode::Esc => Some(Action::Navigate("menu".to_string())),
                KeyCode::Char('r') => {
                    let _ = self.state.update(|s| s.reset());
                    None
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let _ = self.state.update(|s| {
                        if s.cursor.0 > 0 { s.cursor.0 -= 1; }
                    });
                    None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let _ = self.state.update(|s| {
                        if s.cursor.0 < BOARD_SIZE - 1 { s.cursor.0 += 1; }
                    });
                    None
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    let _ = self.state.update(|s| {
                        if s.cursor.1 > 0 { s.cursor.1 -= 1; }
                    });
                    None
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    let _ = self.state.update(|s| {
                        if s.cursor.1 < BOARD_SIZE - 1 { s.cursor.1 += 1; }
                    });
                    None
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    let _ = self.state.update(|s| {
                        if s.make_human_move() {
                            s.make_ai_move();
                        }
                    });
                    None
                }
                _ => None,
            },
            Event::Mouse(mouse) => {
                match mouse.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        let board_area = self.board_area;
                        let _ = self.state.update(|s| {
                            if let Some((row, col)) = GomokuState::screen_to_cell(mouse.column, mouse.row, board_area) {
                                s.cursor = (row, col);
                                if s.make_move_at(row, col) {
                                    s.make_ai_move();
                                }
                            }
                        });
                        None
                    }
                    MouseEventKind::Down(MouseButton::Right) => {
                        let _ = self.state.update(|s| s.reset());
                        None
                    }
                    MouseEventKind::Moved => {
                        let board_area = self.board_area;
                        let _ = self.state.update(|s| {
                            if s.status == GameStatus::Playing {
                                if let Some((row, col)) = GomokuState::screen_to_cell(mouse.column, mouse.row, board_area) {
                                    s.cursor = (row, col);
                                }
                            }
                        });
                        None
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}
