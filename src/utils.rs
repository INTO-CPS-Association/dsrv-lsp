/*
 * Copyright (c) 2026 Emilie Bang Holmberg (github.com/EmmiPigen).
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License.
 *
 * This project utilizes the 'trustworthiness-checker' crate, which is
 * property of the INTO-CPS Association and used under the ICAPL (GPL Mode).
 */

use ropey::Rope;
use tower_lsp_server::ls_types::Position;

// Converts a byte index in the text to a Position (line and column) for LSP diagnostics.
pub fn byte_to_pos(rope: &Rope, byte: usize) -> Option<Position> {
    if byte > rope.len_bytes() {
        return None;
    }

    let line = rope.try_byte_to_line(byte).ok()?;
    let line_start = rope.try_line_to_byte(line).ok()?;

    let col = byte - line_start;
    Some(Position::new(line as u32, col as u32))
}

// The functions is derived from the "tower-lsp-boilerplate" project by IWANABETHATGUY under the MIT Licence
pub fn pos_to_offset(pos: Position, rope: &Rope) -> Option<u32> {
    if pos.line as usize >= rope.len_lines() {
        return None;
    }

    let line_byte_offset = rope.try_line_to_byte(pos.line as usize).ok()?;
    let offset = line_byte_offset.saturating_add(pos.character as usize);
    if offset > rope.len_bytes() {
        return None;
    }

    Some(offset as u32)
}

#[cfg(test)]
mod test {
    use crate::fixtures;

    use super::*;

    #[test]
    fn test_byte_to_pos_simple() {
        let rope = Rope::from_str(fixtures::input_untyped_valid_simple());
        println!("Rope content:\n{}", rope);

        let pos = byte_to_pos(&rope, 7).unwrap();

        let result = Position::new(1, 2); // line 2 (0-based) and column 2 (0-based)
        println!("Position: {:#?}", pos);

        assert_eq!(
            pos, result,
            "Expected position {:#?} but got {:#?}",
            result, pos
        );
    }

    #[test]
    fn test_byte_to_pos_complex() {
        let rope = Rope::from_str(fixtures::input_untyped_complex_with_comments());

        let pos = byte_to_pos(&rope, 523);
        let result = Position::new(18, 23);
        println!("Position: {:#?}", pos);

        assert_eq!(
            pos,
            Some(result),
            "Expected position {:#?} but got {:#?}",
            result,
            pos
        );
    }

    #[test]
    fn test_pos_to_offset_simple() {
        let rope = Rope::from_str(fixtures::input_untyped_valid_simple());
        let pos = Position::new(1, 2); // line 2 (0-based) and column 2 (0-based)
        let offset = pos_to_offset(pos, &rope).unwrap();
        let expected_offset = 7; // 'y' in "in y"
        println!("Offset: {}", offset);
        assert_eq!(
            offset, expected_offset,
            "Expected offset {} but got {}",
            expected_offset, offset
        );
    }

    #[test]
    fn test_pos_to_offset_complex() {
        let rope = Rope::from_str(fixtures::input_untyped_complex_with_comments());
        let pos = Position::new(18, 23);
        let offset = pos_to_offset(pos, &rope).unwrap();
        let expected_offset = 523; // 'y' in "z = x + y"
        println!("Offset: {}", offset);
        assert_eq!(
            offset, expected_offset,
            "Expected offset {} but got {}",
            expected_offset, offset
        );
    }
}
