use ropey::Rope;
use tower_lsp::lsp_types::Position;


// Converts a byte index in the text to a Position (line and column) for LSP diagnostics.
pub fn byte_to_pos(rope: &Rope, byte: usize) -> Option<Position> {
    let line = rope.byte_to_line(byte);
    let line_start = rope.line_to_byte(line);

    let col = byte - line_start;
    Some(Position::new(line as u32, col as u32))
}


// The functions is derived from the "tower-lsp-boilerplate" project by IWANABETHATGUY under the MIT Licence
pub fn pos_to_offset(pos: Position, rope: &Rope) -> Option<u32> {
  if pos.line as usize >= rope.len_lines() {
    return None;
  }
  let line_byte_offset = rope.line_to_byte(pos.line as usize);
  let offset = line_byte_offset + pos.character as usize;
  Some(offset as u32)
}