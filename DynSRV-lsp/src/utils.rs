use ropey::Rope;
use tower_lsp::lsp_types::Position;


// Converts a byte index in the text to a Position (line and column) for LSP diagnostics.
pub fn byte_to_pos(rope: &Rope, byte: usize) -> Option<Position> {
    let line = rope.byte_to_line(byte);
    let line_start = rope.line_to_byte(line);

    let col = byte - line_start;
    Some(Position::new(line as u32, col as u32))
}
