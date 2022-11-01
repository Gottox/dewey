extern crate dewey;

use dewey::VersionCmp;
use std::fs::File;
use std::io::{self, BufRead};

#[test]
fn test_all() {
  let file = File::open("tests/versions").unwrap();
  let versions = io::BufReader::new(file).lines();
  let vec1: Vec<_> = versions.map(Result::unwrap).collect();
  let vec2 = vec1.clone();
  for v1 in vec1.iter() {
    for v2 in vec2.iter() {
      v1.ver_cmp(v2.as_str());
    }
  }
}
