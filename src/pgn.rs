use std::collections::HashMap;

use regex::Regex;
use streaming_iterator::StreamingIterator;

use crate::notation::{AlgebraicMove, self};


pub struct StrIter<'a, Reader: Iterator<Item = String>> {
    line: Option<String>,
    reader: &'a mut Reader,
}

impl<'a, T: Iterator<Item = String>> StrIter<'a, T> {
    pub fn new(itr: &'a mut T) -> Self {
        StrIter {line: None, reader: itr}
    }
}

impl<'a, T: Iterator<Item = String>> StreamingIterator for StrIter<'a, T> {
    type Item = str;

    fn advance(&mut self) {
        self.line = self.reader.next();
    }

    fn get(&self) -> Option<&Self::Item> {
        match &self.line {
            Some(t) => Some(t),
            None => None,
        }
    }
}

pub struct PGNFileReader<'a, Reader: StreamingIterator<Item = str>> {
    reader: &'a mut Reader,

}

pub struct PGNChessGame {
    pub moves: Vec<AlgebraicMove>,
    pub meta: HashMap<String, String>,
}

pub fn read_pgn_file<'a, Reader: StreamingIterator<Item = str>>(reader: &'a mut Reader) -> PGNFileReader<'a, Reader> {
    PGNFileReader{reader}
}

impl<'a, T: StreamingIterator<Item = str>> Iterator for PGNFileReader<'a, T> {
    type Item = PGNChessGame;

    fn next(&mut self) -> Option<Self::Item> {
        let mut meta = HashMap::new();
        while {
            let mut line = self.reader.next()?;
            if line.starts_with('[') {
                line = line.trim_start_matches('[');
                line = line.trim_end_matches(']');
                let (key, val) = line.split_once(" ")?;
                meta.insert(key.to_string(), val.to_string());
                true
            }
            else if line == "" {
                true
            }
            else {
                false
            }
        }{}
        let move_match = Regex::new(r"([0-9]+)\. ?([1-8xa-hBNRQKO\-\+#]+) (\{[^\}]*\})? ?([0-9]+\.\.\.)? ?([1-8xa-hBNRQKO\-\+#]+ )?(\{[^\}]*\})?").unwrap();
        let mut moves = Vec::new();
        for i in move_match.captures_iter(self.reader.get()?) {
            let white_move = i.get(2).unwrap().as_str();
            moves.push(notation::str_to_algebraic(white_move).unwrap());
            if let Some(mov) = i.get(5) {
                moves.push(notation::str_to_algebraic(mov.as_str()).unwrap())
            }
        }

        Some(PGNChessGame{moves, meta})
    }
}