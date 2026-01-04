use anyhow::Result;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;

pub fn tail_lines(path: &Path, max_lines: usize) -> Result<Vec<String>> {
	if !path.exists() {
		return Ok(vec![]);
	}
	let mut file = File::open(path)?;

	// Seek to end and read backwards to only process tail of file
	let file_size = file.metadata()?.len();
	if file_size == 0 {
		return Ok(vec![]);
	}

	// Read last ~64KB max (enough for most previews)
	let read_size = std::cmp::min(file_size, 65536);
	let start_pos = file_size.saturating_sub(read_size);
	file.seek(SeekFrom::Start(start_pos))?;

	let reader = BufReader::new(file);
	let mut buf = VecDeque::with_capacity(max_lines);

	for line in reader.lines() {
		if let Ok(line) = line {
			for piece in split_cr_lines(&line) {
				// Keep only the latest segment for carriage-return updates to avoid flooding.
				let segment = if piece.contains('\r') {
					piece.rsplit('\r').next().unwrap_or(piece)
				} else {
					piece
				};
				if buf.len() == max_lines {
					buf.pop_front();
				}
				let stripped = strip_ansi_fast(segment);
				if stripped.is_empty() {
					continue;
				}
				buf.push_back(stripped);
			}
		}
	}
	Ok(buf.into_iter().collect())
}

/// Fast ANSI escape sequence stripper without regex
fn strip_ansi_fast(input: &str) -> String {
	let mut result = String::with_capacity(input.len());
	let mut chars = input.chars().peekable();

	while let Some(c) = chars.next() {
		if c == '\x1b' {
			// ESC sequence - skip until end
			if chars.peek() == Some(&'[') {
				chars.next(); // consume '['
				// Skip parameter bytes (0x30-0x3F) and intermediate bytes (0x20-0x2F)
				// until final byte (0x40-0x7E)
				while let Some(&next) = chars.peek() {
					if next >= '@' && next <= '~' {
						chars.next(); // consume final byte
						break;
					}
					chars.next(); // skip intermediate
				}
			} else if chars.peek() == Some(&']') {
				// OSC sequence - skip until BEL or ST
				chars.next(); // consume ']'
				while let Some(next) = chars.next() {
					if next == '\x07' {
						break;
					}
					if next == '\x1b' && chars.peek() == Some(&'\\') {
						chars.next();
						break;
					}
				}
			}
			// Other ESC sequences - just skip the ESC
		} else if c != '\r' {
			result.push(c);
		}
	}
	result
}

fn split_cr_lines(input: &str) -> Vec<&str> {
	if input.contains('\r') {
		input.split('\r').filter(|s| !s.is_empty()).collect()
	} else {
		vec![input]
	}
}
